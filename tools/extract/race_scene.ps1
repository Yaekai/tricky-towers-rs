param([string]$Repo)
# Stages the pieces of the original's live Race scene that stage.ps1 doesn't (run it after
# stage.ps1): the ten small and two large clouds of its cloud layers (RACE_CLOUD1..10,
# RACE_LARGE_CLOUD1..2), the islands' waterline, the reflected band over the water
# (ISLAND_GRADIENT) and the wave crest (WAVE). The game places and moves them
# (GameConfig::race_scene). Also the noise a zapped brick dissolves by (BrickLaser's
# _NoiseTex). Local only: extracted\ is gitignored.
Add-Type -AssemblyName System.Drawing
$ErrorActionPreference = 'Stop'
$A = Join-Path $Repo 'research\ripped\ExportedProject\Assets'
$X = Join-Path $Repo 'extracted'
$idx = @{}
Get-Content (Join-Path $Repo 'research\guid_index.tsv') | ForEach-Object { $p = $_ -split "`t"; $idx[$p[0]] = $p[1] }
$utf8 = New-Object System.Text.UTF8Encoding($false)
function Inv([double]$v) { [string]::Format([Globalization.CultureInfo]::InvariantCulture, '{0:0.0###}', $v) }

$add = New-Object System.Collections.Generic.List[string]
$texCache = @{}
function Add-Piece($key, $sprite) {
  $f = Join-Path $A "Sprite\$sprite.asset"
  if (-not (Test-Path $f)) { "missing: $key ($sprite)"; return }
  $t = Get-Content $f -Raw
  $num = '(-?[0-9.]+(?:[eE][-+]?\d+)?)'
  $r = [regex]::Match($t, "textureRect:\s*\n\s*serializedVersion: 2\s*\n\s*x: $num\s*\n\s*y: $num\s*\n\s*width: $num\s*\n\s*height: $num")
  $ppu = [double][regex]::Match($t, "m_PixelsToUnits: $num").Groups[1].Value
  $pv = [regex]::Match($t, "m_Pivot: \{x: $num, y: $num\}")
  $g = [regex]::Match($t, 'm_RD:\s*\n\s*texture: \{fileID: \d+, guid: ([0-9a-f]+)').Groups[1].Value
  $texPath = $idx[$g]
  if (-not $texPath -or -not (Test-Path $texPath)) { "missing texture: $key ($sprite)"; return }
  if (-not $texCache.ContainsKey($texPath)) { $texCache[$texPath] = [System.Drawing.Bitmap]::FromFile($texPath) }
  $tex = $texCache[$texPath]
  $rx = [double]$r.Groups[1].Value; $ry = [double]$r.Groups[2].Value
  $rw = [double]$r.Groups[3].Value; $rh = [double]$r.Groups[4].Value
  $top = [int]($tex.Height - $ry - $rh)
  $rect = New-Object System.Drawing.Rectangle ([int]$rx), $top, ([int]$rw), ([int]$rh)
  $crop = $tex.Clone($rect, $tex.PixelFormat)
  $rel = "backgrounds\scene_$key.png"
  $out = Join-Path $X $rel
  New-Item -ItemType Directory -Force (Split-Path $out) | Out-Null
  $crop.Save($out, [System.Drawing.Imaging.ImageFormat]::Png); $crop.Dispose()
  $add.Add("    `"$key`": (file: `"$($rel -replace '\\','/')`", pixels_per_unit: $(Inv $ppu), pivot: ($(Inv ([double]$pv.Groups[1].Value)), $(Inv ([double]$pv.Groups[2].Value)))),")
}

$small = @('gph_cloud_var01_0','gph_cloud_var02_0','gph_cloud_var03','gph_cloud_var04_0','gph_cloud_var05','gph_cloud_var06_0','gph_cloud_var07_0','gph_cloud_var08_0','gph_cloud_var09','gph_cloud_var10_0')
for ($i = 0; $i -lt $small.Count; $i++) { Add-Piece "race_cloud_$($i + 1)" $small[$i] }
Add-Piece 'race_cloud_large_1' 'gph_cloudLarge_var01_0'
Add-Piece 'race_cloud_large_2' 'gph_cloudLarge_var02'
Add-Piece 'race_waterline' 'gph_waterline04'
Add-Piece 'race_island_gradient' 'gph_gradientRefelcted'
Add-Piece 'race_wave' 'gph_wave'
# Endless Race's checkpoint flag and its pole (SpellContainerLineCheckpointFlag).
Add-Piece 'checkpoint_flag' 'gph_checkpointFlag'
Add-Piece 'checkpoint_flag_pole' 'gph_checkpointFlagPole'

