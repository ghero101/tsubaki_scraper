# Test Phase 1 "No Data" fixes  
$phase1Sources = @(
    "rizzcomic",
    "asmotoon",
    "witchscans",
    "rokaricomics",
    "hivetoons"
)

Write-Host "`n=== Phase 1: Testing 'No Data' Fixes ===" -ForegroundColor Cyan
Write-Host "Sources: $($phase1Sources -join ', ')`n" -ForegroundColor Gray

$results = @()
foreach ($source in $phase1Sources) {
    Write-Host "Testing $source..." -NoNewline
    
    try {
        $response = curl -X GET "http://127.0.0.1:8080/import/source/$source/quick?limit=10&chapters=1" 2>$null | ConvertFrom-Json
        
        if ($response.error) {
            Write-Host " ERROR" -ForegroundColor Red
            $results += [PSCustomObject]@{
                Source = $source
                Status = "ERROR"
                Manga = 0
                Chapters = 0
                Error = $response.error.Substring(0, [Math]::Min(80, $response.error.Length))
            }
        }
        elseif ($response.manga_added -eq 0) {
            Write-Host " NO DATA" -ForegroundColor Yellow
            $results += [PSCustomObject]@{
                Source = $source
                Status = "NO DATA"
                Manga = 0
                Chapters = 0
                Error = ""
            }
        }
        else {
            Write-Host " ✓ ($($response.manga_added) manga, $($response.chapters_added) chapters)" -ForegroundColor Green
            $results += [PSCustomObject]@{
                Source = $source
                Status = "SUCCESS"
                Manga = $response.manga_added
                Chapters = $response.chapters_added
                Error = ""
            }
        }
    }
    catch {
        Write-Host " EXCEPTION" -ForegroundColor Red
        $results += [PSCustomObject]@{
            Source = $source
            Status = "EXCEPTION"
            Manga = 0
            Chapters = 0
            Error = $_.Exception.Message
        }
    }
    
    Start-Sleep -Milliseconds 500
}

Write-Host "`n=== Results ===" -ForegroundColor Cyan
$results | Format-Table -AutoSize

$success = ($results | Where-Object { $_.Status -eq "SUCCESS" }).Count
Write-Host "`nPhase 1 Success Rate: $success / $($results.Count)" -ForegroundColor $(if ($success -ge 4) { "Green" } elseif ($success -ge 3) { "Yellow" } else { "Red" })
