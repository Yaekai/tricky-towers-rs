param([string]$Repo)
# Stages the original's art/sound from research\ripped into extracted\ per docs/art-layout.md.
# Local only: extracted\ is gitignored.
Add-Type -AssemblyName System.Drawing
$ErrorActionPreference = 'Stop'
$A = Join-Path $Repo 'research\ripped\ExportedProject\Assets'
$X = Join-Path $Repo 'extracted'
$idx = @{}
Get-Content (Join-Path $Repo 'research\guid_index.tsv') | ForEach-Object { $p = $_ -split "`t"; $idx[$p[0]] = $p[1] }
New-Item -ItemType Directory -Force $X | Out-Null

$manifest = New-Object System.Collections.Generic.List[string]
$notes = New-Object System.Collections.Generic.List[string]
$missing = New-Object System.Collections.Generic.List[string]
$texCache = @{}
$stagedMap = New-Object System.Collections.Generic.List[string]

function Get-Tex($path) {
  if (-not $texCache.ContainsKey($path)) { $texCache[$path] = [System.Drawing.Bitmap]::FromFile($path) }
  return $texCache[$path]
}

function Read-Sprite($name) {
  $f = Join-Path $A "Sprite\$name.asset"
  if (-not (Test-Path $f)) { return $null }
  $t = Get-Content $f -Raw
  $num = '(-?[0-9.]+(?:[eE][-+]?\d+)?)'
  $r = [regex]::Match($t, "textureRect:\s*\n\s*serializedVersion: 2\s*\n\s*x: $num\s*\n\s*y: $num\s*\n\s*width: $num\s*\n\s*height: $num")
  $ppu = [regex]::Match($t, "m_PixelsToUnits: $num").Groups[1].Value
  $pv = [regex]::Match($t, "m_Pivot: \{x: $num, y: $num\}")
  $g = [regex]::Match($t, 'm_RD:\s*\n\s*texture: \{fileID: \d+, guid: ([0-9a-f]+)').Groups[1].Value
  [pscustomobject]@{
    x = [double]$r.Groups[1].Value; y = [double]$r.Groups[2].Value
    w = [double]$r.Groups[3].Value; h = [double]$r.Groups[4].Value
    ppu = [double]$ppu; px = [double]$pv.Groups[1].Value; py = [double]$pv.Groups[2].Value
    tex = $idx[$g]
  }
}

function Inv([double]$v) { [string]::Format([Globalization.CultureInfo]::InvariantCulture, '{0:0.0###}', $v) }
$utf8 = New-Object System.Text.UTF8Encoding($false)

# Image from a Sprite asset (cut out of its atlas). $pivot overrides the sprite's own pivot.
function Add-Sprite($key, $sprite, $rel, $what, $pivot = $null, $extra = '') {
  $s = Read-Sprite $sprite
  if ($null -eq $s -or -not $s.tex -or -not (Test-Path $s.tex)) { $missing.Add("$key (sprite $sprite)"); return }
  $tex = Get-Tex $s.tex
  $top = [int]($tex.Height - $s.y - $s.h)   # Unity rects start at the bottom
  $rect = New-Object System.Drawing.Rectangle ([int]$s.x), $top, ([int]$s.w), ([int]$s.h)
  $crop = $tex.Clone($rect, $tex.PixelFormat)
  $out = Join-Path $X $rel
  New-Item -ItemType Directory -Force (Split-Path $out) | Out-Null
  $crop.Save($out, [System.Drawing.Imaging.ImageFormat]::Png); $crop.Dispose()
  if ($null -eq $pivot) { $pivot = @($s.px, $s.py) }
  $manifest.Add("    `"$key`": (file: `"$($rel -replace '\\','/')`", pixels_per_unit: $(Inv $s.ppu), pivot: ($(Inv $pivot[0]), $(Inv $pivot[1]))$extra),")
  $notes.Add("- $key : $what. $([int]$s.w)x$([int]$s.h) px, $(Inv $s.ppu) px/unit, from sprite $sprite.")
  $stagedMap.Add("sprite`t$sprite`t$key")
}

# Image that is a whole texture file.
function Add-Texture($key, $file, $rel, $what) {
  $src = Join-Path $A "Texture2D\$file"
  if (-not (Test-Path $src)) { $missing.Add("$key (texture $file)"); return }
  $out = Join-Path $X $rel
  New-Item -ItemType Directory -Force (Split-Path $out) | Out-Null
  Copy-Item $src $out -Force
  $img = Get-Tex $src
  $manifest.Add("    `"$key`": (file: `"$($rel -replace '\\','/')`"),")
  $notes.Add("- $key : $what. $($img.Width)x$($img.Height) px, from texture $file.")
  $stagedMap.Add("texture`t$([IO.Path]::GetFileNameWithoutExtension($file))`t$key")
}

function Add-Audio($key, $clip, $what) {
  $src = Get-ChildItem $A -Recurse -Filter "$clip.ogg" | Select-Object -First 1
  if (-not $src) { $missing.Add("$key (audio $clip)"); return }
  $rel = "audio\$($src.Name)"
  New-Item -ItemType Directory -Force (Join-Path $X 'audio') | Out-Null
  Copy-Item $src.FullName (Join-Path $X $rel) -Force
  $manifest.Add("    `"$key`": (file: `"audio/$($src.Name)`"),")
  $notes.Add("- $key : $what. $clip.ogg.")
  $stagedMap.Add("audio`t$clip`t$key")
}

# --- Bricks (whole-brick textures, spawn orientation) ---
foreach ($b in 'I','O','T','S','Z','J','L') { Add-Texture "brick_$b" "gph_brick$($b)_default_texture.png" "bricks\$b.png" "Brick $b, default pack" }
Add-Sprite 'mystery_block' 'gph_brickMystery01' 'bricks\mystery_block.png' 'Mystery 1, 2x4 block'
Add-Sprite 'mystery_blob'  'gph_brickMystery02' 'bricks\mystery_blob.png'  'Mystery 2, blob in a 3x4 box (cells: 2x3 at x -1.5..0.5 y -2..1, 1x3 at x 0.5..1.5 y -1..2, 1x1 at x -0.5..0.5 y 1..2; plus a decorative rotor, gph_brickMystery02Rotor, drawn on top)'
Add-Sprite 'mystery_table' 'gph_brickMystery03' 'bricks\mystery_table.png' 'Mystery 3, table: 4x2 slab on two 1x1 feet'

# --- Platforms (pivot = middle of top edge; approximate, the sprite top is assumed to be the surface) ---
Add-Sprite 'platform_race'     'gph_house02'          'platforms\race.png'     'Race base (house standing in water); pivot = roof surface, the chimney above it is a 1x1 collider at x -1..0' @(0.5, 0.9007)
Add-Sprite 'platform_survival' 'gph_house01_world02'  'platforms\survival.png' 'Survival base (castle-like house); surface has 3 merlons: x -1..1 and x +/-2..3, top at the pivot' @(0.522, 0.998)
Add-Sprite 'platform_puzzle'   'gph_house01_world03'  'platforms\puzzle.png'   'Puzzle floor piece 01: a single 1-wide, 13-tall column; puzzle floors are built from 3 such pieces' @(0.475, 0.998)

