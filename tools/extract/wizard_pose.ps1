# Samples the cup wizard's pose clips and the CastSelf wand from local animation curves.
Add-Type -AssemblyName System.Numerics

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

function Combine-RigPose($parent, $child) {
  $offset = [System.Numerics.Vector3]::Multiply($child.At, $parent.Scale)
  @{
    At = [System.Numerics.Vector3]::Add($parent.At, [System.Numerics.Vector3]::Transform($offset, $parent.Rotation))
    Rotation = [System.Numerics.Quaternion]::Multiply($parent.Rotation, $child.Rotation)
    Scale = [System.Numerics.Vector3]::Multiply($parent.Scale, $child.Scale)
  }
}

function Read-RigNodes($prefab, $inner) {
  $nodes = @{}
  & (Join-Path $PSScriptRoot 'prefab_tree.ps1') -Prefab $prefab -Transforms | ForEach-Object {
    if ($_.Path -eq $inner.TrimEnd('/')) { $nodes[''] = $_ }
    elseif ($_.Path.StartsWith($inner)) { $nodes[$_.Path.Substring($inner.Length)] = $_ }
  }
  $nodes
}

function Read-RigPose($nodes, $clip, $spritePaths, [bool]$stageWand) {
  if (!(Test-Path -LiteralPath $clip) -or !$nodes.ContainsKey('')) { return }
  $culture = [Globalization.CultureInfo]::InvariantCulture
  $wand = 'ArmBack/SpellCastContainer'
  $poses = @{ '' = [System.Collections.Generic.List[string]]::new() }
  foreach ($path in $spritePaths) {
    $poses[$path] = [System.Collections.Generic.List[string]]::new()
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
      if ($tracks["Euler:$path"]) {
        $angles = Sample-CastCurve $tracks["Euler:$path"] $time $null
        $turn = [System.Numerics.Quaternion]::CreateFromAxisAngle([System.Numerics.Vector3]::UnitZ, $angles[2] * [Math]::PI / 180)
      } else {
        $q = Sample-CastCurve $tracks["Rotation:$path"] $time $node.Rotation
        $turn = [System.Numerics.Quaternion]::Normalize([System.Numerics.Quaternion]::new($q[0], $q[1], $q[2], $q[3]))
      }
      $pose = @{
        At = [System.Numerics.Vector3]::new($pos[0], $pos[1], 0)
        Rotation = $turn
        Scale = [System.Numerics.Vector3]::new($scale[0], $scale[1], 1)
      }
      $slash = $path.LastIndexOf('/')
      if ($slash -ge 0) { $pose = Combine-RigPose $sample[$path.Substring(0, $slash)] $pose }
      $sample[$path] = $pose
      if ($poses.ContainsKey($path)) {
        $q = $pose.Rotation
        $values = @($time, $pose.At.X, $pose.At.Y, $q.X, $q.Y, $q.Z, $q.W, $pose.Scale.X, $pose.Scale.Y)
        $poses[$path].Add([string]::Format($culture,
          '({0:0.0######}, (at: ({1:0.0######}, {2:0.0######}), rotation: ({3:0.0######}, {4:0.0######}, {5:0.0######}, {6:0.0######}), scale: ({7:0.0######}, {8:0.0######})))', $values))
      }
    }
    if ($stageWand -and $sample.ContainsKey($wand)) {
      $tip = (Combine-RigPose $sample[''] $sample[$wand]).At
      $wandFrames.Add([string]::Format($culture, '({0:0.0######}, ({1:0.0######}, {2:0.0######}))', $time, $tip.X, $tip.Y))
    }
    if ([Math]::Round($time, 7) -ge $last) { break }
  }
  [pscustomobject]@{ Wand = $wandFrames; Poses = $poses }
}
