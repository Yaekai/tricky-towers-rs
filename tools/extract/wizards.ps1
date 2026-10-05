param([string]$Repo)
# Stages wizard1..14 (the original's Wizard01..04, WizardAnimal01..04, WizardIndie01..06) per "Wizards" in docs/art-layout.md: parts in
# drawing order with positions and pivots, each clip's frame animations, the cape wave and voice
# lines. Appends to extracted\manifest.ron and extracted\staged.tsv. Run after stage.ps1.
Add-Type -AssemblyName System.Drawing
$ErrorActionPreference = 'Stop'
$inv = [Globalization.CultureInfo]::InvariantCulture
$A = Join-Path $Repo 'research\ripped\ExportedProject\Assets'
$X = Join-Path $Repo 'extracted'
$G = Join-Path $Repo 'research\guid_index.tsv'
$utf8 = New-Object System.Text.UTF8Encoding($false)
$idx = @{}; Get-Content $G | ForEach-Object { $p = $_ -split "`t"; $idx[$p[0]] = $p[1] }
$num = '(-?[0-9.]+(?:[eE][-+]?\d+)?)'
$texCache = @{}
$entries = New-Object System.Collections.Generic.List[string]
$stagedRows = New-Object System.Collections.Generic.List[string]
$imageKey = @{}   # "<n>:<sprite>" -> staged key

function Snake($s) {
  $s = [regex]::Replace($s, '([a-z0-9])([A-Z])', '$1_$2')
  return ($s.ToLowerInvariant() -replace '[^a-z0-9]+', '_').Trim('_')
}
function F($v) { [string]::Format($inv, '{0:0.0###}', [double]$v) }

# Crops a sprite from its atlas; returns its staged key (one per wizard and sprite).
function Stage-Image($n, $sprite, $key, $position = $null) {
  if ($imageKey.ContainsKey("${n}:$sprite") -and -not $position) { return $imageKey["${n}:$sprite"] }
  $t = [IO.File]::ReadAllText((Join-Path $A "Sprite\$sprite.asset"))
  $r = [regex]::Match($t, "textureRect:\s*\n\s*serializedVersion: 2\s*\n\s*x: $num\s*\n\s*y: $num\s*\n\s*width: $num\s*\n\s*height: $num")
  $pv = [regex]::Match($t, "m_Pivot: \{x: $num, y: $num\}")
  $ppu = [regex]::Match($t, "m_PixelsToUnits: $num").Groups[1].Value
  $tex = $idx[[regex]::Match($t, 'm_RD:\s*\n\s*texture: \{fileID: \d+, guid: ([0-9a-f]+)').Groups[1].Value]
  if (-not $texCache.ContainsKey($tex)) { $texCache[$tex] = [System.Drawing.Bitmap]::FromFile($tex) }
  $img = $texCache[$tex]
  $sx = [int][double]$r.Groups[1].Value; $sw = [int][Math]::Round([double]$r.Groups[3].Value); $sh = [int][Math]::Round([double]$r.Groups[4].Value)
  $top = [int]($img.Height - [double]$r.Groups[2].Value - [double]$r.Groups[4].Value)
  $crop = $img.Clone((New-Object System.Drawing.Rectangle $sx, $top, $sw, $sh), $img.PixelFormat)
  $rel = "wizards\w$n\$key.png"
  $out = Join-Path $X $rel
  New-Item -ItemType Directory -Force (Split-Path $out) | Out-Null
  $crop.Save($out, [System.Drawing.Imaging.ImageFormat]::Png); $crop.Dispose()
  $line = "    `"$key`": (file: `"$($rel -replace '\\','/')`", pixels_per_unit: $(F $ppu), pivot: ($(F $pv.Groups[1].Value), $(F $pv.Groups[2].Value))"
  if ($position) { $line += ", position: ($(F $position[0]), $(F $position[1]))" }
  $entries.Add($line + '),')
  $stagedRows.Add("sprite`t$sprite`t$key")
  if (-not $position) { $imageKey["${n}:$sprite"] = $key }
  return $key
}

function Part-Name($path) {
  $nodes = $path -split '/'
  $node = if ($nodes[-1] -eq 'Sprite') { $nodes[-2] } else { $nodes[-1] }
  switch ($node) { 'CloudBack' { 'cloud' } 'Cape' { 'cape' } default { Snake $node } }
}