# --- Race background, approximated per lane ---
# The original draws ONE full-screen background with its own camera (screen space: 1080p = 73.85 x
# 41.54 units at 26 px/unit). Islands sit on the water line, 6 screen units above the screen
# bottom, anchored to the screen's bottom-left / bottom-centre / bottom-right. Here each layer is
# placed per lane relative to the race water line (y = -8.5 cells), converting screen units to
# cells with 1 cell ~ 65 px (2.5 screen units): scale 0.4, half screen width ~ 14.8 cells.
function Bg($n, $sprite, $bx, $by, $bz, $par, $what, $mode = 'race') {
  $f = [Globalization.CultureInfo]::InvariantCulture
  $extra = [string]::Format($f, ', position: ({0:0.0##}, {1:0.0##}), z: {2}, parallax: {3:0.0#}, scale: 0.4', $bx, $by, $bz, $par)
  Add-Sprite "background_$($mode)_$n" $sprite "backgrounds\$($mode)_$n.png" $what $null $extra
}
$W = -8.5; $L = -14.8; $R = 14.8
# Race: screen-pinned like the original's background camera. Units = original screen units
# (1080p window = 41.54 units tall, 26 px/unit). Water band = bottom 6 units.
function ScreenBg($n, $sprite, $sx, $sy, $px, $py, $bz, $par, $what) {
  $f = [Globalization.CultureInfo]::InvariantCulture
  $extra = [string]::Format($f, ', screen: ({0:0.0##}, {1:0.0##}), position: ({2:0.0##}, {3:0.0##}), view_height: 41.54, z: {4}, parallax: {5:0.0#}', $sx, $sy, $px, $py, $bz, $par)
  Add-Sprite "background_race_$n" $sprite "backgrounds\race_$n.png" $what $null $extra
}
ScreenBg 0 'gph_cloudLarge_var01_0' 0.3 0.0 0.0 0.0 0 0.2 'Large cloud, centred on the screen bottom (original viewport y 0), drifts 0.15-0.25 u/s'
ScreenBg 1 'gph_cloudLarge_var02'   0.8 0.0 0.0 0.0 1 0.2 'Large cloud 2'
ScreenBg 2 'gph_backgroundBack01'   0.5 0.0 0.0 6.0 2 0.3 'Back island, bottom-centre on the water band top'
ScreenBg 3 'gph_backgroundMiddle01' 0.0 0.0 0.0 5.35 3 0.3 'Middle island, bottom-left corner (offset 0, -0.65)'
ScreenBg 4 'gph_backgroundMiddle02' 1.0 0.0 -9.5 5.1 4 0.3 'Middle island, bottom-right (offset -9.5, -0.9)'
ScreenBg 5 'gph_backgroundFront01'  0.0 0.0 0.0 3.5 5 0.3 'Front island, bottom-left (offset 0, -2.5)'
ScreenBg 6 'gph_backgroundFront02'  0.0 0.0 10.0 4.8 6 0.3 'Front island, bottom-left + 10 (offset 10, -1.2)'
ScreenBg 7 'gph_backgroundFront03'  1.0 0.0 0.0 3.6 7 0.3 'Front island, bottom-right (offset 0, -2.4)'
ScreenBg 8 'gph_cloud_var01_0'      0.15 0.6 0.0 0.0 8 0.6 'Small cloud (original: viewport y 0.5-2, parallax 0.6)'
ScreenBg 9 'gph_cloud_var04_0'      0.6 0.85 0.0 0.0 9 0.6 'Small cloud'
ScreenBg 10 'gph_cloud_var07_0'     0.85 1.3 0.0 0.0 10 0.8 'Small cloud (parallax 0.8)'
$manifest.Add('    "sky": (colors: [(108, 152, 223), (125, 192, 239), (179, 196, 251)]),')
$manifest.Add('    "sky_race": (colors: [(108, 152, 223), (125, 192, 239), (179, 196, 251)]),')
$manifest.Add('    "sky_survival": (colors: [(51, 30, 103), (91, 60, 175), (91, 83, 173)]),')
$manifest.Add('    "sky_puzzle": (colors: [(252, 219, 144), (244, 230, 177)]),')
$notes.Add('- sky_survival : night gradient top (51,30,103), band (91,60,175), bottom (91,83,173). sky_puzzle : desert gradient top #FCDB90 to bottom #F4E6B1 (plus 4 soft overlay gradients in the original).')
$notes.Add('- sky : Race sky gradient, top (108,152,223) to bottom (179,196,251); the middle stop is the original overlay band (125,192,239) at 65% height.')

# --- Survival: screen layers from the original's background camera (centre-anchored) ---
function ModeScreenBg($mode, $n, $sprite, $sx, $sy, $px, $py, $bz, $par, $what, $mask = '') {
  $f = [Globalization.CultureInfo]::InvariantCulture
  $extra = [string]::Format($f, ', screen: ({0:0.0##}, {1:0.0##}), position: ({2:0.0##}, {3:0.0##}), view_height: 41.54, z: {4}, parallax: {5:0.0#}', $sx, $sy, $px, $py, $bz, $par) + $mask
  Add-Sprite "background_$($mode)_$n" $sprite "backgrounds\$($mode)_$n.png" $what $null $extra
}
ModeScreenBg 'survival' 0 'gph_moon'               0.5 0.5 -14.3 9.7  0 0.1  'Moon, centre + (-14.3, 9.7); in the original it hangs on a hinge and can be knocked down'
ModeScreenBg 'survival' 1 'gph_backgroundIsland02' 0.5 0.5 -28.0 -7.6 1 0.25 'Back island left, centre + (-28, -7.6)'
ModeScreenBg 'survival' 2 'gph_backgroundIsland04' 0.5 0.5 32.5 -8.8  2 0.25 'Back island right, centre + (32.5, -8.8)'
ModeScreenBg 'survival' 3 'gph_backgroundIsland01' 0.5 0.5 -13.5 -8.8 3 0.35 'Front island left, centre + (-13.5, -8.8)'
ModeScreenBg 'survival' 4 'gph_backgroundIsland03' 0.5 0.5 21.6 -9.8  4 0.35 'Front island right, centre + (21.6, -9.8)'

