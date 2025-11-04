param()
$ErrorActionPreference = 'Stop'
$base = 'http://127.0.0.1:8080'
$targets = @(
  [pscustomobject]@{ id = 2;  name = 'FireScans' },
  [pscustomobject]@{ id = 3;  name = 'RizzComic' },
  [pscustomobject]@{ id = 6;  name = 'DrakeComic' },
  [pscustomobject]@{ id = 8;  name = 'Asmotoon' },
  [pscustomobject]@{ id = 9;  name = 'ResetScans' },
  [pscustomobject]@{ id = 10; name = 'Kagane' }
)
$used = @{}
$results = @()

function Get-MangaPage($offset){
  $uri = "$base/manga?limit=50&offset=$offset"
  ((Invoke-WebRequest -UseBasicParsing $uri).Content | ConvertFrom-Json)
}
function Get-MangaInfo($id){
  ((Invoke-WebRequest -UseBasicParsing "$base/manga/$id").Content | ConvertFrom-Json)
}
function Get-Chapters($id){
  ((Invoke-WebRequest -UseBasicParsing "$base/manga/$id/chapters").Content | ConvertFrom-Json)
}
function Download-Chapter($mid,$chap,$sid){
  ((Invoke-WebRequest -UseBasicParsing "$base/download/$mid/$([uri]::EscapeDataString($chap))/$sid").Content | ConvertFrom-Json)
}

foreach($t in $targets){
  $found = $false
  for($o=0; $o -lt 2700 -and -not $found; $o+=50){
    $page = Get-MangaPage -offset $o
    foreach($m in $page.data){
      if($used.ContainsKey($m.id)){ continue }
      $mi = Get-MangaInfo -id $m.id
      if(-not ($mi.sources | Where-Object { $_.source_id -eq $t.id })){ continue }
      $chs = Get-Chapters -id $m.id
      $hit = $chs | Where-Object { $_.source_id -eq $t.id } | Select-Object -First 1
      if($null -ne $hit){
        try {
          $dl = Download-Chapter -mid $m.id -chap $hit.chapter_number -sid $t.id
          $file = $dl.file
        } catch { $file = '(download failed)' }
        $results += [pscustomobject]@{
          source_id = $t.id; source = $t.name; manga = $mi.title; chapter = $hit.chapter_number; file = $file
        }
        $used[$m.id] = $true
        $found = $true
        break
      }
    }
  }
  if(-not $found){
    $results += [pscustomobject]@{ source_id = $t.id; source=$t.name; manga=''; chapter=''; file='(not found)' }
  }
}

$results | Format-Table -AutoSize
