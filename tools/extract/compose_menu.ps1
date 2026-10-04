param([string]$Assets, [string]$Out, [string]$GuidIndex)
# Composes the original title backdrop (the Race background scene) into one 1920x1080 PNG.
# Screen units: 26 px each, origin bottom-left, y up. Positions measured from the original prefabs.
Add-Type -AssemblyName System.Drawing
$idx = @{}; Get-Content $GuidIndex | ForEach-Object { $p = $_ -split "`t"; $idx[$p[0]] = $p[1] }
$W = 1920; $H = 1080; $U = 26.0
$bmp = New-Object System.Drawing.Bitmap $W, $H
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic

function Grad($top, $bottom, [int]$y0, [int]$y1) {
  $rect = New-Object System.Drawing.Rectangle 0, $y0, $W, ($y1 - $y0)
  $c0 = [System.Drawing.Color]::FromArgb(255, $top[0], $top[1], $top[2])
  $c1 = [System.Drawing.Color]::FromArgb(255, $bottom[0], $bottom[1], $bottom[2])
  $br = New-Object System.Drawing.Drawing2D.LinearGradientBrush $rect, $c0, $c1, 90.0
  $g.FillRectangle($br, $rect); $br.Dispose()
}
# Sky: top -> glow band at 65% height -> bottom.
$band = [int]($H * 0.35)
Grad @(108,152,223) @(125,192,239) 0 $band
Grad @(125,192,239) @(179,196,251) ($band - 1) $H

$texCache = @{}
$num = '(-?[0-9.]+(?:[eE][-+]?\d+)?)'
function Draw($sprite, [double]$x, [double]$y, $pivotOverride = $null) {
  $t = [IO.File]::ReadAllText((Join-Path $Assets "Sprite\$sprite.asset"))
  $r = [regex]::Match($t, "textureRect:\s*\n\s*serializedVersion: 2\s*\n\s*x: $num\s*\n\s*y: $num\s*\n\s*width: $num\s*\n\s*height: $num")
  $pv = [regex]::Match($t, "m_Pivot: \{x: $num, y: $num\}")
  $tex = $idx[[regex]::Match($t, 'm_RD:\s*\n\s*texture: \{fileID: \d+, guid: ([0-9a-f]+)').Groups[1].Value]
  if (-not $texCache.ContainsKey($tex)) { $texCache[$tex] = [System.Drawing.Bitmap]::FromFile($tex) }
  $img = $texCache[$tex]
  $sx = [double]$r.Groups[1].Value; $sy = [double]$r.Groups[2].Value; $sw = [double]$r.Groups[3].Value; $sh = [double]$r.Groups[4].Value
  $px = [double]$pv.Groups[1].Value; $py = [double]$pv.Groups[2].Value
  if ($pivotOverride) { $px = $pivotOverride[0]; $py = $pivotOverride[1] }
  $src = New-Object System.Drawing.RectangleF $sx, ($img.Height - $sy - $sh), $sw, $sh
  $left = $x * $U - $px * $sw
  $top = $H - ($y * $U + (1 - $py) * $sh)
  $dst = New-Object System.Drawing.RectangleF $left, $top, $sw, $sh
  $g.DrawImage($img, $dst, $src, [System.Drawing.GraphicsUnit]::Pixel)
}
$SW = $W / $U   # 73.85 screen units wide
# Large clouds centred on the bottom edge (behind the islands).
Draw 'gph_cloudLarge_var01_0' ($SW * 0.3) 0
Draw 'gph_cloudLarge_var02'   ($SW * 0.8) 0
# Water band: bottom 6 units.
Grad @(61,82,201) @(137,146,217) ($H - [int](6 * $U)) $H
# Islands, back to front.
Draw 'gph_backgroundBack01'   ($SW / 2) 6.0
Draw 'gph_backgroundMiddle01' 0 5.35
Draw 'gph_backgroundMiddle02' ($SW - 9.5) 5.1
Draw 'gph_backgroundFront01'  0 3.5
Draw 'gph_backgroundFront02'  10 4.8
Draw 'gph_backgroundFront03'  $SW 3.6
# Small clouds in the sky.
Draw 'gph_cloud_var01_0' ($SW * 0.15) (41.54 * 0.6)
Draw 'gph_cloud_var04_0' ($SW * 0.6)  (41.54 * 0.85)
$g.Dispose()
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png); $bmp.Dispose()
$texCache.Values | ForEach-Object { $_.Dispose() }
"composed $Out"