# --- Puzzle: desert ruins anchored to the screen's bottom corners/centre (two-colour mask sprites) ---
ModeScreenBg 'puzzle' 0 'gph_backgroundBackLeft'     0.0 0.0 15.0 3.8   0 0.3 'Ruins back left (mask: #F4D9B0 / #F7E0BE)' ', mask: ((244, 217, 176), (247, 224, 190))'
ModeScreenBg 'puzzle' 1 'gph_backgroundBackRight'    1.0 0.0 -30.3 3.8  1 0.3 'Ruins back right (mask: #F4D9B0 / #F7E0BE)' ', mask: ((244, 217, 176), (247, 224, 190))'
ModeScreenBg 'puzzle' 2 'gph_backgroundMiddleLeft'   0.0 0.0 0.0 3.44   2 0.5 'Ruins middle left (mask: #E0AC92 / #D69883)' ', mask: ((224, 172, 146), (214, 152, 131))'
ModeScreenBg 'puzzle' 3 'gph_backgroundMiddleCenter' 0.5 0.0 0.0 3.44   3 0.5 'Ruins middle centre (mask: #E0AC92 / #D69883)' ', mask: ((224, 172, 146), (214, 152, 131))'
ModeScreenBg 'puzzle' 4 'gph_backgroundMiddleRight'  1.0 0.0 0.0 3.44   4 0.5 'Ruins middle right (mask: #E0AC92 / #D69883)' ', mask: ((224, 172, 146), (214, 152, 131))'
ModeScreenBg 'puzzle' 5 'gph_backgroundFrontLeft'    0.0 0.0 0.0 3.6    5 0.5 'Ruins front left (mask: #C47464)' ', mask: ((196, 116, 100), (196, 116, 100))'
ModeScreenBg 'puzzle' 6 'gph_backgroundFrontRight'   1.0 0.0 0.0 2.1    6 0.5 'Ruins front right (mask: #C47464)' ', mask: ((196, 116, 100), (196, 116, 100))'

# --- Race water band (original BackgroundWater: bottom 6 screen units) ---
$manifest.Add('    "water_band": (colors: [(61, 82, 201), (137, 146, 217)], position: (0.0, 6.0), view_height: 41.54, z: 1, parallax: 0.3),')
$manifest.Add('    "water_band_race": (colors: [(61, 82, 201), (137, 146, 217)], position: (0.0, 6.0), view_height: 41.54, z: 1, parallax: 0.3),')
$manifest.Add('    "water_band_survival": (colors: []),')
$manifest.Add('    "water_band_puzzle": (colors: []),')
$notes.Add('- water_band : Race background water, bottom 6 screen units, top (61,82,201) to bottom (137,146,217); islands bob on it.')

# --- Puzzle floor pieces (proposed names; the game draws plain cells until it uses them) ---
Add-Sprite 'puzzle_piece_1' 'gph_house01_world03' 'floors\puzzle_piece_1.png' 'Puzzle floor piece 1: column 1 wide, 13 tall; pivot = top-left, drawn 0.03 left and up of the column corner' @(0.0, 1.0)
Add-Sprite 'puzzle_piece_2' 'gph_house02_world03' 'floors\puzzle_piece_2.png' 'Puzzle floor piece 2: column 2 wide, 13 tall; pivot = top-left' @(0.0, 1.0)
Add-Sprite 'puzzle_piece_3' 'gph_house03_world03' 'floors\puzzle_piece_3.png' 'Puzzle floor piece 3: column 1 wide, 13 tall (other art); pivot = top-left' @(0.0, 1.0)
Add-Sprite 'puzzle_piece_base'     'gph_house06_world3' 'floors\puzzle_piece_base.png'     'Under-floor tile, one per 2 units down each column (centres at -1, -3, ... -11)'
Add-Sprite 'puzzle_piece_base_top' 'gph_house07_world3' 'floors\puzzle_piece_base_top.png' 'Top of the under-floor stack (centre 0.37 above the column top)'
Add-Sprite 'puzzle_pro_center'     'gph_house03_world03' 'floors\puzzle_pro_center.png'    'Pro floor: 1-wide, 13-tall centre column (3 of them make the 3-wide pillar)' @(0.0, 1.0)
Add-Sprite 'puzzle_pro_ledge_1'    'gph_house08_world03' 'floors\puzzle_pro_ledge_1.png'   'Pro floor side ledge, 1x1 (variant 1)'
Add-Sprite 'puzzle_pro_ledge_2'    'gph_house09_world03' 'floors\puzzle_pro_ledge_2.png'   'Pro floor side ledge, 1x1 (variant 2)'
Add-Sprite 'puzzle_pro_ledge_3'    'gph_house10_world03' 'floors\puzzle_pro_ledge_3.png'   'Pro floor side ledge, 1x1 (variant 3)'
Add-Sprite 'puzzle_pro_ledge_top'  'gph_house11_world03' 'floors\puzzle_pro_ledge_top.png' 'Pro floor topmost side ledge, 1x1'

# --- Match moments: banner text gradients, spotlights, winner sunrays ---
Add-Texture 'text_gradient_gold'   'gph_gradientTextureFont02.png' 'text\gradient_gold.png'   'Vertical gradient for the gold front layer of WINNER! / countdown numbers'
Add-Texture 'text_gradient_red'    'gph_gradientTextureFont.png'   'text\gradient_red.png'    'Vertical gradient for the red front layer of WIZARD OUT!'
Add-Texture 'text_gradient_bronze' 'gph_gradientTextureFont04.png' 'text\gradient_bronze.png' 'Vertical gradient for the bronze front layer (draw)'
$modes = [ordered]@{ race = '01'; survival = '02'; puzzle = '03' }
foreach ($md in $modes.Keys) {
  Add-Sprite "spotlight_winner_$md" "gph_spotlightWinner_World$($modes[$md])" "match\spotlight_winner_$md.png" "Winner light beam over the winning lane ($md), stretched to screen height, 1.5 wide"
  Add-Sprite "spotlight_loser_$md"  "gph_spotlightLoser_World$($modes[$md])"  "match\spotlight_loser_$md.png"  "Dark veil over losing lanes ($md), 10 wide, alpha 0.1"
}
Add-Sprite 'winner_sunrays' 'gph_castEffectSunrays' 'match\winner_sunrays.png' 'Rotating sunrays behind the WINNER! text'
# Cups: the original shows them only in cup (tournament) results, sized by cup: Tiny, Big, Super, Online.
Add-Sprite 'trophy'        'gph_cupTinyLarge'   'match\trophy.png'        'Tiny Cup trophy (the smallest cup); the original shows a cup only in cup results, not after a single match'
Add-Sprite 'trophy_big'    'gph_cupBigLarge'    'match\trophy_big.png'    'Big Cup trophy'
Add-Sprite 'trophy_super'  'gph_cupSuperLarge'  'match\trophy_super.png'  'Super Cup trophy'
Add-Sprite 'trophy_online' 'gph_cupOnlineLarge' 'match\trophy_online.png' 'Online Cup trophy'
$notes.Add('- Match text (countdown numbers, WINNER!, WIZARD OUT!, FINISH!, FAILED, WAVE): the Enzo letters in two layers. Back: dilated 0.2, outline 0.25 (37,13,40), face (158,23,64) gold/bronze or (160,24,101) red. Front: white face times text_gradient_<gold|red|bronze>. Both have a shadow 0.5 below in (227,33,61). Gold = winner, red = loser, bronze = draw.')
$notes.Add('- Spotlights: 0.5 s after the win the winner beam fades in over 1 s to alpha 151/255 (race), 130/255 (survival), 1 (puzzle), centred on the winning lane bottom, 1.5 wide and screen tall. Loser veils fade to 0.1 over 1 s. Trophy (cups): pops from scale 0.4 to 1 over 0.6 s with an elastic ease. Lose message: move in 2 s, idle 2 s, move out 1 s.')

