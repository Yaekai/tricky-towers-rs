# Samples the CastSelf wand in wizard-root space. The prefab's intermediate transforms
# are identity in XY. CastSelf animates the arm translation and the inner Wizard root;
# the guest clips also key the wand's own translation.

function Read-WandVector($value) {
  $culture = [Globalization.CultureInfo]::InvariantCulture
  $components = [regex]::Matches($value, '[xyz]: ([-+\d.eE]+)')
  ,@($components | ForEach-Object { [double]::Parse($_.Groups[1].Value, $culture) })
}

function Sample-WandCurve($keys, [double]$time, $fallback) {
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
    $point = for ($axis = 0; $axis -lt 3; $axis++) {
      (2 * $u3 - 3 * $u2 + 1) * $left.Value[$axis] +
        ($u3 - 2 * $u2 + $u) * $span * $left.OutSlope[$axis] +
        (-2 * $u3 + 3 * $u2) * $right.Value[$axis] +
        ($u3 - $u2) * $span * $right.InSlope[$axis]
    }
    return ,$point
  }
  return ,$keys[-1].Value
}

function Read-CastWand($prefab, $clip) {
  if (!(Test-Path -LiteralPath $clip)) { return }
  $culture = [Globalization.CultureInfo]::InvariantCulture
  $prefabText = [IO.File]::ReadAllText($prefab)
  $wandObject = [regex]::Match($prefabText, '(?ms)^--- !u!1 &(\d+)\r?\n(?:(?!^--- ).)*?^  m_Name: SpellCastContainer\r?$')
  if (!$wandObject.Success) { return }
  $wandId = $wandObject.Groups[1].Value
  $wandTransform = [regex]::Match($prefabText, '(?ms)^--- !u!4 &\d+\r?\n(?:(?!^--- ).)*?^  m_GameObject: \{fileID: ' + $wandId + '\}\r?\n(?:(?!^--- ).)*?^  m_LocalPosition: \{([^}]+)\}')
  if (!$wandTransform.Success) { return }
  $wandOffset = Read-WandVector $wandTransform.Groups[1].Value

  $text = [IO.File]::ReadAllText($clip)
  $tracks = @{}
  $vectorKey = 'time: ([-+\d.eE]+)\s+value: \{([^}]+)\}\s+inSlope: \{([^}]+)\}\s+outSlope: \{([^}]+)\}'
  foreach ($section in [regex]::Matches($text, '(?ms)^  m_(Position|Scale|Euler)Curves:\r?\n(.*?)(?=^  m_)')) {
    $kind = $section.Groups[1].Value
    foreach ($curve in [regex]::Split($section.Groups[2].Value, '(?m)^  - curve:') | Select-Object -Skip 1) {
      $path = [regex]::Match($curve, '(?m)^    path: ?([^\r\n]*)').Groups[1].Value
      if ($path -notin @('', 'ArmBack', 'ArmBack/SpellCastContainer')) { continue }
      $tracks["${kind}:$path"] = @([regex]::Matches($curve, $vectorKey) | ForEach-Object {
        [pscustomobject]@{
          Time = [double]::Parse($_.Groups[1].Value, $culture)
          Value = Read-WandVector $_.Groups[2].Value
          InSlope = Read-WandVector $_.Groups[3].Value
          OutSlope = Read-WandVector $_.Groups[4].Value
        }
      })
    }
  }
  $length = [double]::Parse([regex]::Match($text, 'm_StopTime: ([-+\d.eE]+)').Groups[1].Value, $culture)
  # Keep the curved motion between the clip's sprite frames.
  $rate = 4 * [double]::Parse([regex]::Match($text, 'm_SampleRate: ([-+\d.eE]+)').Groups[1].Value, $culture)
  $zero = @(0.0, 0.0, 0.0)
  $one = @(1.0, 1.0, 1.0)
  for ($frame = 0; $frame -le [Math]::Ceiling($length * $rate); $frame++) {
    $time = [Math]::Min($frame / $rate, $length)
    $arm = Sample-WandCurve $tracks['Position:ArmBack'] $time $zero
    $wand = Sample-WandCurve $tracks['Position:ArmBack/SpellCastContainer'] $time $wandOffset
    $root = Sample-WandCurve $tracks['Position:'] $time $zero
    $scale = Sample-WandCurve $tracks['Scale:'] $time $one
    $turn = Sample-WandCurve $tracks['Euler:'] $time $zero
    $angle = $turn[2] * [Math]::PI / 180
    $x = ($arm[0] + $wand[0]) * $scale[0]
    $y = ($arm[1] + $wand[1]) * $scale[1]
    $px = $root[0] + $x * [Math]::Cos($angle) - $y * [Math]::Sin($angle)
    $py = $root[1] + $x * [Math]::Sin($angle) + $y * [Math]::Cos($angle)
    [string]::Format($culture, '({0:0.0######}, ({1:0.0######}, {2:0.0######}))', $time, $px, $py)
    if ($time -ge $length) { break }
  }
}
