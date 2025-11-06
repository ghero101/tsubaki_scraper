# Test script to fetch HTML and see what selectors are present
param(
    [string]$Url = "https://qiscans.org/manga/?page=1"
)

$headers = @{
    "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
    "Accept" = "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"
    "Accept-Language" = "en-US,en;q=0.5"
}

try {
    Write-Host "Fetching $Url..." -ForegroundColor Cyan
    $response = Invoke-WebRequest -Uri $Url -Headers $headers -TimeoutSec 30 -UseBasicParsing
    
    Write-Host "Status: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "Content Length: $($response.Content.Length) bytes" -ForegroundColor Green
    
    # Check for common selectors
    $selectors = @(
        "div.page-item-detail",
        "div.page-listing-item",
        "div.listupd .bs .bsx",
        "div.bsx",
        "div.manga-item",
        "div.utao .uta .imgu",
        "article.bs",
        "div.post-item",
        "div.series-item"
    )
    
    Write-Host "`n=== Checking Selectors ===" -ForegroundColor Yellow
    foreach ($selector in $selectors) {
        $selectorTag = $selector -replace "div\.", "" -replace "article\.", ""
        $pattern = "class\s*=\s*[`"`'].*?$($selectorTag.Split()[0]).*?[`"`']"
        if ($response.Content -match $pattern) {
            Write-Host "✅ Found: $selector" -ForegroundColor Green
        } else {
            Write-Host "❌ Not found: $selector" -ForegroundColor Red
        }
    }
    
    # Look for any divs with "manga" or "item" in the class
    Write-Host "`n=== Looking for manga-related classes ===" -ForegroundColor Yellow
    $mangaClasses = [regex]::Matches($response.Content, 'class="([^"]*(?:manga|item|post|comic)[^"]*)"')
    $uniqueClasses = $mangaClasses | ForEach-Object { $_.Groups[1].Value } | Select-Object -Unique | Sort-Object
    
    if ($uniqueClasses.Count -gt 0) {
        Write-Host "Found $($uniqueClasses.Count) potential classes:" -ForegroundColor Green
        $uniqueClasses | ForEach-Object { Write-Host "  - $_" }
    } else {
        Write-Host "No manga-related classes found" -ForegroundColor Red
    }
    
    # Save HTML to file for manual inspection
    $fileName = "html_dump_$(Get-Date -Format 'yyyyMMdd_HHmmss').html"
    $response.Content | Out-File -FilePath $fileName -Encoding UTF8
    Write-Host "`nHTML saved to: $fileName" -ForegroundColor Cyan
}
catch {
    Write-Host "Error: $_" -ForegroundColor Red
    Write-Host "Status Code: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Red
}
