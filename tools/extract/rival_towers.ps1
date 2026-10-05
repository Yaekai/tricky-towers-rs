param([string]$Repo)
# Writes extracted\rival_towers.ron: the 13 rival towers of the race trials (EnemyTowerLevel01..13),
# each a list of stone pieces in pop-in order. A piece: its art key, centre and turn (degrees) from
# the lane centre at the base's top, scale, and the pop-in group (one group per top-level child;
# an ivy cluster pops in as one). Local only: extracted\ is gitignored.
$ErrorActionPreference = 'Stop'
$A = Join-Path $Repo 'research\ripped\ExportedProject\Assets'
$idx = @{}; Get-Content (Join-Path $Repo 'research\guid_index.tsv') | ForEach-Object { $p = $_ -split "`t"; $idx[$p[0]] = [IO.Path]::GetFileNameWithoutExtension($p[1]) }
$inv = [Globalization.CultureInfo]::InvariantCulture
function V($s, $k) { [double]::Parse([regex]::Match($s, "$($k): (-?[\d.eE+-]+)").Groups[1].Value, $inv) }
function F($v) { ([math]::Round($v, 3)).ToString($inv) }
$out = New-Object System.Collections.Generic.List[string]
$out.Add('{')
foreach ($n in 1..13) {
  $file = Join-Path $A ('Resources\game\floors\race\EnemyTowerLevel{0:D2}.prefab' -f $n)
  $docs = [regex]::Split([IO.File]::ReadAllText($file), '(?m)^--- ') | Where-Object { $_ -match '^!u!(\d+) &(\d+)' }
  $tr = @{}; $trByGo = @{}; $sprites = @()
  foreach ($d in $docs) {
    $m = [regex]::Match($d, '^!u!(\d+) &(\d+)'); $type = $m.Groups[1].Value; $id = $m.Groups[2].Value
    $go = [regex]::Match($d, 'm_GameObject: \{fileID: (\d+)').Groups[1].Value
    if ($type -eq '4') {
      $pos = [regex]::Match($d, 'm_LocalPosition: \{([^}]*)\}').Groups[1].Value
      $sc = [regex]::Match($d, 'm_LocalScale: \{([^}]*)\}').Groups[1].Value
      $rot = [regex]::Match($d, 'm_LocalRotation: \{([^}]*)\}').Groups[1].Value
      $kids = [regex]::Matches(([regex]::Match($d, '(?s)m_Children:(.*?)m_Father').Groups[1].Value), 'fileID: (\d+)') | ForEach-Object { $_.Groups[1].Value }
      $t = [pscustomobject]@{ id = $id; parent = [regex]::Match($d, 'm_Father: \{fileID: (\d+)').Groups[1].Value; kids = @($kids)
        x = (V $pos 'x'); y = (V $pos 'y'); s = (V $sc 'x'); r = 2 * [math]::Atan2((V $rot 'z'), (V $rot 'w')) }
      $tr[$id] = $t; $trByGo[$go] = $t
    } elseif ($type -eq '212') {
      $g = [regex]::Match($d, 'm_Sprite: \{fileID: \d+, guid: ([0-9a-f]+)').Groups[1].Value
      $sprites += [pscustomobject]@{ go = $go; name = $idx[$g] }
    }
  }
  $root = $tr.Values | Where-Object { $_.parent -eq '0' } | Select-Object -First 1
  $group = @{}; $i = 0
  foreach ($k in $root.kids) { $i++; $stack = @($k); while ($stack.Count) { $c = $stack[0]; $stack = @($stack | Select-Object -Skip 1); $group[$c] = $i; $stack += $tr[$c].kids } }
  $out.Add("    $($n): [")
  foreach ($s in $sprites) {
    $m = [regex]::Match($s.name, '^gph_brick(\w)_stone$'); $l = [regex]::Match($s.name, '^gph_towerPiece01([LR])_var01$')
    if ($m.Success) { $key = "rival_brick_$($m.Groups[1].Value)" } elseif ($l.Success) { $key = "rival_minibase_$($l.Groups[1].Value)" } elseif ($s.name -match '^anim_ivy_(\w)_') { $key = "ivy" } else { continue }
    $t = $trByGo[$s.go]; $x = 0.0; $y = 0.0; $sc = 1.0; $r = 0.0; $c = $t
    # Sum the transform chain up to (not including) the root.
    $chain = @(); while ($c -and $c.parent -ne '0') { $chain += $c; $c = $tr[$c.parent] }
    $px = 0.0; $py = 0.0
    foreach ($c in $chain) {
      $px *= $c.s; $py *= $c.s
      $cos = [math]::Cos($c.r); $sin = [math]::Sin($c.r)
      $nx = $px * $cos - $py * $sin + $c.x; $ny = $px * $sin + $py * $cos + $c.y
      $px = $nx; $py = $ny; $sc *= $c.s; $r += $c.r
    }
    $deg = $r * 180 / [math]::PI
    if ($key -eq 'ivy') { $out.Add("        (art: `"ivy_$([regex]::Match($s.name, '^anim_ivy_(\w)_').Groups[1].Value)`", x: $(F $px), y: $(F $py), turn: $(F $deg), scale: $(F $sc), group: $($group[$t.id])),"); continue }
    $out.Add("        (art: `"$key`", x: $(F $px), y: $(F $py), turn: $(F $deg), scale: $(F $sc), group: $($group[$t.id])),")
  }
  $out.Add('    ],')
}
$out.Add('}')
$path = Join-Path $Repo 'extracted\rival_towers.ron'
[IO.File]::WriteAllLines($path, $out)
Write-Host "Wrote $path"