# --- Title screen tiles (MainMenu scene, 1920x1080 reference pixels) ---
$tiles = [ordered]@{
  local    = @('LocalMP',      'gph_mainMenuLocalMPWizards',     'gph_mainMenuLocalMPWizards_deselected',     'gph_mainMenuForegroundLocalMP_selected')
  single   = @('SinglePlayer', 'gph_mainMenuSinglePlayerWizard', 'gph_mainMenuSinglePlayerWizard_deselected', 'gph_mainMenuForegroundSinglePlayer')
  online   = @('OnlineMP',     'gph_mainMenuOnlineMP',           'gph_mainMenuOnlineMP_deselected',           'gph_mainMenuForegroundOnlineMP_selected')
  shop     = @('DLC',          'gph_mainMenuDLC',                'gph_mainMenuDLC_deselected',                'gph_mainMenuForegroundDLC_selected')
  help     = @('Settings',     $null,                            $null,                                       'gph_mainMenuForegroundSettings_selected')
  settings = @('Settings',     'gph_mainMenuSettingsCogs',       'gph_mainMenuSettingsCogs',                  'gph_mainMenuForegroundSettings_selected')
  quit     = @('Quit',         $null,                            $null,                                       'gph_mainMenuForegroundQuit_selected')
}
foreach ($k in $tiles.Keys) {
  $t = $tiles[$k]
  Add-Sprite "menu_tile_$($k)_deselected" "gph_mainMenuBackground$($t[0])_deselected" "menu\tile_$($k)_deselected.png" "Title tile $k, grey background"
  Add-Sprite "menu_tile_$($k)_selected"   "gph_mainMenuBackground$($t[0])_selected"   "menu\tile_$($k)_selected.png"   "Title tile $k, coloured background"
  if ($t[1]) { Add-Sprite "menu_tile_$($k)_front"            $t[1] "menu\tile_$($k)_front.png"            "Title tile $k, art over the picked tile (wizards / cogs)" }
  if ($t[2]) { Add-Sprite "menu_tile_$($k)_front_deselected" $t[2] "menu\tile_$($k)_front_deselected.png" "Title tile $k, grey art over the tile when not picked" }
  if ($t[3]) { Add-Sprite "menu_tile_$($k)_overlay"          $t[3] "menu\tile_$($k)_overlay.png"          "Title tile $k, frame drawn on top when picked" }
}
Add-Sprite 'menu_tile_header'  'gph_mainMenuItemHeader'      'menu\tile_header.png'  'Label bar on top of a picked tile'
Add-Sprite 'menu_corner_large' 'gph_menuSelectedCornerLarge' 'menu\corner_large.png' 'Frame corner, big tiles (69x68), drawn top-left and flipped for the others'
Add-Sprite 'menu_corner_small' 'gph_menuSelectedCornerSmall' 'menu\corner_small.png' 'Frame corner, small tiles (38x34)'
foreach ($i in 1..5) { Add-Sprite "menu_sunrays_$i" "gph_mainMenuSun0$i" "menu\sunrays_$i.png" "Sun-ray layer $i behind a picked tile (556x556)" }
$notes.Add('- Title tiles (1920x1080, centres from screen centre, y up): local (-249, 225.5) 437x335; single (-246, -155.5) 435x335; online (278, 232.5) 441x335; shop (277, -62.5) 450x165; help (146, -250) 197x150; settings (396, -251) 197x150; quit (0, -406) 436x68. Front art offsets from the tile centre: local (-5, -25) 538x272, single (17, -42) 488x235, online (-21, -19) 467x283, shop (-20, -9) 483x128, settings cogs (0, 0) 181x134. Picked tile: rays 1 turn 0->180 deg and rays 2 turn 0->-90 deg every 2 s (looping); the 4 corners pulse outward and back every 1 s.')


# --- Select Players and Select Mode (PlayerSelect / ModeSelect scenes, 1920x1080 UI) ---
Add-Sprite 'menu_header' 'gph_menuHeaderBackground' 'menu\header.png' 'Title bar of Select Players / Select Mode: 800x90 hanging from the top centre'
Add-Sprite 'menu_player_select_background_deselected' 'gph_playerSelectBackground_deselected' 'menu\player_select_background_deselected.png' 'Seat panel, grey (332x283; also the 332x100 brick pack panel, sliced)'
Add-Sprite 'menu_player_select_background_selected'   'gph_playerSelectBackground_selected'   'menu\player_select_background_selected.png'   'Seat panel behind a ready wizard, under the sun rays'
Add-Sprite 'menu_player_select_outline'   'gph_playerSelectOutline'        'menu\player_select_outline.png'   'Frame over a ready seat panel (332x283)'
Add-Sprite 'menu_player_select_nameplate' 'gph_playerSelectNameBackground' 'menu\player_select_nameplate.png' 'Name plate under a seat (367x78)'
# Seats 2 and 4 have no own "ready" picture: the original reuses the title-screen wizards.
$ready = @{ 1 = 'gph_playerSelectWizard01Ready'; 2 = 'gph_mainMenuOnlineMPWizard02'; 3 = 'gph_playerSelectWizard03Ready'; 4 = 'gph_mainMenuLocalMPWizard04' }
foreach ($n in 1..4) {
  Add-Sprite "menu_player_select_wizard$($n)_sleeping" "gph_playerSelectWizard0$($n)Sleeping" "menu\player_select_wizard$($n)_sleeping.png" "Seat $n wizard, asleep until someone joins"
  Add-Sprite "menu_player_select_wizard$($n)_ready"    $ready[$n]                              "menu\player_select_wizard$($n)_ready.png"    "Seat $n wizard, awake"
}
Add-Sprite 'menu_arrow_small'      'btn_menuArrowSmall'       'menu\arrow_small.png'      'Seat arrows left/right of a panel (38x64, at x -165 / +165)'
Add-Sprite 'menu_arrow'            'btn_menuArrow'            'menu\arrow.png'            'Mode column arrow (94x55), above and below the picked tile'
Add-Sprite 'menu_arrow_deselected' 'btn_menuArrowDeselected'  'menu\arrow_deselected.png' 'Mode column arrow when the column is not active'
foreach ($m in @('Race', 'Survival', 'Puzzle', 'Random', 'SurvivalSP')) {
  Add-Sprite "menu_mode_$($m.ToLower())" "gph_menuMode$m" "menu\mode_$($m.ToLower()).png" "Select Mode: $m tile"
}
Add-Sprite 'menu_difficulty_background'         'gph_difficultyItemBackground'         'menu\difficulty_background.png'         'Select Mode: difficulty tile background (461x393)'
Add-Sprite 'menu_difficulty_background_special' 'gph_difficultyItemBackground_special' 'menu\difficulty_background_special.png' 'Select Mode: difficulty tile background for Pro/Special'
foreach ($d in @('Easy', 'Normal', 'Pro', 'Random')) {
  Add-Sprite "menu_difficulty_$($d.ToLower())" "gph_difficulty$d" "menu\difficulty_$($d.ToLower()).png" "Select Mode: $d picture on the difficulty tile"
}
Add-Sprite 'menu_quick_match' 'gph_quickMatch' 'menu\quick_match.png' 'Select Mode: cup column tile picture (online / cup flow)'
foreach ($d in @('easy', 'normal', 'special')) {
  Add-Sprite "menu_mode_title_background_$d" "gph_modeTitleBackground_$d"     "menu\mode_title_background_$d.png" "In-match mode title banner ($d), NOT a Select Mode tile"
  Add-Sprite "menu_mode_title_side_$d"       "gph_modeTitleBackgroundSide_$d" "menu\mode_title_side_$d.png"       "In-match mode title banner wings ($d)"
  Add-Sprite "menu_mode_title_star_$d"       "gph_modeTitleBackgroundStar_$d" "menu\mode_title_star_$d.png"       "In-match mode title banner stars ($d)"
}
$notes.Add('- Select Players: 4 seat columns between 5% and 95% of the width, each 22.5% wide (432 px at 1920), centred. Per seat, from the screen centre (y up): character panel 332x283 at +115, its 318x500 wizard window centred on the same point (+115), arrows at x -165/+165 on that line, brick pack panel 332x100 at -110, name plate 367x78 at -230. Corners: menu_corner_small on both panels. Header 800x90 hanging from the top centre. Wizard sprite offsets in the panel: sleeping 1 (0,0) 213x250, 2 (0,38) 225x323, 3 (-9,14) 237x275, 4 (0,11.7) 212x267; ready 1 (0,0) 235x314, 2 (0,21) 259x344, 3 (-3,4) 279x300, 4 (7.5,7) 253x322. A ready seat = selected background + 5 sun-ray layers (masked to 327x267) + outline + wizard. Texts: "Select Players", "[continue]\nTo join", "Player {n}", "Sign In".')
$notes.Add('- Select Mode: columns of 480x800 panels, 540 px apart, centred on x (2 columns: -270 / +270; 3 with the cup column), y 0. Column 1 modes (Race, Survival, Puzzle, then Random), column 2 difficulties (Easy, Normal, Pro, Random). Picked-tile frame 513x445 at x -5.5 with menu_corner_large corners; arrows 94x55 at 130 px outside the top and bottom edge. Tile pictures: race (-3,9) 487x411, survival (6,0) 482x393, puzzle (-7,-1) 505x392, random (0,0) 495x393; difficulty background (-5,0) 461x393 with picture easy (-3.9,35.5) 209x157, normal (-2,38) 328x175, pro (-2.5,42) 370x188 on the _special background, random (0,50) 365x208; light/dark spell icons 56x56 at (85,-190) and (145,-190). The third difficulty is keyed DIFFICULTY_PRO but its English text is "Special". The menu_mode_title_* sprites are the banner shown at the start of a match, not Select Mode art.')

