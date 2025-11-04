# Detailed source testing - fetches HTML to see what's actually being returned
param(
    [Parameter(Mandatory=$true)]
    [string]$Source,
    
    [string]$BaseUrl
)

$sourceUrls = @{
    "firescans" = "https://firescans.xyz/series?page=1"
    "rizzcomic" = "https://rizzcomic.com/manga/?page=1"
    "asmotoon" = "https://asmotoon.com/manga/?page=1"
    "drakecomic" = "https://drakecomic.org/manga/?page=1"
    "resetscans" = "https://reset-scans.org/manga/?page=1"
    "witchscans" = "https://witchscans.com/manga/?page=1"
    "rokaricomics" = "https://rokaricomics.com/manga/?page=1"
    "hivetoons" = "https://hivetoons.com/manga/?page=1"
    "manhuaus" = "https://manhuaus.com/manga/?page=1"
    "temple-scan" = "https://templescan.net/manga?page=1"
    "thunderscans" = "https://thunderscans.com/manga/?page=1"
    "asurascans" = "https://asurascans.com/manga/?page=1"
    "kenscans" = "https://kenscans.com/manga?page=1"
    "sirenscans" = "https://sirenscans.com/manga/?page=1"
    "vortexscans" = "https://vortexscans.com/manga/?page=1"
    "qiscans" = "https://qiscans.com/manga?page=1"
    "madarascans" = "https://madarascans.com/manga/?page=1"
    "rizzfables" = "https://rizzfables.com/manga/?page=1"
    "stonescape" = "https://stonescape.xyz/manga/?page=1"
    "grimscans" = "https://grimscans.team/manga/?page=1"
    "nyxscans" = "https://nyxscans.com/manga?page=1"
    "kagane" = "https://kagane.org"
    "mangadex" = "https://api.mangadex.org/manga?limit=10&includes[]=cover_art"
}

$url = if ($BaseUrl) { $BaseUrl } else { $sourceUrls[$Source.ToLower()] }

if (-not $url) {
    Write-Host "Unknown source: $Source" -ForegroundColor Red
    Write-Host "Available sources:" -ForegroundColor Yellow
    $sourceUrls.Keys | Sort-Object | ForEach-Object { Write-Host "  - $_" }
    exit 1
}

Write-Host "`n=== Testing Source: $Source ===" -ForegroundColor Cyan
Write-Host "URL: $url`n" -ForegroundColor Gray

Write-Host "Fetching..." -NoNewline

try {
    $response = Invoke-WebRequest -Uri $url -UserAgent "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0 Safari/537.36" -TimeoutSec 10 -ErrorAction Stop
    
    Write-Host " OK" -ForegroundColor Green
    Write-Host "Status: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "Content-Type: $($response.Headers['Content-Type'])" -ForegroundColor Gray
    Write-Host "Content-Length: $($response.Content.Length) bytes`n" -ForegroundColor Gray
    
    # Check for common manga listing elements
    $html = $response.Content
    
    Write-Host "=== HTML Analysis ===" -ForegroundColor Cyan
    
    # Common WP-Manga selectors
    $patterns = @{
        "WP-Manga div.page-item-detail" = "div\.page-item-detail"
        "WP-Manga h3 a" = "h3.*?<a[^>]*href"
        "Series cards" = "div\.series-card"
        "Series title link" = "a\.series-title"
        "Chapter list" = "(chapter-list|chapterlist|eplister)"
        "Manga cards" = "div\.manga-card"
        "Post items" = "div\.post-item"
    }
    
    foreach ($name in $patterns.Keys) {
        $pattern = $patterns[$name]
        if ($html -match $pattern) {
            $matches = ([regex]::Matches($html, $pattern)).Count
            Write-Host "✓ Found $name ($matches matches)" -ForegroundColor Green
        }
        else {
            Write-Host "✗ No $name found" -ForegroundColor Red
        }
    }
    
    # Save sample for inspection
    $sampleFile = "debug_$($Source)_sample.html"
    $html | Out-File $sampleFile -Encoding UTF8
    Write-Host "`n✓ Saved HTML sample to: $sampleFile" -ForegroundColor Yellow
    
    # Show first few titles if found
    Write-Host "`n=== Extracted Titles ===" -ForegroundColor Cyan
    $titlePatterns = @(
        '<h3[^>]*>.*?<a[^>]*href="([^"]*)"[^>]*>([^<]+)</a>',
        '<a[^>]*class="[^"]*series-title[^"]*"[^>]*href="([^"]*)"[^>]*>([^<]+)</a>',
        '<div[^>]*class="[^"]*post-title[^"]*"[^>]*>.*?<a[^>]*href="([^"]*)"[^>]*>([^<]+)</a>'
    )
    
    $found = 0
    foreach ($pattern in $titlePatterns) {
        $titleMatches = [regex]::Matches($html, $pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)
        if ($titleMatches.Count -gt 0) {
            Write-Host "Found $($titleMatches.Count) titles using pattern:" -ForegroundColor Green
            $titleMatches | Select-Object -First 5 | ForEach-Object {
                $title = $_.Groups[2].Value.Trim()
                $url = $_.Groups[1].Value
                Write-Host "  - $title" -ForegroundColor White
                Write-Host "    $url" -ForegroundColor Gray
            }
            $found = $titleMatches.Count
            break
        }
    }
    
    if ($found -eq 0) {
        Write-Host "No titles extracted with standard patterns" -ForegroundColor Yellow
    }
}
catch {
    Write-Host " FAILED" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    
    if ($_.Exception.InnerException) {
        Write-Host "Inner: $($_.Exception.InnerException.Message)" -ForegroundColor Red
    }
}
