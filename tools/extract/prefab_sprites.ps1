param([string]$Prefab, [string]$GuidIndex)
# Lists each SpriteRenderer in a prefab with its GameObject's local position/scale and parent chain.
$idx = @{}; Get-Content $GuidIndex | ForEach-Object { $p = $_ -split "`t"; $idx[$p[0]] = $p[1] }
$text = Get-Content $Prefab -Raw
$docs = [regex]::Split($text, '(?m)^--- ') | Where-Object { $_ -match '^!u!(\d+) &(\d+)' }
$go = @{}; $tr = @{}; $sr = @()
foreach ($d in $docs) {
  $m = [regex]::Match($d, '^!u!(\d+) &(\d+)'); $type = $m.Groups[1].Value; $id = $m.Groups[2].Value
  switch ($type) {
    '1'   { $go[$id] = [regex]::Match($d, 'm_Name: (.*)').Groups[1].Value.Trim() }
    '4'   { $tr[[regex]::Match($d, 'm_GameObject: \{fileID: (\d+)').Groups[1].Value] = [pscustomobject]@{
              id = $id
              pos = [regex]::Match($d, 'm_LocalPosition: (\{[^}]*\})').Groups[1].Value
              scale = [regex]::Match($d, 'm_LocalScale: (\{[^}]*\})').Groups[1].Value
              parent = [regex]::Match($d, 'm_Father: \{fileID: (\d+)').Groups[1].Value } }
    '212' { $sr += [pscustomobject]@{ go = [regex]::Match($d, 'm_GameObject: \{fileID: (\d+)').Groups[1].Value
              sprite = [regex]::Match($d, 'm_Sprite: \{fileID: \d+, guid: ([0-9a-f]+)').Groups[1].Value
              order = [regex]::Match($d, 'm_SortingOrder: (-?\d+)').Groups[1].Value } }
  }
}
foreach ($s in $sr) {
  $t = $tr[$s.go]; $name = $go[$s.go]
  $sp = if ($s.sprite -and $idx[$s.sprite]) { Split-Path -Leaf $idx[$s.sprite] } else { '?' }
  $parentGo = ($tr.GetEnumerator() | Where-Object { $_.Value.id -eq $t.parent } | Select-Object -First 1).Key
  "  [$name] sprite=$sp pos=$($t.pos) scale=$($t.scale) order=$($s.order) parent=$($go[$parentGo])"
}
"  root transforms: " + (($tr.GetEnumerator() | Where-Object { $_.Value.parent -eq '0' } | ForEach-Object { "$($go[$_.Key]) $($_.Value.pos)" }) -join '; ')
