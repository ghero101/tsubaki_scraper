param(
  [Parameter(Mandatory=$true)][int]$SourceId
)
$ErrorActionPreference='Stop'
$base = 'http://127.0.0.1:8080'
for($o=0; $o -lt 5000; $o+=50){
  $page = ((Invoke-WebRequest -UseBasicParsing "$base/manga?limit=50&offset=$o").Content | ConvertFrom-Json);
  foreach($m in $page.data){
    $chs = ((Invoke-WebRequest -UseBasicParsing "$base/manga/$($m.id)/chapters").Content | ConvertFrom-Json);
    $hit = $chs | Where-Object { $_.source_id -eq $SourceId } | Select-Object -First 1;
    if($null -ne $hit){
      $mi = ((Invoke-WebRequest -UseBasicParsing "$base/manga/$($m.id)").Content | ConvertFrom-Json);
      [pscustomobject]@{ id=$m.id; title=$mi.title; chapter=$hit.chapter_number; url=$hit.url } | ConvertTo-Json -Compress;
      exit 0
    }
  }
}
Write-Output '{}'
exit 2