# Whole textures: the zap noise, and Endless Race's perspective cloud planes with their two
# scrolling masks (CloudRaceEndless01/02).
function Add-Whole($key, $texture) {
  $src = Join-Path $A "Texture2D\$texture.png"
  if (-not (Test-Path $src)) { "missing: $key ($texture)"; return }
  Copy-Item $src (Join-Path $X "backgrounds\scene_$key.png") -Force
  $add.Add("    `"$key`": (file: `"backgrounds/scene_$key.png`"),")
}
Add-Whole 'brick_zap_noise' 'gph_noise'
Add-Whole 'endless_cloud_1' 'gph_cloudLargeRaceEndless_01'
Add-Whole 'endless_cloud_2' 'gph_cloudLargeRaceEndless_02'
Add-Whole 'endless_cloud_mask_1' 'gph_cloudLargeRaceEndless_mask01'
Add-Whole 'endless_cloud_mask_2' 'gph_cloudLargeRaceEndless_mask02'

# The cup winner popup (TournamentWinnerPopup): the name and trophy backs, the fourth-place
# medal, and its sounds (SFX_DRUM_ROLL, TROPHY).
Add-Piece 'cup_winner_name_back' 'gph_cupModeWinnerPopupNameBackground'
Add-Piece 'cup_winner_trophy_back' 'gph_cupModeWinnerPopupTrophyBackground'
Add-Piece 'menu_leaderboard_medal_4' 'gph_onlineScoreMedalLast'
function Add-Sound($key, $clip) {
  $src = Get-ChildItem $A -Recurse -Filter "$clip.ogg" | Select-Object -First 1
  if (-not $src) { "missing: $key ($clip)"; return }
  New-Item -ItemType Directory -Force (Join-Path $X 'audio') | Out-Null
  Copy-Item $src.FullName (Join-Path $X "audio\$($src.Name)") -Force
  $add.Add("    `"$key`": (file: `"audio/$($src.Name)`"),")
}
Add-Sound 'sfx_drum_roll' '017_Drum_roll'
Add-Sound 'sfx_trophy' '103_Trophy'
# Its fireworks (Firework, FireWorksRed/Blue): the rocket's whistle, the bangs and the
# exploding stars' blob (StarExplosionShineyParticle, two frames side by side).
foreach ($i in 1..4) { Add-Sound "sfx_brick_fall_$i" "058_Falling_brick_0$i" }
foreach ($i in 1..3) { Add-Sound "sfx_firework_$i" "104_Fireworks_0$i" }
# The winner's wizard pops in and out with SFX_WIZARD_LOSE.
Add-Sound 'sfx_wizard_lose' '089_Wizard_lose_v2'
# Race Special's thunder (LIGHTNING_STRIKE_01..03) and Endless Puzzle's laser moving up a
# level (MOVE_LASER_1..3).
foreach ($i in 1..3) { Add-Sound "sfx_thunder_$i" "099_Ambience_thunderstrikes_0$i" }
foreach ($i in 1..3) { Add-Sound "sfx_laser_move_$i" "106_Laser_move_0$i" }
Add-Whole 'firework_blob' 'gph_castEffect3Blob04'
# Endless Puzzle's sun (DesertEndlessSun's Eclipse material): its colour and alpha by how
# much of the sun shows.
Add-Whole 'eclipse_gradient' 'gph_eclipse_gradient'

$path = Join-Path $X 'manifest.ron'
$keys = @{}; foreach ($l in $add) { if ($l -match '^\s*"([^"]+)"') { $keys[$Matches[1]] = 1 } }
$m = @([IO.File]::ReadAllLines($path) | Where-Object { -not ($_ -match '^\s*"([^"]+)"' -and $keys.ContainsKey($Matches[1])) -and $_ -notmatch 'staged by race_scene.ps1' })
$last = [Array]::LastIndexOf($m, '}')
$merged = @($m[0..($last - 1)]) + @('    // ---- the live Race scene, staged by race_scene.ps1 ----') + $add + @('}')
[IO.File]::WriteAllLines($path, [string[]]$merged, $utf8)
"race_scene: $($add.Count) entries"
