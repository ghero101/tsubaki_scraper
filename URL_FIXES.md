# URL Fixes Applied

## Fixed Sources

### FireScans ✓
- **Old**: `/series?page=1` (404)
- **New**: `/manga/?page=1` (200, 13 titles)
- **File**: `sources/firescans.rs`
- **Status**: FIXED

## Sources Needing Custom URL Patterns

### Kenscans
- **Domain**: https://kenscans.com ✓
- **Pattern**: `/` (250 links) or `/series` (6 links)
- **Issue**: wp_manga uses `/manga/?page=` which returns 404
- **Solution**: Make wp_manga try multiple patterns

### QiScans  
- **Domain**: https://qiscans.com ✓
- **Pattern**: `/` (254 links) or `/series` (2 links)
- **Issue**: Same as kenscans
- **Solution**: Same

### NyxScans
- **Domain**: https://nyxscans.com ✓
- **Pattern**: `/` (255 links) or `/series` (5 links)
- **Issue**: Same as kenscans
- **Solution**: Same

### StoneScape
- **Domain**: https://stonescape.xyz ✓
- **Pattern**: `/series` (200 OK)
- **Issue**: wp_manga uses `/manga/?page=` which returns 404
- **Solution**: Same

### TempleScan
- **Domain**: https://templescan.net ✓
- **Pattern**: Need to investigate - home page has 0 manga links
- **Issue**: Might use JS loading or different structure
- **Solution**: Manual investigation needed

## Implementation Plan

### Phase 2A: Enhanced WP-Manga Pattern Support
Modify `sources/wp_manga.rs` to try multiple URL patterns:
1. `/manga/?page={}` (current default)
2. `/series?page={}` (for stonescape, kenscans alternatives)
3. `/` (for kenscans, qiscans, nyxscans home pages)

Add pattern detection:
- Try each pattern
- Use first one that returns manga
- Cache working pattern per domain

### Phase 2B: Individual Source Overrides
If needed, create custom implementations for:
- kenscans -> Use `/` with custom selectors
- qiscans -> Same
- nyxscans -> Same
- stonescape -> Use `/series`
- templescan -> Investigate JS/AJAX loading

## Testing Command
```bash
pwsh scripts/test_all_sources.ps1
```

Expected improvement: 2 -> 6+ working sources
