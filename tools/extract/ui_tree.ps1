param([string]$Scene, [string]$GuidIndex, [string]$Under = '', [switch]$All)
# Lists UI images in a scene or prefab: path, sprite, anchored position and size (in the UI's
# reference pixels, relative to the parent) and anchors. Optional -Under keeps only paths that
# contain that text; -All also lists objects without an image (sprite '-'). Output: path<TAB>sprite<TAB>x<TAB>y<TAB>w<TAB>h<TAB>anchorMin<TAB>anchorMax
$idx = @{}; Get-Content $GuidIndex | ForEach-Object { $p = $_ -split "`t"; $idx[$p[0]] = $p[1] }
$text = [IO.File]::ReadAllText($Scene)
$docs = [regex]::Split($text, '(?m)^--- ') | Where-Object { $_ -match '^!u!(\d+) &(\d+)' }
$goName = @{}; $rtByGo = @{}; $rtById = @{}; $images = @()
foreach ($d in $docs) {
  $m = [regex]::Match($d, '^!u!(\d+) &(\d+)'); $type = $m.Groups[1].Value; $id = $m.Groups[2].Value
  if ($type -eq '1') { $goName[$id] = [regex]::Match($d, 'm_Name: (.*)').Groups[1].Value.Trim() }
  elseif ($type -eq '224' -or $type -eq '4') {
    $t = [pscustomobject]@{ id = $id; go = [regex]::Match($d, 'm_GameObject: \{fileID: (\d+)').Groups[1].Value
      parent = [regex]::Match($d, 'm_Father: \{fileID: (\d+)').Groups[1].Value
      pos = [regex]::Match($d, 'm_AnchoredPosition: \{([^}]*)\}').Groups[1].Value -replace '\s', ''
      size = [regex]::Match($d, 'm_SizeDelta: \{([^}]*)\}').Groups[1].Value -replace '\s', ''
      amin = [regex]::Match($d, 'm_AnchorMin: \{([^}]*)\}').Groups[1].Value -replace '\s', ''
      amax = [regex]::Match($d, 'm_AnchorMax: \{([^}]*)\}').Groups[1].Value -replace '\s', '' }
    $rtByGo[$t.go] = $t; $rtById[$id] = $t
  }
  elseif ($type -eq '114' -and $d -match 'm_Sprite: \{fileID: \d+, guid: ([0-9a-f]+)') {
    $images += [pscustomobject]@{ go = [regex]::Match($d, 'm_GameObject: \{fileID: (\d+)').Groups[1].Value; sprite = $Matches[1] }
  }
}
if ($All) { $seen = @{}; foreach ($im in $images) { $seen[$im.go] = 1 }
  foreach ($t in $rtById.Values) { if (-not $seen[$t.go]) { $images += [pscustomobject]@{ go = $t.go; sprite = "" } } } }
foreach ($im in $images) {
  $t = $rtByGo[$im.go]; if (-not $t) { continue }
  $names = @(); $cur = $t
  while ($cur) { $names = @($goName[$cur.go]) + $names; $cur = if ($cur.parent -and $cur.parent -ne '0') { $rtById[$cur.parent] } else { $null } }
  $path = $names -join '/'
  if ($Under -and $path -notlike "*$Under*") { continue }
  $sp = if ($idx[$im.sprite]) { [IO.Path]::GetFileNameWithoutExtension($idx[$im.sprite]) } else { '-' }
  "$path`t$sp`t$($t.pos)`t$($t.size)`t$($t.amin)`t$($t.amax)"
}
