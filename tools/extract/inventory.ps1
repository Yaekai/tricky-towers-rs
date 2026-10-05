param([string]$Repo)
# Writes extracted\inventory.md (every sprite, texture, font and sound in the original, grouped by
# purpose) and stages everything not yet staged under a suggested logical name.
# Run after stage.ps1 (which writes extracted\manifest.ron and extracted\staged.tsv). Local only.
Add-Type -AssemblyName System.Drawing
$ErrorActionPreference = 'Stop'
$A = Join-Path $Repo 'research\ripped\ExportedProject\Assets'
$X = Join-Path $Repo 'extracted'
$utf8 = New-Object System.Text.UTF8Encoding($false)
$inv = [Globalization.CultureInfo]::InvariantCulture
$idx = @{}
Get-Content (Join-Path $Repo 'research\guid_index.tsv') | ForEach-Object { $p = $_ -split "`t"; $idx[$p[0]] = $p[1] }

# What stage.ps1 already staged: "kind`tsource`tkey"
$staged = @{}
Get-Content (Join-Path $X 'staged.tsv') | ForEach-Object { $p = $_ -split "`t"; $staged["$($p[0]):$($p[1])"] = $p[2] }
$usedKeys = @{}
Select-String -Path (Join-Path $X 'manifest.ron') -Pattern '^\s*"([^"]+)"' | ForEach-Object { $usedKeys[$_.Matches[0].Groups[1].Value] = 1 }

function Snake($s) {
  $s = $s -replace '^(gph_|icn_|anim_)', ''
  $s = [regex]::Replace($s, '([a-z0-9])([A-Z])', '$1_$2')
  $s = $s.ToLowerInvariant() -replace '[^a-z0-9]+', '_'
  return $s.Trim('_')
}
function Unique($k) {
  $base = $k; $n = 2
  while ($usedKeys.ContainsKey($k)) { $k = "$($base)_$n"; $n++ }
  $usedKeys[$k] = 1; return $k
}

function Category($name) {
  $n = $name.ToLowerInvariant()
  if ($n -match 'sdf atlas') { return 'fonts' }
  if ($n -match 'mainmenu|menu|playerselect|difficulty|modetitle|popup|btn|button|logo|leaderboard|settings|keyboard|kbkey|dpad|analog|quickmatch|lockedchallenge|^gph_cup|cupmode|rankbar|rankpanel|rankglyph|onlinescore|demomessage|gameinfo|personalbest|keybind|controller|psx|xbox|steam|dlc|lobby|gradienttexturefont|modaltile|language|unavailble|hostdropped|icn_time') { return 'menus' }
  if ($n -match 'skull|ghost') { return 'effects' }
  if ($n -match 'wizard|cape') { return 'wizards' }
  if ($n -match 'nudgesign|heart|^icn_spell|spellbackground|icn_lightspell|icn_darkspell|nextbrick|hud|health|count') { return 'hud' }
  if ($n -match 'roof|confetti|firework|trophy|winner|loser|spotlight|crown|finishflag|medal|^gph_star') { return 'match' }
  if ($n -match 'brick|mystery|towerpiece') { return 'bricks' }
  if ($n -match 'ivy|petrif|ice|grass|moss|bubble|balloon|mist|laser|spell|cast|lightning|orb|guide|gravel|debris|impact|trail|beam|eclipse|rotor|chain|lock|norotate|sunray|whitecircle|gradientradial|gradientreflected|noise') { return 'effects' }
  if ($n -match 'background|cloud|island|house|sky|moon|sand|water|wave|sun|snow|rain|foreground|finishline|pace|score|ruin|desert|gradient|pillar|stars|reflection|galaxy|floor|cactus|checkpoint|corona') { return 'backgrounds' }
  if ($n -match 'particle|splash|dust|poof|spark|^anim_') { return 'particles' }
  return 'other'
}
function Mode($name) {
  $n = $name.ToLowerInvariant()
  if ($n -match 'world02|lws|island0|moon|starry|spooky|survival') { return 'survival' }
  if ($n -match 'world03|desert|sand|ruin|laser|pillar|puzzle|middlecenter|middleleft|middleright|backleft|backright|frontleft|frontright') { return 'puzzle' }
  return 'race'
}
$prefix = @{ menus = 'menu'; wizards = 'wizard'; hud = 'hud'; match = 'match'; bricks = 'brick'; effects = 'fx'; backgrounds = 'bg'; particles = 'particle'; fonts = 'font'; other = 'misc' }
function Suggest($name, $cat) {
  $s = Snake $name
  if ($cat -eq 'backgrounds') { return "bg_$(Mode $name)_$s" }
  $p = $prefix[$cat]
  if ($s.StartsWith($p)) { return $s }
  return "$($p)_$s"
}

