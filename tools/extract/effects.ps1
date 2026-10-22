param([string]$Repo)
# Adds spell-effect animations that reuse frames staged by inventory.ps1 (run it after that):
# vine_overlay_<P> grows over a brick in 5 frames at 25 fps (the original's Ivy<P> clips).
$X = Join-Path $Repo 'extracted'
$utf8 = New-Object System.Text.UTF8Encoding($false)
$m = [IO.File]::ReadAllLines((Join-Path $X 'manifest.ron'))
$have = @{}; foreach ($l in $m) { if ($l -match '^\s*"([^"]+)"') { $have[$Matches[1]] = 1 } }
$add = New-Object System.Collections.Generic.List[string]
$shapes = [ordered]@{ I = 'fx_ivy_i'; O = 'fx_ivy_o'; T = 'fx_ivy_t'; S = 'fx_ivy_s'; Z = 'fx_ivy_z'; J = 'fx_ivy_j'; L = 'fx_ivy_l'; mystery_block = 'brick_ivy_mystery01'; mystery_blob = 'brick_ivy_mystery02'; mystery_table = 'brick_ivy_mystery03' }
foreach ($k in $shapes.Keys) {
  $frames = 0..4 | ForEach-Object { "$($shapes[$k])_$_" }
  if (@($frames | Where-Object { -not $have.ContainsKey($_) }).Count -gt 0) { continue }
  $list = (0..4 | ForEach-Object { '({0:0.00}, "{1}")' -f ($_ * 0.04), $frames[$_] }) -join ', '
  $add.Add("    `"vine_overlay_$k`": (frames: [$($list -replace ',(\d)', '.$1')], length: 0.2, looping: false),")
}
if ($have.ContainsKey('vine_overlay_T') -eq $false -and $add.Count -gt 0) {
  # Plain name the game looks up today: the T version's last frame is a fair stand-in.
  $add.Add('    "vine_overlay": (file: "all/effects/fx_ivy_t_4.png"),')
}
# The wizard's turn (RotateSprite's Rotate clip): 4 frames, 1/18 s each, 0.222 s, once.
if ($have.ContainsKey('wizard_01_rotate_3') -and -not $have.ContainsKey('wizard_rotate')) {
  $add.Add('    "wizard_rotate": (frames: [(0.0, "wizard_01_rotate_0"), (0.0556, "wizard_01_rotate_1"), (0.1111, "wizard_01_rotate_2"), (0.1667, "wizard_01_rotate_3")], length: 0.2222, looping: false),')
}
$last = [Array]::LastIndexOf($m, '}')
$merged = @($m[0..($last - 1)]) + @('    // ---- effect animations, staged by effects.ps1 ----') + $add + @('}')
[IO.File]::WriteAllLines((Join-Path $X 'manifest.ron'), [string[]]$merged, $utf8)
"effects: $($add.Count) entries"