# --- "Press to start" screen (Start scene; logo pieces are world sprites at 26 px/unit) ---
Add-Sprite 'start_message_background' 'gph_menuStartMessageBackground' 'menu\start_message_background.png' 'Bar behind "Press [continue] to start" (63 px high, sized to the text)'
Add-Sprite 'logo_brick' 'gph_logoBrick' 'menu\logo_brick.png' 'Brick that drops onto the logo at 1.63 s'
foreach ($i in 1..8) { Add-Sprite "logo_debris_$i" "gph_logoDebris0$i" "menu\logo_debris_$i.png" "Rubble piece $i under the logo" }
foreach ($i in 1..4) { Add-Sprite "logo_plants_$i" "gph_logoPlants0$i" "menu\logo_plants_$i.png" "Plants growing on top of the logo, piece $i" }
$notes.Add('- Start screen: logo gph_logo is 818x294 at 26 px/unit (818x294 px at 1080p), centre 7 units (182 px) above the screen centre. Pieces relative to the logo container (units, y up): brick (-1.78, 10.11); plants 1 (-9.67, 11.71), 2 (-5.38, 11.79), 3 (3.33, 11.66), 4 (9.795, 11.206) behind the logo; debris 1 (-1.02, 0.49), 2 (0.45, 1.12), 3 (3.58, 0.13), 4 (9.36, 0.35), 5 (5.32, 1.7), 6 (-6.14, 0.16), 7 (-7.27, 1.19); small debris 8 copies at (-5.59, 1.56) x0.846, (-5.03, -0.92) x0.846, (2.82, -1.29) x0.692, (7.93, -0.46) x0.521, (8.63, 1.01) x0.658, (9.06, 1.35) x0.587, (11.36, 1.36) x0.658, (-9.03, 0.59) x0.587, (-9.77, 0.94) x0.736. MoveIn clip (2.62 s): container drops from y 22.39 to 0 by 0.38 s overshooting to -2.41, bounces (-1.26, 0.64, 1.47, 0.15) and settles at 0.81 s; at 1.63 s the brick lands: container dips to -2.24 at 1.92 s, back +0.33, -0.21, 0 at 2.48 s; logo tilts z +3.05 deg at 1.69 s, -1.87 at 1.87, +0.82, +0.32, -0.18, 0 at 2.17; debris pieces hop up about 2.3 units at 1.63 s and fall back by 2.04 s. Sounds on clip events: SWOOSH, GRASS_GROW, FALLING_BRICK, BRICK_LAND (brick nudge + menu crumble). Prompt "Press [continue] to start" ([continue] is the Enter / A key icon) in a bar 218 px below the screen centre, full width, shown once the logo is placed; it blinks on 0.5 s / off 0.5 s, and every 0.075 s after the press. Press plays game start + heavy brick + crumble, rumbles, then goes to the title tiles. Ambience: AMBIENCE_MENU.')

# --- Splash where a lost brick hits the water (Splash particle prefab) ---
# The 64x192 sheet is 3 pictures stacked (one per particle, picked at random), not frames in a row.
$sheetSrc = Join-Path $A 'Texture2D\anim_waterSplash64x196.png'
if (Test-Path $sheetSrc) {
  $sheet = Get-Tex $sheetSrc
  New-Item -ItemType Directory -Force (Join-Path $X 'fx') | Out-Null
  foreach ($i in 1..3) {
    $cell = $sheet.Clone((New-Object System.Drawing.Rectangle 0, (($i - 1) * 64), 64, 64), $sheet.PixelFormat)
    $cell.Save((Join-Path $X "fx\splash_$i.png"), [System.Drawing.Imaging.ImageFormat]::Png); $cell.Dispose()
    # 25.6 px/unit makes the 64 px picture 2.5 units, the largest particle size.
    $manifest.Add("    `"splash_$i`": (file: `"fx/splash_$i.png`", pixels_per_unit: 25.6, pivot: (0.5, 0.5)),")
  }
  $manifest.Add('    "splash": (file: "fx/splash_1.png", pixels_per_unit: 25.6, pivot: (0.5, 0.5)),')
} else { $missing.Add('splash (texture anim_waterSplash64x196.png)') }
foreach ($i in 1..4) { Add-Audio "sfx_splash_$i" "038_Water_splash_0$i" "Splash sound $i (one of the 4 at random)" }
$notes.Add('- Splash: a brick splashes once when its centre falls below y -8.5 (cells, the water line). The splash spawns at the brick centre minus a random 0..0.4 down and lasts up to 2 s; one of 4 splash sounds plays at random. The brick gets an impulse of -0.9 x its velocity (so it nearly stops), rotation damped by 0.25, and afterwards a constant upward force of 17 every physics step (it floats) unless it is a Bubble brick. Two layers: (1) splash_1..3 front: 5 particles at once along a 2.86-wide horizontal line (half arc), no motion, each one of the 3 pictures at random, size random 1.3..2.5 units, lifetime 1 s, size over life 0 -> 1 at 0.10 -> 0.92 at 0.42 -> 0 at 0.68, white, drawn in front (sorting order 2). (2) drops: 12 white dots (gph_whiteCircle) at once from a 1.5-radius disc, shot upward in a narrow cone (4.6 deg) at speed 10..18 units/s, gravity x2 (world gravity 18 -> 36 units/s^2), size 0.3..0.4 units, full size until 0.45 s, shrinking to 0 at 0.93 s (lifetime 5 s, size curve 1 -> 1 at 0.09 -> 0 at 0.19), sorting order 0.')