# ---------------- Sprites ----------------
$texCache = @{}
function Get-Tex($path) { if (-not $texCache.ContainsKey($path)) { $texCache[$path] = [System.Drawing.Bitmap]::FromFile($path) }; return $texCache[$path] }
$num = '(-?[0-9.]+(?:[eE][-+]?\d+)?)'
$sprites = @{}
$spriteByGuid = @{}
foreach ($f in Get-ChildItem (Join-Path $A 'Sprite') -Filter *.asset) {
  $t = [IO.File]::ReadAllText($f.FullName)
  $r = [regex]::Match($t, "textureRect:\s*\n\s*serializedVersion: 2\s*\n\s*x: $num\s*\n\s*y: $num\s*\n\s*width: $num\s*\n\s*height: $num")
  $pv = [regex]::Match($t, "m_Pivot: \{x: $num, y: $num\}")
  $g = [regex]::Match($t, 'm_RD:\s*\n\s*texture: \{fileID: \d+, guid: ([0-9a-f]+)').Groups[1].Value
  $meta = Select-String -Path "$($f.FullName).meta" -Pattern '^guid: ([0-9a-f]+)' -List
  $s = [pscustomobject]@{
    name = $f.BaseName; x = [double]$r.Groups[1].Value; y = [double]$r.Groups[2].Value
    w = [double]$r.Groups[3].Value; h = [double]$r.Groups[4].Value
    ppu = [double][regex]::Match($t, "m_PixelsToUnits: $num").Groups[1].Value
    px = [double]$pv.Groups[1].Value; py = [double]$pv.Groups[2].Value
    tex = $idx[$g]; key = $null; cat = $null
  }
  $s.cat = Category $s.name
  $sprites[$s.name] = $s
  if ($meta) { $spriteByGuid[$meta.Matches[0].Groups[1].Value] = $s }
}
$texUsedBySprites = @{}
foreach ($s in $sprites.Values) { if ($s.tex) { $texUsedBySprites[[IO.Path]::GetFileNameWithoutExtension($s.tex)] = 1 } }

$newManifest = New-Object System.Collections.Generic.List[string]
foreach ($s in ($sprites.Values | Sort-Object name)) {
  if ($staged.ContainsKey("sprite:$($s.name)")) { $s.key = $staged["sprite:$($s.name)"]; continue }
  if (-not $s.tex -or -not (Test-Path $s.tex) -or $s.w -lt 1 -or $s.h -lt 1) { continue }
  $key = Unique (Suggest $s.name $s.cat)
  $tex = Get-Tex $s.tex
  $top = [int]($tex.Height - $s.y - $s.h)
  $w = [Math]::Max(1, [int][Math]::Round($s.w)); $h = [Math]::Max(1, [int][Math]::Round($s.h))
  if ([int]$s.x + $w -gt $tex.Width) { $w = $tex.Width - [int]$s.x }
  if ($top + $h -gt $tex.Height) { $h = $tex.Height - $top }
  if ($top -lt 0 -or $w -lt 1 -or $h -lt 1) { continue }
  $rect = New-Object System.Drawing.Rectangle ([int]$s.x), $top, $w, $h
  $crop = $tex.Clone($rect, $tex.PixelFormat)
  $rel = "all\$($s.cat)\$key.png"
  $out = Join-Path $X $rel
  New-Item -ItemType Directory -Force (Split-Path $out) | Out-Null
  $crop.Save($out, [System.Drawing.Imaging.ImageFormat]::Png); $crop.Dispose()
  $newManifest.Add([string]::Format($inv, '    "{0}": (file: "{1}", pixels_per_unit: {2:0.0###}, pivot: ({3:0.0###}, {4:0.0###})),', $key, ($rel -replace '\\','/'), $s.ppu, $s.px, $s.py))
  $s.key = $key
}

# ---------------- Textures (whole files not cut into sprites, plus fonts) ----------------
$textures = @()
foreach ($f in Get-ChildItem (Join-Path $A 'Texture2D') -Filter *.png) {
  $img = Get-Tex $f.FullName
  $t = [pscustomobject]@{ name = $f.BaseName; w = $img.Width; h = $img.Height; key = $null; cat = (Category $f.BaseName); atlas = $texUsedBySprites.ContainsKey($f.BaseName) }
  if ($staged.ContainsKey("texture:$($f.BaseName)")) { $t.key = $staged["texture:$($f.BaseName)"] }
  elseif (-not $t.atlas) {
    $key = Unique (Suggest $f.BaseName $t.cat)
    if ($t.cat -eq 'fonts') { $key = Unique ('font_' + (Snake ($f.BaseName -replace ' SDF Atlas', ''))) }
    $rel = "all\textures\$key.png"
    New-Item -ItemType Directory -Force (Join-Path $X 'all\textures') | Out-Null
    Copy-Item $f.FullName (Join-Path $X $rel) -Force
    $newManifest.Add("    `"$key`": (file: `"$($rel -replace '\\','/')`"),")
    $t.key = $key
  }
  $textures += $t
}
$texCache.Values | ForEach-Object { $_.Dispose() }