# Reads a clip: tracks (part name -> list of (time, sprite)), length, looping.
function Read-Clip($name) {
  $f = Join-Path $A "AnimationClip\$name.anim"
  if (-not (Test-Path $f)) { return $null }
  $t = [IO.File]::ReadAllText($f)
  $stop = [double]::Parse([regex]::Match($t, 'm_StopTime: ([\d.eE+-]+)').Groups[1].Value, $inv)
  $loop = [regex]::Match($t, 'm_LoopTime: (\d)').Groups[1].Value -eq '1'
  $pp = $t.IndexOf('m_PPtrCurves:'); $tracks = @{}
  if ($pp -ge 0) {
    $sect = $t.Substring($pp); $e = $sect.IndexOf("`n  m_SampleRate"); if ($e -gt 0) { $sect = $sect.Substring(0, $e) }
    foreach ($tr in ([regex]::Split($sect, '\n  - curve:') | Select-Object -Skip 1)) {
      $path = [regex]::Match($tr, 'path: (.*)').Groups[1].Value.Trim()
      $keys = [regex]::Matches($tr, 'time: ([\d.eE-]+)\s*\n\s*value: \{fileID: \d+, guid: ([0-9a-f]+)')
      if ($keys.Count -eq 0) { continue }
      if (-not $path) { $path = 'root' }
      $tracks[(Part-Name $path)] = @($keys | ForEach-Object { ,@([double]::Parse($_.Groups[1].Value, $inv), [IO.Path]::GetFileNameWithoutExtension($idx[$_.Groups[2].Value])) })
    }
  }
  return [pscustomobject]@{ length = $stop; loop = $loop; tracks = $tracks }
}

# Clip names per wizard, from WizardAnimationController and the Wizard0N override controllers.
$clipMap = @{
  1 = [ordered]@{ idle = 'Idle'; action = 'Action'; cheer = 'Cheer'; cast_self = 'CastSelf'; casting = 'Casting'; laugh_evil = 'Wizard01Laugh'; startle = 'Wizard01Startle'; dying = 'Wizard01Dying' }
}
# The original's 14 characters: Wizard01-04, the Animal pack (WizardAnimal01-04, ids 5-8 here)
# and the Indie pack (WizardIndie01-06, ids 9-14).
$prefabName = @{}
foreach ($w in 1..4) { $prefabName[$w] = "Wizard0$w" }
foreach ($w in 5..8) { $prefabName[$w] = "WizardAnimal0$($w - 4)" }
foreach ($w in 9..14) { $prefabName[$w] = "WizardIndie0$($w - 8)" }
foreach ($w in 5..14) {
  $pn = $prefabName[$w]
  $clipMap[$w] = [ordered]@{ idle = "$($pn)Idle"; action = "$($pn)Action"; cheer = "$($pn)Cheer"; cast_self = "$($pn)CastSelf"; casting = "$($pn)Casting"; laugh_evil = "$($pn)Laugh"; startle = "$($pn)Startle"; dying = "$($pn)Dying" }
}
foreach ($w in 2..4) {
  $clipMap[$w] = [ordered]@{ idle = "Wizard0$($w)Idle"; action = "Wizard0$($w)Action"; cheer = "Wizard0$($w)Cheer"; cast_self = "Wizard0$($w)CastSelf"; casting = "Wizard0$($w)Casting"; laugh_evil = "Wizard0$($w)Laugh"; startle = "Wizard0$($w)Startle"; dying = $(if ($w -eq 4) { 'Wizard04SDying' } else { "Wizard0$($w)Dying" }) }
}
# Voice lines: Wizard01-04 share one pool and the original picks one at random each time; this
# stages the first line of each pool.
$say = [ordered]@{ action = 'wizard_cast_3'; cheer = 'yay_02'; cast_self = 'wizard_cast_1'; casting = 'cast_02'; laugh_evil = 'laugh_evil_short01'; startle = 'sad_01'; dying = 'sad_02' }

