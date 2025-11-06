# API Endpoint Discovery Script for Next.js Sites
param(
    [string]$Domain = "qiscans.org"
)

$headers = @{
    "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
    "Accept" = "application/json, text/plain, */*"
    "Accept-Language" = "en-US,en;q=0.5"
}

Write-Host "=== API Endpoint Discovery for $Domain ===" -ForegroundColor Cyan

# Common API patterns to test
$apiPatterns = @(
    "/api/series",
    "/api/manga",
    "/api/comics",
    "/api/v1/series",
    "/api/v1/manga",
    "/api/series/all",
    "/api/manga/list",
    "/api/series?page=1",
    "/api/manga?page=1",
    "/series",
    "/manga"
)

$workingEndpoints = @()

foreach ($pattern in $apiPatterns) {
    $url = "https://$Domain$pattern"
    Write-Host "`nTrying: $url" -ForegroundColor Yellow
    
    try {
        $response = Invoke-WebRequest -Uri $url -Headers $headers -TimeoutSec 10 -UseBasicParsing -ErrorAction Stop
        
        if ($response.StatusCode -eq 200) {
            $contentType = $response.Headers["Content-Type"]
            $contentLength = $response.Content.Length
            
            Write-Host "✅ SUCCESS!" -ForegroundColor Green
            Write-Host "   Status: $($response.StatusCode)" -ForegroundColor Green
            Write-Host "   Content-Type: $contentType" -ForegroundColor Green
            Write-Host "   Content-Length: $contentLength bytes" -ForegroundColor Green
            
            # Check if it's JSON
            if ($contentType -like "*json*") {
                Write-Host "   Format: JSON ✓" -ForegroundColor Green
                try {
                    $json = $response.Content | ConvertFrom-Json
                    Write-Host "   JSON Structure:" -ForegroundColor Cyan
                    $json | ConvertTo-Json -Depth 2 -Compress | Write-Host
                }
                catch {
                    Write-Host "   (Could not parse JSON)" -ForegroundColor Yellow
                }
            }
            
            $workingEndpoints += [PSCustomObject]@{
                Endpoint = $pattern
                Status = $response.StatusCode
                ContentType = $contentType
                Size = $contentLength
            }
            
            # Save response for manual inspection
            $filename = "api_response_$($pattern -replace '/', '_' -replace '\?', '_').json"
            $response.Content | Out-File -FilePath $filename -Encoding UTF8
            Write-Host "   Saved to: $filename" -ForegroundColor Cyan
        }
    }
    catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        if ($statusCode) {
            Write-Host "❌ $statusCode" -ForegroundColor Red
        }
        else {
            Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
        }
    }
    
    Start-Sleep -Milliseconds 500
}

Write-Host "`n=== Summary ===" -ForegroundColor Magenta
if ($workingEndpoints.Count -gt 0) {
    Write-Host "Found $($workingEndpoints.Count) working endpoint(s):" -ForegroundColor Green
    $workingEndpoints | Format-Table -AutoSize
}
else {
    Write-Host "No API endpoints found. This site might:" -ForegroundColor Yellow
    Write-Host "  1. Use GraphQL instead of REST" -ForegroundColor Yellow
    Write-Host "  2. Have API endpoints behind authentication" -ForegroundColor Yellow  
    Write-Host "  3. Load data via SSR/ISR without exposed APIs" -ForegroundColor Yellow
    Write-Host "  4. Require headless browser for scraping" -ForegroundColor Yellow
}

# Try to find GraphQL endpoint
Write-Host "`n=== Checking for GraphQL ===" -ForegroundColor Cyan
$graphqlEndpoints = @("/graphql", "/api/graphql", "/gql")

foreach ($gql in $graphqlEndpoints) {
    $url = "https://$Domain$gql"
    try {
        $body = '{"query":"{ __typename }"}'
        $response = Invoke-WebRequest -Uri $url -Method POST -Headers (@{
            "User-Agent" = $headers["User-Agent"]
            "Content-Type" = "application/json"
        }) -Body $body -TimeoutSec 5 -UseBasicParsing -ErrorAction Stop
        
        Write-Host "✅ GraphQL found at: $url" -ForegroundColor Green
        Write-Host "   Response: $($response.Content.Substring(0, [Math]::Min(200, $response.Content.Length)))" -ForegroundColor Cyan
    }
    catch {
        # Silently ignore GraphQL endpoint failures
    }
}

Write-Host "`n=== Next Steps ===" -ForegroundColor Magenta
Write-Host "1. Check saved JSON files for data structure"
Write-Host "2. Look for pagination parameters (page, limit, offset)"
Write-Host "3. Identify manga/series data fields (title, id, url, cover)"
Write-Host "4. Implement Rust client for discovered endpoints"
