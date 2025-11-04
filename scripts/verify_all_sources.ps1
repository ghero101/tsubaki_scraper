param()
$ErrorActionPreference = 'Stop'
$base = 'http://127.0.0.1:8080'

# Source name hints; include enum and generic wp names
$sources = @(
  'mangadex','firescans','rizzcomic','drakecomic','asmotoon','reset-scans','kagane','temple-scan','thunderscans',
  'asurascans','kenscans','sirenscans','vortexscans','witchscans','qiscans','madarascans','rizzfables','rokaricomics','stonescape','manhuaus','grimscans','hivetoons','nyxscans'
)

function Get-MangaPage($offset){ ((Invoke-WebRequest -UseBasicParsing "$base/manga?limit=50&offset=$offset").Content | ConvertFrom-Json) }
function Get-Manga($id){ ((Invoke-WebRequest -UseBasicParsing "$base/manga/$id").Content | ConvertFrom-Json) }
function Get-Chapters($id){ ((Invoke-WebRequest -UseBasicParsing "$base/manga/$id/chapters").Content | ConvertFrom-Json) }
function Import-Source($name){ (Invoke-WebRequest -UseBasicParsing "$base/import/source/$name/manga").Content | Out-Null }

$results = @()
foreach($s in $sources){
  try {
    Import-Source $s | Out-Null
  } catch {}
  $found = $null
  for($o=0; $o -lt 5000 -and -not $found; $o+=50){
    $page = Get-MangaPage $o
    foreach($m in $page.data){
      $chs = Get-Chapters $m.id
      $hit = $chs | Where-Object { $_.source_name -match $s -or $_.source_name -match $s.Replace('-','') } | Select-Object -First 1
      if($hit){ $found = [pscustomobject]@{ source=$s; mid=$m.id; title=$m.title; chapter=$hit.chapter_number; url=$hit.url }; break }
    }
  }
  if($found){ $results += $found } else { $results += [pscustomobject]@{ source=$s; mid=''; title=''; chapter=''; url='' } }
}

$results | Format-Table -AutoSize