# ---------------- Audio ----------------
$audio = @()
foreach ($f in Get-ChildItem $A -Recurse -Include *.ogg,*.wav | Sort-Object FullName) {
  $dir = $f.Directory.Name.ToLowerInvariant()
  $cat = if ($dir -eq 'music') { 'music' } elseif ($dir -eq 'sfx') {
      $n = $f.BaseName.ToLowerInvariant()
      if ($n -match 'ambience|thunder') { 'ambience' } elseif ($n -match 'menu|select_item|confirm|back|xp_|count_up|exp_gained|medal|coin|trophy|4th_place|gong|fireworks|score') { 'menu and results' } else { 'gameplay sfx' }
    } elseif ($dir -eq 'speech') { 'wizard speech' } else { "voice pack: $dir" }
  $au = [pscustomobject]@{ name = $f.BaseName; dir = $dir; cat = $cat; kb = [int]($f.Length / 1024); key = $null }
  if ($staged.ContainsKey("audio:$($f.BaseName)")) { $au.key = $staged["audio:$($f.BaseName)"] }
  else {
    $p = if ($cat -eq 'music') { 'music' } elseif ($cat -like 'voice*' -or $cat -eq 'wizard speech') { "sfx_voice_$dir" } elseif ($cat -eq 'ambience') { 'sfx_amb' } else { 'sfx' }
    $base = Snake ($f.BaseName -replace '^\d+_', '')
    if ($p -like 'sfx_voice_*' -and $base.StartsWith($dir)) { $base = $base.Substring($dir.Length).Trim('_') }
    $key = Unique "$($p)_$base"
    $rel = "audio\all\$dir\$($f.Name)"
    New-Item -ItemType Directory -Force (Join-Path $X "audio\all\$dir") | Out-Null
    Copy-Item $f.FullName (Join-Path $X $rel) -Force
    $newManifest.Add("    `"$key`": (file: `"$($rel -replace '\\','/')`"),")
    $au.key = $key
  }
  $audio += $au
}

# ---------------- Animations ----------------
$anims = @()
foreach ($f in Get-ChildItem (Join-Path $A 'AnimationClip') -Filter *.anim) {
  $t = [IO.File]::ReadAllText($f.FullName)
  $rate = [regex]::Match($t, 'm_SampleRate: ([\d.]+)').Groups[1].Value
  $pp = $t.IndexOf('m_PPtrCurves:'); if ($pp -lt 0) { continue }
  $sect = $t.Substring($pp)
  $end = $sect.IndexOf("`n  m_SampleRate"); if ($end -gt 0) { $sect = $sect.Substring(0, $end) }
  $tracks = [regex]::Split($sect, '\n  - curve:') | Select-Object -Skip 1
  $lines = @()
  foreach ($tr in $tracks) {
    $keys = [regex]::Matches($tr, 'time: ([\d.eE-]+)\s*\n\s*value: \{fileID: \d+, guid: ([0-9a-f]+)')
    if ($keys.Count -eq 0) { continue }
    $path = [regex]::Match($tr, 'path: (.*)').Groups[1].Value.Trim()
    $frames = @(); $times = @()
    foreach ($k in $keys) {
      $sp = $spriteByGuid[$k.Groups[2].Value]
      $label = if ($sp) { if ($sp.key) { "$($sp.name) ($($sp.key))" } else { $sp.name } } else { '?' }
      $tm = [double]::Parse($k.Groups[1].Value, $inv); $times += $tm
      $frames += [string]::Format($inv, '{0} @ {1:0.###}s', $label, $tm)
    }
    $fps = '-'
    if ($times.Count -gt 1) {
      $gaps = @(); for ($i = 1; $i -lt $times.Count; $i++) { $gaps += ($times[$i] - $times[$i-1]) }
      $med = ($gaps | Sort-Object)[[int]($gaps.Count / 2)]
      if ($med -gt 0) { $fps = [string]::Format($inv, '{0:0.#}', 1 / $med) }
    }
    $lines += "  - track ``$path``, $($keys.Count) frames, ~$fps fps: " + ($frames -join ', ')
  }
  if ($lines.Count -gt 0) { $anims += [pscustomobject]@{ name = $f.BaseName; rate = $rate; lines = $lines } }
}
# Sprite-sheet style animations (anim_<name>_<WxH>_<n>) grouped by name, frames in order.
$sheets = $sprites.Values | Where-Object { $_.name -match '^anim_(.+?)_(\d+)$' } | Group-Object { $_.name -replace '_\d+$', '' }

