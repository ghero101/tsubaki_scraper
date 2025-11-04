param([Parameter(Mandatory=$true)][string]$Path)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead($Path)
try {
  $has = $false
  foreach($e in $zip.Entries){ if($e.FullName -ieq 'ComicInfo.xml'){ $has=$true; break } }
  if($has){
    $tmp = Join-Path $env:TEMP ([System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Force -Path $tmp | Out-Null
    [System.IO.Compression.ZipFileExtensions]::ExtractToFile($zip.GetEntry('ComicInfo.xml'), (Join-Path $tmp 'ComicInfo.xml'), $true)
    Get-Content (Join-Path $tmp 'ComicInfo.xml') -TotalCount 20 | Out-String
  } else {
    'NO_COMICINFO'
  }
}
finally { $zip.Dispose() }
