# Samples the CastSelf pose and its wand from the original's planar transform curves.

function Read-CastVector($value) {
  $culture = [Globalization.CultureInfo]::InvariantCulture
  $components = [regex]::Matches($value, '[xyzw]: ([-+\d.eE]+)')
  ,@($components | ForEach-Object { [double]::Parse($_.Groups[1].Value, $culture) })
}

function Sample-CastCurve($keys, [double]$time, $fallback) {
  if (!$keys) { return ,$fallback }
  if ($time -le $keys[0].Time) { return ,$keys[0].Value }
  for ($i = 1; $i -lt $keys.Count; $i++) {
    $right = $keys[$i]
    if ($time -gt $right.Time) { continue }
    $left = $keys[$i - 1]
    $span = $right.Time - $left.Time
    $u = ($time - $left.Time) / $span
    $u2 = $u * $u
    $u3 = $u2 * $u
    $point = for ($axis = 0; $axis -lt $left.Value.Count; $axis++) {
      (2 * $u3 - 3 * $u2 + 1) * $left.Value[$axis] +
        ($u3 - 2 * $u2 + $u) * $span * $left.OutSlope[$axis] +
        (-2 * $u3 + 3 * $u2) * $right.Value[$axis] +
        ($u3 - $u2) * $span * $right.InSlope[$axis]
    }
    return ,$point
  }
  return ,$keys[-1].Value
}

function Combine-CastPose($parent, $child) {
  $x = $child[0] * $parent[3]
  $y = $child[1] * $parent[4]
  $sin = [Math]::Sin($parent[2])
  $cos = [Math]::Cos($parent[2])
  ,@(($parent[0] + $x * $cos - $y * $sin), ($parent[1] + $x * $sin + $y * $cos),
    ($parent[2] + $child[2]), ($parent[3] * $child[3]), ($parent[4] * $child[4]))
}

function Read-CastRig($prefab, $clip, $spritePaths) {
  if (!(Test-Path -LiteralPath $clip)) { return }
  $culture = [Globalization.CultureInfo]::InvariantCulture
  $inner = 'WizardObject/Wizard'
  $wand = 'ArmBack/SpellCastContainer'
  $nodes = @{}
  & (Join-Path $PSScriptRoot 'prefab_tree.ps1') -Prefab $prefab -Transforms | ForEach-Object {
    if ($_.Path -eq $inner) { $nodes[''] = $_ }
    elseif ($_.Path.StartsWith("$inner/")) { $nodes[$_.Path.Substring($inner.Length + 1)] = $_ }
  }
  if (!$nodes.ContainsKey('') -or !$nodes.ContainsKey($wand)) { return }
  $poses = @{ '' = [System.Collections.Generic.List[string]]::new() }
  foreach ($path in $spritePaths) {
    if ($path.StartsWith("$inner/")) {
      $poses[$path.Substring($inner.Length + 1)] = [System.Collections.Generic.List[string]]::new()
    }
  }
  $wandFrames = [System.Collections.Generic.List[string]]::new()

  $text = [IO.File]::ReadAllText($clip)
  $tracks = @{}
  $vectorKey = 'time: ([-+\d.eE]+)\s+value: \{([^}]+)\}\s+inSlope: \{([^}]+)\}\s+outSlope: \{([^}]+)\}'
  foreach ($section in [regex]::Matches($text, '(?ms)^  m_(Position|Scale|Rotation|Euler)Curves:\r?\n(.*?)(?=^  m_)')) {
    $kind = $section.Groups[1].Value
    foreach ($curve in [regex]::Split($section.Groups[2].Value, '(?m)^  - curve:') | Select-Object -Skip 1) {
      $path = [regex]::Match($curve, '(?m)^    path: ?([^\r\n]*)').Groups[1].Value
      if (!$nodes.ContainsKey($path)) { continue }
      $tracks["${kind}:$path"] = @([regex]::Matches($curve, $vectorKey) | ForEach-Object {
        [pscustomobject]@{
          Time = [double]::Parse($_.Groups[1].Value, $culture)
          Value = Read-CastVector $_.Groups[2].Value
          InSlope = Read-CastVector $_.Groups[3].Value
          OutSlope = Read-CastVector $_.Groups[4].Value
        }
      })
    }
  }
  $length = [double]::Parse([regex]::Match($text, 'm_StopTime: ([-+\d.eE]+)').Groups[1].Value, $culture)
  # Keep the curved motion between the clip's sprite frames.
  $rate = 4 * [double]::Parse([regex]::Match($text, 'm_SampleRate: ([-+\d.eE]+)').Groups[1].Value, $culture)
  $paths = $nodes.Keys | Sort-Object Length
  $last = [Math]::Round($length, 7)
  for ($frame = 0; $frame -le [Math]::Ceiling($length * $rate); $frame++) {
    $time = [Math]::Min($frame / $rate, $length)
    $sample = @{}
    foreach ($path in $paths) {
      $node = $nodes[$path]
      $pos = Sample-CastCurve $tracks["Position:$path"] $time @($node.X, $node.Y)
      $scale = Sample-CastCurve $tracks["Scale:$path"] $time @($node.SX, $node.SY)
      $turn = $node.Turn
      if ($tracks["Rotation:$path"]) {
        $rotation = Sample-CastCurve $tracks["Rotation:$path"] $time $null
        $turn = 2 * [Math]::Atan2($rotation[2], $rotation[3])
      } elseif ($tracks["Euler:$path"]) {
        $rotation = Sample-CastCurve $tracks["Euler:$path"] $time $null
        $turn = $rotation[2] * [Math]::PI / 180
      }
      $pose = @($pos[0], $pos[1], $turn, $scale[0], $scale[1])
      $slash = $path.LastIndexOf('/')
      if ($slash -ge 0) { $pose = Combine-CastPose $sample[$path.Substring(0, $slash)] $pose }
      $sample[$path] = $pose
      if ($poses.ContainsKey($path)) {
        $values = @($time) + $pose
        $poses[$path].Add([string]::Format($culture,
          '({0:0.0######}, (at: ({1:0.0######}, {2:0.0######}), turn: {3:0.0######}, scale: ({4:0.0######}, {5:0.0######})))', $values))
      }
    }
    $tip = Combine-CastPose $sample[''] $sample[$wand]
    $wandFrames.Add([string]::Format($culture, '({0:0.0######}, ({1:0.0######}, {2:0.0######}))', $time, $tip[0], $tip[1]))
    if ([Math]::Round($time, 7) -ge $last) { break }
  }
  [pscustomobject]@{ Wand = $wandFrames; Poses = $poses; Inner = "$inner/" }
}