# ---------------- Write inventory.md ----------------
$o = New-Object System.Collections.Generic.List[string]
$o.Add('# Art and sound inventory (generated locally from your own install, do not commit)')
$o.Add('')
$o.Add("Every sprite, texture, font and sound found in the original, grouped by what it is for. Each line: source name, size, and the logical name it is staged under in manifest.ron. Everything is now staged; names under ``all/`` are suggestions following the art-layout patterns. Pixels per unit: bricks 65, world/backgrounds/HUD 26.")
$o.Add('')
$o.Add("Totals: $($sprites.Count) sprites, $($textures.Count) textures ($(($textures | Where-Object atlas).Count) of them are sprite atlases), $($audio.Count) sounds, $($anims.Count) animation clips with sprite frames, $(@($sheets).Count) sprite-sheet animations.")
$titles = [ordered]@{ menus = 'Menus and title'; hud = 'HUD (player panels, hearts, spell slots, icons)'; wizards = 'Wizards (body parts, faces, capes, poses)'; bricks = 'Bricks (all packs, stone/ice versions, Mystery, minibase pieces)'; effects = 'Spell effects (ivy, petrify, ice, bubble, balloon, ghosts, mist, laser, casting)'; backgrounds = 'Backgrounds (per mode)'; match = 'Match moments (countdown, win, lose, trophies, spotlights)'; particles = 'Particles and sprite-sheet effects'; other = 'Other' }
foreach ($c in $titles.Keys) {
  $o.Add(''); $o.Add("## $($titles[$c])")
  $list = $sprites.Values | Where-Object { $_.cat -eq $c } | Sort-Object name
  if ($c -eq 'backgrounds') {
    foreach ($m in 'race','survival','puzzle') {
      $o.Add(''); $o.Add("### $m")
      foreach ($s in ($list | Where-Object { (Mode $_.name) -eq $m })) { $o.Add([string]::Format($inv, '- {0}: {1:0}x{2:0} px -> {3}', $s.name, $s.w, $s.h, $(if ($s.key) { $s.key } else { '-' }))) }
    }
  } else {
    foreach ($s in $list) { $o.Add([string]::Format($inv, '- {0}: {1:0}x{2:0} px -> {3}', $s.name, $s.w, $s.h, $(if ($s.key) { $s.key } else { '-' }))) }
  }
}
$o.Add(''); $o.Add('## Whole textures (brick packs, normal maps, gradients, noise; atlases are listed for reference)')
foreach ($t in ($textures | Where-Object { $_.cat -ne 'fonts' } | Sort-Object name)) {
  $k = if ($t.key) { $t.key } elseif ($t.atlas) { '(atlas: its sprites are listed above)' } else { '-' }
  $o.Add("- $($t.name): $($t.w)x$($t.h) px -> $k")
}
$o.Add(''); $o.Add('## Fonts (TextMeshPro signed-distance-field atlases)')
foreach ($t in ($textures | Where-Object { $_.cat -eq 'fonts' } | Sort-Object name)) { $o.Add("- $($t.name): $($t.w)x$($t.h) px -> $($t.key)") }
$o.Add(''); $o.Add('## Animations (Unity clips that swap sprites; frames in order with their time)')
foreach ($an in ($anims | Sort-Object name)) { $o.Add("- **$($an.name)** (clip sample rate $($an.rate))"); $an.lines | ForEach-Object { $o.Add($_) } }
$o.Add(''); $o.Add('## Sprite-sheet animations (frames in order; play rate set by their clip or particle system)')
foreach ($g in ($sheets | Sort-Object Name)) {
  $fr = $g.Group | Sort-Object { [int]($_.name -replace '^.*_', '') } | ForEach-Object { $_.key }
  $o.Add("- $($g.Name) ($($g.Count) frames): " + ($fr -join ', '))
}
$o.Add(''); $o.Add('## Audio')
foreach ($grp in ($audio | Group-Object cat | Sort-Object Name)) {
  $o.Add(''); $o.Add("### $($grp.Name)")
  foreach ($au in $grp.Group) { $o.Add("- $($au.name).ogg ($($au.kb) KB) -> $($au.key)") }
}
[IO.File]::WriteAllLines((Join-Path $X 'inventory.md'), [string[]]$o, $utf8)

# Append the new entries to manifest.ron (before the closing brace).
$m = [IO.File]::ReadAllLines((Join-Path $X 'manifest.ron'))
$last = [Array]::LastIndexOf($m, '}')
$merged = @($m[0..($last - 1)]) + @('    // ---- everything else, staged by inventory.ps1 under suggested names ----') + $newManifest + @('}')
[IO.File]::WriteAllLines((Join-Path $X 'manifest.ron'), [string[]]$merged, $utf8)
"inventory lines: $($o.Count); newly staged: $($newManifest.Count); manifest entries: $($usedKeys.Count)"
