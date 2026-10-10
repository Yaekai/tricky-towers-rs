param([string]$Repo, [string]$FontName = 'EnzoBlack SDF', [string]$OutName = 'main')
# Turns the original's TextMeshPro font (a signed-distance-field letter picture plus glyph
# metrics) into extracted\fonts\main.png (plain white letters with alpha) and
# extracted\fonts\main.ron (the letter sheet described in docs/art-layout.md). Local only.
Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing @"
using System; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices;
public static class Sdf {
  // Distance field -> white letters: alpha is a smooth step around the 0.5 edge.
  public static Bitmap ToAlpha(Bitmap src, float lo, float hi) {
    int w = src.Width, h = src.Height;
    var dst = new Bitmap(w, h, PixelFormat.Format32bppArgb);
    var r = new Rectangle(0, 0, w, h);
    var s = src.LockBits(r, ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var d = dst.LockBits(r, ImageLockMode.WriteOnly, PixelFormat.Format32bppArgb);
    var sb = new byte[s.Stride * h]; Marshal.Copy(s.Scan0, sb, 0, sb.Length);
    var db = new byte[d.Stride * h];
    for (int i = 0; i < w * h; i++) {
      int o = i * 4;
      // The field is in the alpha channel for Alpha8 atlases, or the colour channels otherwise.
      int raw = sb[o + 3] == 255 ? sb[o + 2] : sb[o + 3]; float v = raw / 255f;
      float t = Math.Min(1f, Math.Max(0f, (v - lo) / (hi - lo)));
      t = t * t * (3f - 2f * t);
      db[o] = 255; db[o + 1] = 255; db[o + 2] = 255; db[o + 3] = (byte)(t * 255f + 0.5f);
    }
    Marshal.Copy(db, 0, d.Scan0, db.Length);
    src.UnlockBits(s); dst.UnlockBits(d);
    return dst;
  }
  // The distance field itself, in the alpha of white pixels (alpha stays linear when loaded),
  // for drawing the letters sharp at any size as TextMeshPro does.
  public static Bitmap ToField(Bitmap src) {
    int w = src.Width, h = src.Height;
    var dst = new Bitmap(w, h, PixelFormat.Format32bppArgb);
    var r = new Rectangle(0, 0, w, h);
    var s = src.LockBits(r, ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var d = dst.LockBits(r, ImageLockMode.WriteOnly, PixelFormat.Format32bppArgb);
    var sb = new byte[s.Stride * h]; Marshal.Copy(s.Scan0, sb, 0, sb.Length);
    var db = new byte[d.Stride * h];
    for (int i = 0; i < w * h; i++) {
      int o = i * 4;
      int raw = sb[o + 3] == 255 ? sb[o + 2] : sb[o + 3];
      db[o] = 255; db[o + 1] = 255; db[o + 2] = 255; db[o + 3] = (byte)raw;
    }
    Marshal.Copy(db, 0, d.Scan0, db.Length);
    src.UnlockBits(s); dst.UnlockBits(d);
    return dst;
  }
}
"@
$inv = [Globalization.CultureInfo]::InvariantCulture
$A = Join-Path $Repo 'research\ripped\ExportedProject\Assets'
$X = Join-Path $Repo 'extracted'
$asset = [IO.File]::ReadAllText((Join-Path $A "Resources\fonts\$FontName.asset"))
function Num($name) { [double]::Parse([regex]::Match($asset, "(?m)^\s+$($name): (-?[\d.]+)").Groups[1].Value, $inv) }
$size = Num 'PointSize'; $line = Num 'LineHeight'; $asc = Num 'Ascender'

# The atlas picture.
$src = [System.Drawing.Bitmap]::FromFile((Join-Path $A "Texture2D\$FontName Atlas.png"))
$letters = [Sdf]::ToAlpha($src, 0.42, 0.58)
New-Item -ItemType Directory -Force (Join-Path $X 'fonts') | Out-Null
$letters.Save((Join-Path $X "fonts\$OutName.png"), [System.Drawing.Imaging.ImageFormat]::Png)
# The outline: the same field cut further out. TextMeshPro's outline width 0.35 (the menus'
# titles and hints) moves the edge from 0.5 to about 0.325 of the field.
$outline = [Sdf]::ToAlpha($src, 0.25, 0.4)
$outline.Save((Join-Path $X "fonts\${OutName}_outline.png"), [System.Drawing.Imaging.ImageFormat]::Png)
$outline.Dispose()
$field = [Sdf]::ToField($src)
$field.Save((Join-Path $X "fonts\${OutName}_sdf.png"), [System.Drawing.Imaging.ImageFormat]::Png)
$field.Dispose()
$aw = $src.Width; $ah = $src.Height
$src.Dispose(); $letters.Dispose()

# Glyphs: TextMeshPro gives x, y (from the picture's top left), size, offset from the pen on the
# base line to the glyph's top left (y up) and advance, all in picture pixels.
$block = $asset.Substring($asset.IndexOf('m_glyphInfoList:'))
$rx = '- id: (\d+)\s+x: (-?[\d.]+)\s+y: (-?[\d.]+)\s+width: (-?[\d.]+)\s+height: (-?[\d.]+)\s+xOffset: (-?[\d.]+)\s+yOffset: (-?[\d.]+)\s+xAdvance: (-?[\d.]+)'
$lines = New-Object System.Collections.Generic.List[string]
foreach ($m in [regex]::Matches($block, $rx)) {
  $id = [int]$m.Groups[1].Value
  $ch = [char]$id
  $lit = switch ($ch) { "'" { "'\''" } '\' { "'\\'" } default { if ($id -lt 32) { $null } else { "'$ch'" } } }
  if (-not $lit) { continue }
  $g = 2..8 | ForEach-Object { [double]::Parse($m.Groups[$_].Value, $inv) }
  $gx = $g[0]; $gy = $g[1]; $gw = $g[2]; $gh = $g[3]
  # Characters with nothing to draw (space) keep only their advance.
  if ($id -eq 32 -or $gx + $gw -gt $aw -or $gy + $gh -gt $ah) { $gx = 0; $gy = 0; $gw = 0; $gh = 0 }
  $lines.Add([string]::Format($inv, '        {0}: (rect: ({1:0.###}, {2:0.###}, {3:0.###}, {4:0.###}), offset: ({5:0.###}, {6:0.###}), advance: {7:0.###}),', $lit, $gx, $gy, $gw, $gh, $g[4], $g[5], $g[6]))
}
$out = @("// The original's letters ($FontName), from your own install. Do not commit.", '(',
  "    image: `"fonts/$OutName.png`",",
  "    outline: Some(`"fonts/${OutName}_outline.png`"),",
  "    sdf: Some(`"fonts/${OutName}_sdf.png`"),",
  '    outline_pad: 5,',
  [string]::Format($inv, '    size: {0:0.###},', $size),
  [string]::Format($inv, '    line_height: {0:0.###},', $line),
  [string]::Format($inv, '    ascender: {0:0.###},', $asc),
  '    glyphs: {') + $lines + @('    },', ')')
[IO.File]::WriteAllLines((Join-Path $X "fonts\$OutName.ron"), [string[]]$out, (New-Object System.Text.UTF8Encoding($false)))
"font: $($lines.Count) glyphs, size $size, line $line, ascender $asc, atlas ${aw}x$ah"