# --- Single Player and the Endless select (SinglePlayerSelect / LeaderboardGameModeSelect scenes) ---
Add-Sprite 'menu_mode_endless'          'gph_leaderboardModeRace'     'menu\mode_endless.png'          'Endless select: Endless Race button picture (690x173)'
Add-Sprite 'menu_mode_endless_survival' 'gph_leaderboardModeSurvival' 'menu\mode_endless_survival.png' 'Endless select: Endless Survival button picture (689x167)'
Add-Sprite 'menu_mode_endless_puzzle'   'gph_leaderboardModePuzzle'   'menu\mode_endless_puzzle.png'   'Endless select: Endless Puzzle button picture (690x224)'
foreach ($i in 2..4) { Add-Sprite "menu_challenge_wizard$i" "gph_menuChallengeItemWizard0$i" "menu\challenge_wizard$i.png" "Wizard $i face in the corner of an Endless button" }
Add-Sprite 'menu_button_background' 'gph_settingsMenuItemBackground' 'menu\button_background.png' 'Single Player list button (550x78; also Settings rows)'
Add-Sprite 'menu_button_cup_background' 'gph_cupOnlineButtonBackground' 'menu\button_cup_background.png' 'Band across a Single Player button (Endless / Cup), stretched to the button width, 54 high'
Add-Sprite 'menu_leaderboard_panel' 'gph_popupBackgroundPanel' 'menu\leaderboard_panel.png' 'Endless leaderboard panel (800x800), 9-sliced top 51 / bottom 37'
Add-Sprite 'menu_leaderboard_row' 'gph_leaderboardEntryBackground' 'menu\leaderboard_row.png' 'Endless leaderboard row, 9-sliced left 6 / right 7'
$notes.Add('- Single Player screen (title "Single Player"): no mode tiles. The seat (one wizard panel 332x283, arrows at x -200/+200, brick pack panel 332x100 at -225 below it) sits 200 px above the centre; under it a list of 550x78 buttons (menu_button_background, corners menu_corner_small 10 px outside) starting 175 px below the centre: "Trials" at 0, then "Endless" 100 px lower (Endless button has the menu_button_cup_background band). Endless opens the "Endless" screen: buttons 690 wide (Race 690x173, Survival 689x167, Puzzle 690x224 with the picture 28 px up) in a container centred at (135, 375) from the screen centre, each with a challenge wizard face (Race wizard 3, Survival wizard 2, Puzzle wizard 4; 110x108 at (33, 6) from the bottom-left), and an 800x800 leaderboard panel (gph_popupBackgroundPanel) at (0, -30). So Endless Race is shown with menu_mode_endless (gph_leaderboardModeRace).')

# --- Game Completed popup and pause menu ---
Add-Sprite 'popup_background' 'gph_popupBackground' 'menu\popup_background.png' 'Game Completed panel, 9-sliced (borders left 0, bottom 6, right 0, top 198), spills off the bottom-right corner'
Add-Sprite 'menu_item_selected' 'gph_menuItemSelectedBackground' 'menu\item_selected.png' 'Bar behind the picked popup / pause item, 9-sliced (left 16, right 5); width follows the label'
$notes.Add('- Game Completed popup (WinGamePopup): not centred. It sits in the bottom-right corner: content 610 wide x (60 per item + 70) high, anchored to the bottom-right, slides up from below in 0.25 s (ease out quad). The popup_background panel reaches 120 px past the right edge and 100 px below the bottom. Items are right-aligned labels (465x30, 40 px in from the right), 60 px apart, with menu_item_selected (24 high, 30 px in from the right) behind the picked one. Local multiplayer items: "Next match" (only when more than one mode was picked), "Rematch", "Select Mode", "Main menu". There is no "1st Place" title in it. Pause: full screen dark (0.016, 0.012, 0.035, 0.90) with a panel (0.031, 0.020, 0.094, 0.85); "Game Paused" title 480 px in from the left at +260, items "Resume", "Retry", "Quit" below it at +120, each on a 30-high row with menu_item_selected (28 high) behind the picked one.')