# Per-character voice lines by kind (the original's speech table): every line of each kind is
# staged as wizard<n>_say_<kind>_<i>; the original picks one at random each time. Wizards 1-4,
# and laugh_hard for every character, use the shared pool in Resources\audio\speech.
$voiceDefault = [ordered]@{
  startle = 'sad_01', 'sad_02', 'sad_05'
  laugh = 1..8 | ForEach-Object { "laugh_happy_short0$_" }
  laugh_evil = 1, 2, 3, 4, 5, 7, 8 | ForEach-Object { "laugh_evil_short0$_" }
  laugh_hard = 'laugh_long_01', 'laugh_long_02', 'laugh_long_03'
  cheer = 'yay_02', 'yay_05', 'yay_07', 'yay_09'
  action = 'wizard_cast_1', 'wizard_cast_2', 'wizard_cast_3', 'cast_03', 'cast_04', 'cast_06', 'cast_08'
  cast_self = 'wizard_cast_1', 'wizard_cast_2', 'cast_02', 'cast_06', 'cast_08'
  cast_other = 'cast_02', 'cast_06', 'cast_08', 'wizard_cast_1', 'wizard_cast_2'
}
# Wizard number -> folder, then per kind the file prefix and line count, in the order sad
# (startle), laugh, laugh_evil, cheer, cast (cast serves action, cast_self and cast_other).
$voicePacks = @{
  5 = 'animals\platypus', 'sad_platypus_', 2, 'laugh_platypus_', 3, 'laugh_evil_platypus_', 2, 'cheer_platypus_', 5, 'cast_platypus_', 5
  6 = 'animals\toucan', 'sad_toucan_', 4, 'laugh_toucan_', 3, 'laugh_evil_toucan_', 5, 'cheer_toucan_', 4, 'cast_toucan_', 5
  7 = 'animals\tanuki', 'sad_tanuki_', 3, 'laugh_tanuki_', 3, 'laugh_evil_tanuki_', 4, 'cheer_tanuki_', 4, 'cast_tanuki_', 3
  8 = 'animals\fish', 'sad_fish_', 3, 'laugh_fish_', 2, 'laugh_evil_fish_', 3, 'cheer_fish_', 3, 'cast_fish_', 5
  9 = 'indie\lethalleague', 'sad_lethal_league_', 2, 'laugh_lethal_league_', 4, 'laugh_evil_lethal_league_', 2, 'cheer_lethal_league_', 3, 'cast_lethal_league_', 6
  10 = 'indie\awesomenauts', '225_Clunk_sad_0', 3, '224_Clunk_laugh_0', 3, '225_Clunk_laugh_evil_0', 3, '223_Clunk_cheer_0', 3, '222_Clunk_cast_0', 3
  11 = 'indie\towerfall', '221_Last_sad_0', 3, '219_Last_laugh_0', 3, '220_Last_laugh_evil_0', 3, '218_Last_cheer_0', 3, '217_Last_cast_0', 3
  12 = 'indie\armello', '231_Thane_sad_0', 3, '229_Thane_laugh_0', 3, '230_Thane_laugh_evil_0', 3, '228_Thane_cheer_0', 3, '227_Thane_cast_0', 3
  13 = 'indie\nuclearthrone', '211_Fish_sad_0', 3, '209_Fish_laugh_0', 3, '209_Fish_laugh_evil_0', 3, '208_Fish_cheer_0', 3, '207_Fish_cast_0', 3
  14 = 'indie\octodad', '216_Octo_sad_0', 3, '214_Octo_laugh_0', 3, '215_Octo_laugh_evil_0', 3, '213_Octo_cheer_0', 3, '212_Octo_cast_0', 3
}
$speech = Join-Path $A 'Resources\audio\speech'
New-Item -ItemType Directory -Force (Join-Path $X 'audio\voice') | Out-Null

