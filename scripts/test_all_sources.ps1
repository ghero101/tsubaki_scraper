# Test all manga sources systematically
# Tests quick import (10 manga, 1 chapter each) for each source

$sources = @(
    "mangadex",
    "firescans", 
    "rizzcomic",
    "drakecomic",
    "asmotoon",
    "resetscans",
    "kagane",
    "temple-scan",
    "thunderscans",
    # WP-Manga sites
    "asurascans",
    "kenscans",
    "sirenscans",
    "vortexscans",
    "witchscans",
    "qiscans",
    "madarascans",
    "rizzfables",
    "rokaricomics",
    "stonescape",
    "manhuaus",
    "grimscans",
    "hivetoons",
    "nyxscans"
)

$results = @()

Write-Host "`n=== Testing All Manga Sources ===" -ForegroundColor Cyan
Write-Host "Testing quick import (10 manga, 1 chapter each)`n" -ForegroundColor Gray

foreach ($source in $sources) {
    Write-Host "Testing: $source..." -NoNewline
    
    try {
        $response = curl -X GET "http://127.0.0.1:8080/import/source/$source/quick?limit=10&chapters=1" 2>$null | ConvertFrom-Json
        
        if ($response.error) {
            Write-Host " ERROR" -ForegroundColor Red
            $results += [PSCustomObject]@{
                Source = $source
                Status = "ERROR"
                Manga = 0
                Chapters = 0
                Error = $response.error
            }
        }
        elseif ($response.manga_added -eq 0) {
            Write-Host " NO DATA" -ForegroundColor Yellow
            $results += [PSCustomObject]@{
                Source = $source
                Status = "NO DATA"
                Manga = 0
                Chapters = 0
                Error = "No manga returned"
            }
        }
        else {
            Write-Host " OK ($($response.manga_added) manga, $($response.chapters_added) chapters)" -ForegroundColor Green
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
    
    # Small delay between requests
    Start-Sleep -Milliseconds 500
}

Write-Host "`n=== Summary ===" -ForegroundColor Cyan
$results | Format-Table -AutoSize

$success = ($results | Where-Object { $_.Status -eq "SUCCESS" }).Count
$total = $results.Count

Write-Host "`nSuccessful: $success / $total" -ForegroundColor $(if ($success -eq $total) { "Green" } else { "Yellow" })

# Show errors
$errors = $results | Where-Object { $_.Status -ne "SUCCESS" }
if ($errors.Count -gt 0) {
    Write-Host "`n=== Failed Sources ===" -ForegroundColor Red
    $errors | Format-Table Source, Status, Error -AutoSize
}