# --- Cast beams, lightning and the spell cloud (game/beam, game/lightning, wizard prefabs) ---
Add-Sprite 'cast_beam'            'gph_castEffectBackground'          'fx\cast_beam.png'            'Dark cast beam column: stretched x60 tall going up from the wizard, x300 going down onto the victim'
Add-Sprite 'cast_beam_blob'       'gph_castEffectBlob'                'fx\cast_beam_blob.png'       'Blob riding the beam (2 of them, sorting 11)'
Add-Sprite 'cast_beam_blob_small' 'gph_castEffectBlobSmall'           'fx\cast_beam_blob_small.png' 'Small blob riding the beam (x1.2, sorting 12)'
Add-Sprite 'cast_sunrays'         'gph_castEffectSunrays'             'fx\cast_sunrays.png'         'Rays at the beam foot (3 layers, one at x0.8), and at both ends of a lightning'
Add-Sprite 'cast_circle_thick'    'gph_castEffectCircleOutlineThick'  'fx\cast_circle_thick.png'    'Thick ring at the beam foot / lightning ends'
Add-Sprite 'cast_circle_thin'     'gph_castEffectCircleOutlineThin'   'fx\cast_circle_thin.png'     'Thin ring at the beam foot / lightning ends'
Add-Sprite 'cast_circle_gradient' 'gph_castEffectCircleInnerGradient' 'fx\cast_circle_gradient.png' 'Glow disc at the beam foot / lightning ends'
Add-Sprite 'cast_spark'           'gph_castEffectAnimals02'           'fx\cast_spark.png'           'Spark (BeamSpark)'
Add-Sprite 'cast_lane_beam'       'gph_castBeamLane_World01'          'fx\cast_lane_beam.png'       'Lane wave down the victim''s lane, Race/Puzzle look'
Add-Sprite 'cast_lane_beam_survival' 'gph_castBeamLane_World02'       'fx\cast_lane_beam_survival.png' 'Lane wave, Survival look (drawn 4.55 units right of the lane centre)'
Add-Texture 'cast_lightning' 'gph_lightning.png' 'fx\cast_lightning.png' 'Light cast lightning strip texture, repeated along the bolt (0.02 texture repeats per unit)'
Add-Sprite 'spell_cloud' 'gph_spellBackground_black' 'fx\spell_cloud.png' 'Spell bubble next to the wizard holding the spell icon (drawn x1.31)'
Add-Sprite 'spell_cloud_button_light' 'gph_PSsquareButton_up' 'fx\spell_cloud_button_light.png' 'Button hint on the light spell bubble'
Add-Sprite 'spell_cloud_button_dark'  'gph_PSCircleButton_up' 'fx\spell_cloud_button_dark.png'  'Button hint on the dark spell bubble'
$notes.Add('- Cast beams: a dark cast is a beam up from the caster (BeamIn 0.33 s, gives up after 1 s), then a beam down onto the victim''s brick (BeamDownIn 0.63 s, timeout 1 s), then the lane wave over the victim''s lane (BeamLaneIn 1 s, BeamLaneOut 0.17 s; the overlay is placed at the background camera''s centre and stretched to its full height). Each beam is: cast_beam stretched (x60 up, x300 down) with two white strokes 0.1 wide at x -0.88 / +0.88 (20 tall), blobs riding it, and at its foot 3 sunray layers (Sunrays clip 2.5 s, looping), thick/thin rings and a glow disc. When it ends it squeezes to 1% width in 0.03 s. Explosion clip 0.46 s. Light cast: a lightning strip from the wizard to the brick (Lightning clip 0.4 s) with rays, rings and glow at both ends; the strip is a mesh wobbling as a sine (amplitude grows towards the middle), texture cast_lightning repeating every 50 units.')
$notes.Add('- Spell clouds: each wizard carries two bubbles, light (near the head, about (0.6, 4.8) from the wizard) and dark (about (2.9, 3.5)), each spell_cloud x1.31 with the spell icon inside and a button hint below it. Clips: Show 0.83 s, Floating 3 s loop, Used 1.22 s (bubble, hint and icon animate out), Hide 0.38 s. Positions per wizard: 1 light (0.58, 4.8) dark (2.87, 3.53); 2 light (0.38, 5.37) dark (2.67, 4.1); 3 light (0.75, 5) dark (3.04, 3.73).')
# --- Endless Race: pacer line and the Endless versions of the Race backdrop ---
Add-Sprite 'pacer_line'  'gph_pacerRaceEndless'        'fx\pacer_line.png'  'Endless Race pacer line tile (1x2 units, 77 tiles side by side 1 unit apart); the line is its top edge' @(0.5, 1.0)
Add-Sprite 'pacer_layer' 'gph_endless_race_pace_layer' 'fx\pacer_layer.png' 'Tint drawn under the pacer line (EndlessRacePaceLayer)'
function EndlessBg($n, $sprite, $sx, $sy, $px, $py, $bz, $par, $what) {
  $f = [Globalization.CultureInfo]::InvariantCulture
  $extra = [string]::Format($f, ', screen: ({0:0.0##}, {1:0.0##}), position: ({2:0.0##}, {3:0.0##}), view_height: 41.54, z: {4}, parallax: {5:0.0#}', $sx, $sy, $px, $py, $bz, $par)
  Add-Sprite "background_endless_$n" $sprite "backgrounds\endless_$n.png" $what $null $extra
}
EndlessBg 0 'gph_backgroundBack01_endless'   0.5 0.0 0.0 6.0 2 0.3 'Endless back island (same place as Race)'
EndlessBg 1 'gph_backgroundMiddle01_endless' 0.0 0.0 0.0 5.35 3 0.3 'Endless middle island left'
EndlessBg 2 'gph_backgroundMiddle02_endless' 1.0 0.0 -9.5 5.1 4 0.3 'Endless middle island right'
EndlessBg 3 'gph_backgroundFront01_endless'  0.0 0.0 0.0 3.5 5 0.3 'Endless front island left'
EndlessBg 4 'gph_backgroundFront02_endless'  0.0 0.0 10.0 4.8 6 0.3 'Endless front island centre-left'
EndlessBg 5 'gph_backgroundFront03_endless'  1.0 0.0 0.0 3.6 7 0.3 'Endless front island right'
$notes.Add('- Endless Race: pacer starts at height -5 (slides in from -15 over 1 s), speed 0.2 u/s. Each spell line picked up is a checkpoint: lines sit at the target height, first 10, then +10 each (+11, +12... once past 150). At a checkpoint the pacer stops, eases 2 units down over 1 s, waits 2 s, then speed += step (step starts 0.05, grows 0.01 per checkpoint up to 0.1; speed max 1.5). Game over when the pacer reaches the tower top. Score = ceil(10 x tower height). Light spells only, picked from Ivy, Undo, Ivy, Undo, Petrify. No lives: lost bricks are just gone.')

# --- The original's letters (Enzo), as a letter picture plus sheet ---
& (Join-Path $PSScriptRoot 'font.ps1') -Repo $Repo
$manifest.Add('    "font_main": (file: "fonts/main.ron"),')
$notes.Add('- font_main : the original''s Enzo letters, from its distance-field atlas turned into plain white letters (fonts/main.png, 1024x1024) with 199 glyph metrics (fonts/main.ron). The original also draws an outline and gradient on titles.')

# --- Title screen ---
Add-Sprite 'logo' 'gph_logo_0' 'menu\logo.png' 'Game logo with brick, plants and debris (818x420)'
$menuBg = Join-Path $X 'menu\menu_background.png'
New-Item -ItemType Directory -Force (Split-Path $menuBg) | Out-Null
& (Join-Path $PSScriptRoot 'compose_menu.ps1') -Assets $A -Out $menuBg -GuidIndex (Join-Path $Repo 'research\guid_index.tsv')
$manifest.Add('    "menu_background": (file: "menu/menu_background.png"),')
$notes.Add('- menu_background : 1920x1080 composite of the original title backdrop (it reuses the Race scene: sky, water band, islands, clouds at their measured screen positions, 26 px per unit).')

# --- Player panel portraits ---
foreach ($w in 1..4) { Add-Sprite "wizard_$w" "icn_gameInfoWizard0$w" "hud\wizard_$w.png" "Wizard $w portrait (game-info icon; full-body parts are in the wizards atlas)" }