foreach ($n in 1..14) {
  $prefab = Join-Path $A "Resources\game\wizards\$($prefabName[$n]).prefab"
  $rows = & (Join-Path $PSScriptRoot 'prefab_tree.ps1') -Prefab $prefab -GuidIndex $G | Where-Object { $_ -match '^WizardObject/' }
  $parts = @()
  foreach ($row in $rows) {
    $c = $row -split "`t"
    $parts += [pscustomobject]@{ name = (Part-Name $c[0]); sprite = $c[1]; x = [double]::Parse($c[2], $inv); y = [double]::Parse($c[3], $inv); order = [int]$c[6] }
  }
  $parts = $parts | Sort-Object order
  $entries.Add("    `"wizard$n`": (parts: [$((($parts | ForEach-Object { '"' + $_.name + '"' }) -join ', '))]),")
  foreach ($p in $parts) { Stage-Image $n $p.sprite "wizard$($n)_$($p.name)" @($p.x, $p.y) | Out-Null }

  foreach ($clip in $clipMap[$n].Keys) {
    $c = Read-Clip $clipMap[$n][$clip]
    if (-not $c) { continue }
    foreach ($part in $c.tracks.Keys) {
      if (-not ($parts | Where-Object name -eq $part)) { continue }
      $frames = foreach ($k in $c.tracks[$part]) {
        $short = Snake (($k[1] -replace '^(gph_|anim_)', '') -replace '(?i)^wizard(animal|indie)?0\d', '')
        $key = Stage-Image $n $k[1] "wizard$($n)_img_$short"
        [string]::Format($inv, '({0:0.0###}, "{1}")', $k[0], $key)
      }
      $loop = if ($clip -eq 'idle') { 'true' } else { 'false' }
      $len = [Math]::Max($c.length, $c.tracks[$part][-1][0] + 0.0333)
      $entries.Add("    `"wizard$($n)_$($part)_$clip`": (frames: [$($frames -join ', ')], length: $(F $len), looping: $loop),")
    }
  }
  # The cape waves on its own animator.
  $cape = Read-Clip "$($prefabName[$n])CapeWave"
  if ($cape -and $cape.tracks.Count -gt 0 -and ($parts | Where-Object name -eq 'cape')) {
    $tr = $cape.tracks.Values | Select-Object -First 1
    $frames = foreach ($k in $tr) { $key = Stage-Image $n $k[1] ("wizard$($n)_img_" + (Snake ($k[1] -replace '^(gph_|anim_)', ''))); [string]::Format($inv, '({0:0.0###}, "{1}")', $k[0], $key) }
    $entries.Add("    `"wizard$($n)_cape_idle`": (frames: [$($frames -join ', ')], length: $(F $cape.length), looping: true),")
  }
  foreach ($clip in $say.Keys) {
    $src = Get-ChildItem (Join-Path $A 'AudioClip') -Recurse -Filter "$($say[$clip]).ogg" -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $src) { $src = Get-ChildItem $A -Recurse -Filter "$($say[$clip]).ogg" | Select-Object -First 1 }
    if (-not $src) { continue }
    $rel = "audio\voice\$($src.Name)"
    New-Item -ItemType Directory -Force (Join-Path $X 'audio\voice') | Out-Null
    Copy-Item $src.FullName (Join-Path $X $rel) -Force
    $entries.Add("    `"wizard$($n)_say_$clip`": (file: `"$($rel -replace '\\','/')`"),")
  }
  $lines = [ordered]@{}
  foreach ($kind in $voiceDefault.Keys) { $lines[$kind] = @($voiceDefault[$kind] | ForEach-Object { Join-Path $speech "$_.ogg" }) }
  if ($voicePacks.Contains($n)) {
    $v = $voicePacks[$n]
    $files = { param($i) @(1..$v[$i + 1] | ForEach-Object { Join-Path $speech "$($v[0])\$($v[$i])$_.ogg" }) }
    $lines['startle'] = & $files 1
    $lines['laugh'] = & $files 3
    $lines['laugh_evil'] = & $files 5
    $lines['cheer'] = & $files 7
    foreach ($kind in 'action', 'cast_self', 'cast_other') { $lines[$kind] = & $files 9 }
  }
  foreach ($kind in $lines.Keys) {
    $i = 0
    foreach ($src in $lines[$kind]) {
      $i++
      if (-not (Test-Path $src)) { Write-Warning "missing voice line $src"; continue }
      $rel = 'audio\voice\' + (Split-Path (Split-Path $src) -Leaf) + '_' + (Split-Path $src -Leaf)
      Copy-Item $src (Join-Path $X $rel) -Force
      $entries.Add("    `"wizard$($n)_say_$($kind)_$i`": (file: `"$($rel -replace '\\','/')`"),")
    }
  }
}
$texCache.Values | ForEach-Object { $_.Dispose() }

$m = [IO.File]::ReadAllLines((Join-Path $X 'manifest.ron'))
$last = [Array]::LastIndexOf($m, '}')
$merged = @($m[0..($last - 1)]) + @('    // ---- wizards, staged by wizards.ps1 ----') + $entries + @('}')
[IO.File]::WriteAllLines((Join-Path $X 'manifest.ron'), [string[]]$merged, $utf8)
[IO.File]::AppendAllLines((Join-Path $X 'staged.tsv'), [string[]]$stagedRows, $utf8)
"wizards: $($entries.Count) entries"
