# Iteration 2 Fixes - Source-Specific Selector Updates

## Problem Identified
firescans and rizzcomic have their own `search_manga_first_page` functions that weren't using the updated wp_manga selectors. They were still using old hardcoded selectors.

## Files Modified

### 1. sources/firescans.rs
**Issue**: Used hardcoded `div.series-card` selector  
**Fix**: Added multi-pattern selector support:
- `div.page-listing-item` + `h3 a` (MadaraProject theme - CURRENT)
- `div.series-card` + `a.series-title` (old layout fallback)
- `div.page-item-detail` + `h3 > a` (standard WP-Manga fallback)

Tries patterns in order, uses first that finds manga.

### 2. sources/rizzcomic.rs
**Issue**: Link selector `h3 > a, a.item-title, a.series-title` didn't match rizzcomic's structure  
**Fix**: 
- Added generic `a` selector to catch direct links
- Enhanced title extraction to try `title` attribute first (rizzcomic uses this)
- Added empty title check to filter invalid entries

## Expected Results After Restart

### Should Now Work:
- ✅ firescans (MadaraProject theme detected)
- ✅ rizzcomic (title attribute extraction + generic link selector)

### Already Working:
- ✅ resetscans
- ✅ manhuaus
- ✅ stonescape
- ✅ witchscans
- ✅ rokaricomics

### Total Expected: 7/21 sources (33%)

## Test Commands
```bash
curl "http://127.0.0.1:8080/import/source/firescans/quick?limit=10&chapters=1"
curl "http://127.0.0.1:8080/import/source/rizzcomic/quick?limit=10&chapters=1"
```

## Still TODO After This
- **asmotoon** - no data (needs investigation)
- **hivetoons** - no data (needs investigation)  
- **kenscans, qiscans, nyxscans** - 404 errors (need correct URLs)
- 9 sources with 403/DNS/SSL issues (lower priority)