# --- Orbs and overlays ---
Add-Sprite 'life_orb'      'gph_heart_filled'     'fx\life_orb.png'   'Heart (used for the Extra Life orb and HUD hearts)'
Add-Sprite 'heart'         'gph_heart_filled'     'hud\heart.png'     'HUD heart'
Add-Sprite 'drop_guide'    'gph_brick_guide'      'fx\drop_guide.png' 'Drop guide (stretched to the brick width and view height, tinted)'
Add-Sprite 'spell_line'    'gph_spellPickup'      'fx\spell_line.png' 'One 1-unit piece of the spell line (16 per line, tinted with 9 rainbow colours)'
Add-Sprite 'finish_line'   'gph_finishLinetile'   'fx\finish_line.png' 'Finish line tile (repeat horizontally)'
Add-Texture 'laser' 'gph_laseron.png' 'fx\laser.png' 'Puzzle laser beam (1024x32 texture, repeat sideways); pillar art: gph_laserPillarTile / gph_laserPillarTop'
Add-Sprite 'spell_orb_light' 'icn_lightSpell'     'fx\orb_light.png'  'Light magic icon (the original line has no end orbs; icon used instead)'
Add-Sprite 'spell_orb_dark'  'icn_darkSpell'      'fx\orb_dark.png'   'Dark magic icon'
Add-Sprite 'minibase'      'gph_towerPiece01R_var01' 'fx\minibase.png' 'Minibase ledge, right-hand version (left: gph_towerPiece01L_var01)'
Add-Sprite 'balloon'       'gph_balloon_var01'    'fx\balloon.png'    'Slow spell balloon'
Add-Sprite 'bubble'        'gph_bubble3_var1'     'fx\bubble.png'     'Bubble (size 3; size 2 and 4 variants exist)'
Add-Sprite 'water'         'gph_foregroundWater'  'fx\water.png'      'Race foreground water tile (1x2 units, repeat sideways); pivot at the top edge' @(0.5, 1.0)
Add-Sprite 'mist'          'gph_cursedLineMist'   'fx\mist.png'       'Mist puff'
Add-Sprite 'grass_overlay' 'gph_grassLarge_var01' 'fx\grass.png'      'Grass tuft for Grassy (4 variants)'
foreach ($b in 'I','O','T','S','Z','J','L') {
  Add-Sprite "brick_$($b)_stone" "gph_brick$($b)_petrified01" "bricks\stone_$b.png" "Petrified brick $b"
  Add-Sprite "brick_$($b)_ice"   "gph_brick$($b)_ice"         "bricks\ice_$b.png"   "Ice brick $b"
}
foreach ($m in @(@('block','01'), @('blob','02'), @('table','03'))) {
  Add-Sprite "mystery_$($m[0])_stone" "gph_brickMystery$($m[1])_petrified" "bricks\mystery_$($m[0])_stone.png" "Petrified Mystery $($m[0])"
  Add-Sprite "mystery_$($m[0])_ice"   "gph_brickMystery$($m[1])_ice"       "bricks\mystery_$($m[0])_ice.png"   "Ice Mystery $($m[0])"
}

# --- Spell icons ---
$icons = [ordered]@{ Ivy='icn_spellIvy'; Petrify='icn_spellPetrify'; Undo='icn_spellUndo'; ExtraLife='icn_spellExtraLife'; Minibase='icn_spellTowerpiece'; AutoBuild='icn_spellAutoBuild'; Large='icn_spellLarge'; Fast='icn_spellFastBrick'; NoRotate='icn_spellRotatedBrick'; AutoRotate='icn_spellAutoRotate'; Ice='icn_spellIceBrick'; Mist='icn_spellMist'; Slow='icn_spellSlowDown'; Mystery='icn_spellMystery'; Bubble='icn_spellBubble'; Grassy='icn_spellGrass'; Ghost='icn_spellGhosts' }
foreach ($k in $icons.Keys) { Add-Sprite "spell_icon_$k" $icons[$k] "icons\$k.png" "Spell icon $k" }
Add-Sprite 'spell_slot_light' 'gph_spellBackground_white' 'icons\slot_light.png' 'Light spell slot background'
Add-Sprite 'spell_slot_dark'  'gph_spellBackground_black' 'icons\slot_dark.png'  'Dark spell slot background'

# --- Music and sounds ---
Add-Audio 'music_menu'     '044_Music_menu'      'Menu music'
Add-Audio 'music_race'     '000_Track_01_single_speed' 'Race music'
Add-Audio 'music_survival' '004_Survival'        'Survival music'
Add-Audio 'music_puzzle'   '002_Puzzle_v2'       'Puzzle music'
Add-Audio 'sfx_land'       '007_Brick_land'      'Brick lands'
Add-Audio 'sfx_move'       '056_Move_brick_v2'   'Brick moves a step'
Add-Audio 'sfx_nudge'      '059_Nudge_brick_v2'  'Nudge'
Add-Audio 'sfx_rotate'     '028_Rotate_brick'    'Rotate'
Add-Audio 'sfx_drop_lost'  '057_Lose_brick_v3'   'Brick lost'
Add-Audio 'sfx_line_collect' '006_Pickup_spell'  'Spell line collected'
Add-Audio 'sfx_cast_light' '013_Cast_lightning_level_01' 'Light magic cast'
Add-Audio 'sfx_cast_dark'  '064_Cast_dark_magic' 'Dark magic cast'
Add-Audio 'sfx_countdown'  '093_Countdown'       'Countdown tick'
Add-Audio 'sfx_win'        '046_Win_game_stinger' 'Win'
Add-Audio 'sfx_lose'       '045_Lose_game_stinger' 'Lose'

$texCache.Values | ForEach-Object { $_.Dispose() }

$hdr = '// Generated locally from your own Tricky Towers install. Do not commit.'
[IO.File]::WriteAllLines((Join-Path $X 'manifest.ron'), [string[]](@($hdr, '{') + $manifest + @('}')), $utf8)
$notesHead = @('# extracted/ notes (generated, local only)', '', 'Pixels per unit: Unity sprite value; a brick cell is 1 unit.', '')
$notesTail = @('', '## Race sky and water (no image; draw as gradients)', '- Sky gradient colours (RGB): (108,152,223), (179,196,251), (125,192,239).', '- Survival sky: top (51,30,103), glow band (91,60,175), bottom (91,83,173); during curse waves it fades to top (26,13,58), band (144,52,110), bottom (147,64,157). Survival clouds tinted (89,80,169).', '- Puzzle ruins (bg_puzzle_* / background_puzzle_*) are two-colour MASK sprites: the original shader replaces red with colour 1 and yellow with colour 2 (outline fades to transparent). Back: #F4D9B0 / #F7E0BE. Middle: #E0AC92 / #D69883. Front: #C47464 / #C47464. Sandstorm tint ARGB #9B8679A0.', '- Background water: bottom 6 screen units, top (61,82,201) to bottom (137,146,217); islands bob gently.', '', '## Also available in research\ripped, not staged', '- Wizard sprites (atlas SpriteAtlasTexture-wizards), cast effects, other brick packs, ambience and wizard speech audio.', '- Race Pro / endless / survival / desert background layers.')
[IO.File]::WriteAllLines((Join-Path $X 'notes.md'), [string[]]($notesHead + $notes + $notesTail), $utf8)
[IO.File]::WriteAllLines((Join-Path $X 'staged.tsv'), [string[]]$stagedMap, $utf8)
"staged $($manifest.Count) entries; missing: $($missing.Count)"
$missing | ForEach-Object { "MISSING $_" }
