param([string]$Assets, [string]$Out)
# guid -> asset path for every .meta under Assets (sprites, textures, audio, prefabs).
$map = @{}
Get-ChildItem $Assets -Recurse -Filter *.meta | ForEach-Object {
  $line = Select-String -Path $_.FullName -Pattern '^guid: ([0-9a-f]+)' -List
  if ($line) { $map[$line.Matches[0].Groups[1].Value] = $_.FullName.Substring(0, $_.FullName.Length - 5) }
}
$map.GetEnumerator() | ForEach-Object { "$($_.Key)`t$($_.Value)" } | Set-Content -Encoding utf8 $Out
"indexed $($map.Count)"
