param(
  [Parameter(Mandatory=$true)][string]$Source
)
$ErrorActionPreference='Stop'
$base = 'http://127.0.0.1:8080'
# Map name/id to numeric id
switch -Regex ($Source.ToLower()){
  '^(2|firescans)$'   { $sid = 2; break }
  '^(3|rizzcomic)$'   { $sid = 3; break }
  '^(6|drakecomic)$'  { $sid = 6; break }
  '^(8|asmotoon)$'    { $sid = 8; break }
  '^(9|reset-?scans)$'{ $sid = 9; break }
  '^(10|kagane)$'     { $sid = 10; break }
  default { Write-Output "unknown source: $Source"; exit 1 }
}

function GetPage([int]$offset){ ((Invoke-WebRequest -UseBasicParsing "$base/manga?limit=50&offset=$offset").Content | ConvertFrom-Json).data }
function GetChs([string]$id){ ((Invoke-WebRequest -UseBasicParsing "$base/manga/$id/chapters").Content | ConvertFrom-Json) }
function GetTitle([string]$id){ ((Invoke-WebRequest -UseBasicParsing "$base/manga/$id").Content | ConvertFrom-Json).title }
function Download([string]$mid,[string]$chap,[int]$sid){ ((Invoke-WebRequest -UseBasicParsing "$base/download/$mid/$([uri]::EscapeDataString($chap))/$sid").Content | ConvertFrom-Json) }

$found = $null
foreach($o in 0..1000){ $offset=$o*50; $list = GetPage $offset; foreach($m in $list){ $chs = GetChs $m.id; $hit = $chs | Where-Object { $_.source_id -eq $sid } | Select-Object -First 1; if($hit){ $found = [pscustomobject]@{ mid=$m.id; chapter=$hit.chapter_number }; break } } if($found){ break } }
if(-not $found){ Write-Output "no chapters found for source $sid"; exit 2 }
$title = GetTitle $found.mid
$dl = Download $found.mid $found.chapter $sid
if($dl.file){ $file = Join-Path (Get-Location) $dl.file; if(Test-Path $file){ (Get-Item $file | Select-Object Name,Length) | Format-Table -AutoSize } ; Write-Output ("source=$sid`t"+"manga=$title`t"+"chapter=$($found.chapter)"+"`tfile=$($dl.file)") } else { $dl | ConvertTo-Json -Compress }
