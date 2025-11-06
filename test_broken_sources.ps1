# Test all broken sources
$sources = @(
    "qiscans",
    "templetoons",
    "asmotoon",
    "hivetoons",
    "asurascans",
    "grimscans",
    "kenscans",
    "rizzfables",
    "sirenscans",
    "thunderscans",
    "vortexscans",
    "drakecomic",
    "madarascans",
    "nyxscans"
)

$results = @()

foreach ($source in $sources) {
    Write-Host "Testing $source..." -ForegroundColor Cyan
    try {
        $response = Invoke-WebRequest -Uri "http://127.0.0.1:8080/import/source/$source/quick?limit=5&chapters=1" -TimeoutSec 60 -UseBasicParsing
        $content = $response.Content | ConvertFrom-Json
        
        if ($content.manga_count -gt 0) {
            $status = "✅ WORKING"
            $color = "Green"
        } else {
            $status = "⚠️  NO DATA"
            $color = "Yellow"
        }
        
        $results += [PSCustomObject]@{
            Source = $source
            Status = $status
            MangaCount = $content.manga_count
            ChapterCount = $content.chapter_count
        }
        Write-Host "$source : $status ($($content.manga_count) manga, $($content.chapter_count) chapters)" -ForegroundColor $color
    }
    catch {
        $errorMsg = $_.Exception.Message
        if ($errorMsg -match "403") {
            $status = "❌ 403 FORBIDDEN"
            $color = "Red"
        }
        elseif ($errorMsg -match "404") {
            $status = "❌ 404 NOT FOUND"
            $color = "Red"
        }
        elseif ($errorMsg -match "503") {
            $status = "❌ 503 SERVICE UNAVAILABLE"
            $color = "Red"
        }
        elseif ($errorMsg -match "DNS") {
            $status = "❌ DNS ERROR"
            $color = "Red"
        }
        else {
            $status = "❌ ERROR"
            $color = "Red"
        }
        
        $results += [PSCustomObject]@{
            Source = $source
            Status = $status
            MangaCount = 0
            ChapterCount = 0
        }
        Write-Host "$source : $status - $errorMsg" -ForegroundColor $color
    }
    
    Start-Sleep -Milliseconds 500
}

Write-Host "`n=== SUMMARY ===" -ForegroundColor Magenta
$results | Format-Table -AutoSize

$working = ($results | Where-Object { $_.Status -eq "✅ WORKING" }).Count
$total = $results.Count
Write-Host "`nWorking: $working / $total" -ForegroundColor Cyan
