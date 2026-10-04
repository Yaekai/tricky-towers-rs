param([string]$Prefab, [string]$GuidIndex)
# Prints every sprite in a prefab with its path from the root, its position summed through its
# parents (scale applied, rotation ignored), its sorting order and the sprite name.
# Output: path<TAB>sprite<TAB>x<TAB>y<TAB>scaleX<TAB>scaleY<TAB>order
$idx = @{}; Get-Content $GuidIndex | ForEach-Object { $p = $_ -split "`t"; $idx[$p[0]] = $p[1] }
$inv = [Globalization.CultureInfo]::InvariantCulture
$text = [IO.File]::ReadAllText($Prefab)
$docs = [regex]::Split($text, '(?m)^--- ') | Where-Object { $_ -match '^!u!(\d+) &(\d+)' }
$goName = @{}; $trByGo = @{}; $trById = @{}; $renderers = @()
function V($s, $k) { [double]::Parse([regex]::Match($s, "$($k): (-?[\d.eE+-]+)").Groups[1].Value, $inv) }
foreach ($d in $docs) {
  $m = [regex]::Match($d, '^!u!(\d+) &(\d+)'); $type = $m.Groups[1].Value; $id = $m.Groups[2].Value
  if ($type -eq '1') { $goName[$id] = [regex]::Match($d, 'm_Name: (.*)').Groups[1].Value.Trim() }
  elseif ($type -eq '4') {
    $pos = [regex]::Match($d, 'm_LocalPosition: \{([^}]*)\}').Groups[1].Value
    $sc = [regex]::Match($d, 'm_LocalScale: \{([^}]*)\}').Groups[1].Value
    $t = [pscustomobject]@{ id = $id; go = [regex]::Match($d, 'm_GameObject: \{fileID: (\d+)').Groups[1].Value
      parent = [regex]::Match($d, 'm_Father: \{fileID: (\d+)').Groups[1].Value
      x = (V $pos 'x'); y = (V $pos 'y'); sx = (V $sc 'x'); sy = (V $sc 'y') }
    $trByGo[$t.go] = $t; $trById[$id] = $t
  }
  elseif ($type -eq '212') {
    $renderers += [pscustomobject]@{ go = [regex]::Match($d, 'm_GameObject: \{fileID: (\d+)').Groups[1].Value
      sprite = [regex]::Match($d, 'm_Sprite: \{fileID: \d+, guid: ([0-9a-f]+)').Groups[1].Value
      order = [regex]::Match($d, 'm_SortingOrder: (-?\d+)').Groups[1].Value }
  }
}
foreach ($r in $renderers) {
  $t = $trByGo[$r.go]; $names = @(); $x = 0.0; $y = 0.0; $sx = 1.0; $sy = 1.0
  # Walk up: world = parent.pos + parent.scale * child.pos.
  $chain = @(); $cur = $t
  while ($cur) { $chain += $cur; $cur = if ($cur.parent -and $cur.parent -ne '0') { $trById[$cur.parent] } else { $null } }
  [array]::Reverse($chain)
  foreach ($c in $chain) { $x = $x + $sx * $c.x; $y = $y + $sy * $c.y; $sx = $sx * $c.sx; $sy = $sy * $c.sy; $names += $goName[$c.go] }
  $path = ($names | Select-Object -Skip 1) -join '/'
  $sp = if ($r.sprite -and $idx[$r.sprite]) { [IO.Path]::GetFileNameWithoutExtension($idx[$r.sprite]) } else { '-' }
  [string]::Format($inv, "{0}`t{1}`t{2:0.###}`t{3:0.###}`t{4:0.###}`t{5:0.###}`t{6}", $path, $sp, $x, $y, $sx, $sy, $r.order)
}
