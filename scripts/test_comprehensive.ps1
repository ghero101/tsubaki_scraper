# Comprehensive Source Testing - All Phases
# Shows impact of fixes across all sources

Write-Host "`n╔═══════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   Comprehensive Source Testing - Phases 1-3          ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════════╝`n" -ForegroundColor Cyan

$allSources = @(
    # Already working (baseline)
    @{name="resetscans"; phase="Baseline"; expected="SUCCESS"},
    @{name="manhuaus"; phase="Baseline"; expected="SUCCESS"},
    
    # Phase 2 - URL fixes
    @{name="firescans"; phase="Phase 2"; expected="SUCCESS"},
    @{name="kenscans"; phase="Phase 2"; expected="SUCCESS"},
    @{name="qiscans"; phase="Phase 2"; expected="SUCCESS"},
    @{name="nyxscans"; phase="Phase 2"; expected="SUCCESS"},
    @{name="stonescape"; phase="Phase 2"; expected="SUCCESS"},
    
    # Phase 1 - Selector fixes
    @{name="rizzcomic"; phase="Phase 1"; expected="SUCCESS"},
    @{name="asmotoon"; phase="Phase 1"; expected="SUCCESS"},
    @{name="witchscans"; phase="Phase 1"; expected="SUCCESS"},
    @{name="rokaricomics"; phase="Phase 1"; expected="SUCCESS"},
    @{name="hivetoons"; phase="Phase 1"; expected="SUCCESS"},
    
    # Still broken (known issues)
    @{name="drakecomic"; phase="TODO"; expected="ERROR"},
    @{name="madarascans"; phase="TODO"; expected="ERROR"},
    @{name="thunderscans"; phase="TODO"; expected="ERROR"},
    @{name="asurascans"; phase="TODO"; expected="ERROR"},
    @{name="sirenscans"; phase="TODO"; expected="ERROR"},
    @{name="vortexscans"; phase="TODO"; expected="ERROR"},
    @{name="grimscans"; phase="TODO"; expected="ERROR"},
    @{name="rizzfables"; phase="TODO"; expected="ERROR"},
    @{name="templescan"; phase="TODO"; expected="ERROR"}
)

$phaseResults = @{
    "Baseline" = @{expected=2; actual=0}
    "Phase 1" = @{expected=5; actual=0}
    "Phase 2" = @{expected=5; actual=0}
    "TODO" = @{expected=0; actual=0}
}

$results = @()
$total = $allSources.Count
$current = 0

foreach ($s in $allSources) {
    $current++
    $pct = [math]::Round(($current / $total) * 100)
    Write-Progress -Activity "Testing Sources" -Status "$current/$total - $($s.name)" -PercentComplete $pct
    
    $phase = $s.phase
    $source = $s.name
    
    try {
        $response = curl -X GET "http://127.0.0.1:8080/import/source/$source/quick?limit=10&chapters=1" 2>$null | ConvertFrom-Json
        
        if ($response.error) {
            $status = "ERROR"
            $manga = 0
            $chapters = 0
            $error = $response.error.Substring(0, [Math]::Min(60, $response.error.Length))
        }
        elseif ($response.manga_added -eq 0) {
            $status = "NO DATA"
            $manga = 0
            $chapters = 0
            $error = ""
        }
        else {
            $status = "SUCCESS"
            $manga = $response.manga_added
            $chapters = $response.chapters_added
            $error = ""
            $phaseResults[$phase].actual++
        }
    }
    catch {
        $status = "EXCEPTION"
        $manga = 0
        $chapters = 0
        $error = $_.Exception.Message.Substring(0, [Math]::Min(60, $_.Exception.Message.Length))
    }
    
    $results += [PSCustomObject]@{
        Source = $source
        Phase = $phase
        Status = $status
        Manga = $manga
        Chapters = $chapters
        Expected = $s.expected
        Match = if ($status -eq $s.expected) {"✓"} else {"✗"}
    }
    
    Start-Sleep -Milliseconds 300
}

Write-Progress -Activity "Testing Sources" -Completed

# Display results grouped by phase
Write-Host "`n╔═══════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                    RESULTS BY PHASE                   ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════════╝`n" -ForegroundColor Cyan

foreach ($phaseName in @("Baseline", "Phase 2", "Phase 1", "TODO")) {
    $phaseItems = $results | Where-Object { $_.Phase -eq $phaseName }
    $phaseSuccess = ($phaseItems | Where-Object { $_.Status -eq "SUCCESS" }).Count
    $phaseTotal = $phaseItems.Count
    $expected = $phaseResults[$phaseName].expected
    
    $color = if ($phaseSuccess -ge $expected) { "Green" } elseif ($phaseSuccess -gt 0) { "Yellow" } else { "Red" }
    
    Write-Host "`n=== $phaseName ===" -ForegroundColor Cyan
    Write-Host "Expected: $expected working | Actual: $phaseSuccess / $phaseTotal" -ForegroundColor $color
    
    $phaseItems | Format-Table Source, Status, Manga, Chapters, Match -AutoSize
}

# Overall summary
$totalSuccess = ($results | Where-Object { $_.Status -eq "SUCCESS" }).Count
$totalExpected = ($allSources | Where-Object { $_.expected -eq "SUCCESS" }).Count

Write-Host "`n╔═══════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                   OVERALL SUMMARY                     ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════════╝`n" -ForegroundColor Cyan

Write-Host "Total Sources Tested:    $total" -ForegroundColor Gray
Write-Host "Expected Working:        $totalExpected" -ForegroundColor Gray
Write-Host "Actually Working:        $totalSuccess" -ForegroundColor $(if ($totalSuccess -ge $totalExpected) { "Green" } else { "Yellow" })
Write-Host "Success Rate:            $([math]::Round(($totalSuccess/$total)*100))%" -ForegroundColor $(if ($totalSuccess -ge 12) { "Green" } elseif ($totalSuccess -ge 7) { "Yellow" } else { "Red" })
Write-Host ""

if ($totalSuccess -lt $totalExpected) {
    Write-Host "⚠ Some expected sources are not working. Server may need restart." -ForegroundColor Yellow
}
elseif ($totalSuccess -ge $totalExpected) {
    Write-Host "✓ All targeted sources are working!" -ForegroundColor Green
}

# Show failures
$failures = $results | Where-Object { $_.Status -ne "SUCCESS" -and $_.Expected -eq "SUCCESS" }
if ($failures.Count -gt 0) {
    Write-Host "`n=== Unexpected Failures ===" -ForegroundColor Red
    $failures | Format-Table Source, Phase, Status -AutoSize
}
