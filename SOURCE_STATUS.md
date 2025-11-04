# Manga Source Status & Fix Plan

## Test Results (2024-11-04)

### Working Sources (2/23 tested)
- ✅ **resetscans** - 21 titles, 10 chapters tested
- ✅ **manhuaus** - 15 titles, 10 chapters tested

### Broken Sources

#### URL Changed / 404 Errors
- ❌ **firescans** - 404 on https://firescans.xyz/series?page=1
  - Action: Try different URL patterns (/manga/, /comics/, /series/)
- ❌ **temple-scan** - 404 on https://templescan.net/manga?page=1
  - Action: Verify correct domain (templetoons.com vs templescan.net)
- ❌ **kenscans** - 404 on https://kenscans.com/manga?page=1
  - Action: Try https://kencomics.com
- ❌ **qiscans** - 404 on https://qiscans.com/manga?page=1
  - Action: Try https://qiscans.org
- ❌ **stonescape** - 404 on https://stonescape.xyz/manga/?page=1
  - Action: Verify domain is still active
- ❌ **nyxscans** - 404 on https://nyxscans.com/manga?page=1
  - Action: Verify URL structure

#### Anti-Bot / 403 Forbidden
- ❌ **drakecomic** - 403 on https://drakecomic.org/manga/?page=1
  - Action: Add better headers, cookies, delay between requests
- ❌ **madarascans** - 403 on https://madarascans.com/manga/?page=1
  - Action: Cloudflare protection, need better UA/headers
- ❌ **rizzfables** - 403 on https://teksishe.net/4/6973620
  - Action: Wrong URL format entirely

#### DNS / Connection Errors  
- ❌ **thunderscans** - DNS error on thunderscans.com
  - Action: Verify domain is still registered
- ❌ **asurascans** - SSL/TLS error on asurascans.com
  - Action: Check certificate, try http, or domain changed
- ❌ **sirenscans** - SSL/TLS error on sirenscans.com
  - Action: Same as asurascans
- ❌ **vortexscans** - SSL/TLS error on vortexscans.org
  - Action: Same as asurascans
- ❌ **grimscans** - DNS error on grimscans.team
  - Action: Domain likely down/moved

#### No Data Returned (Empty Scrape)
- ⚠️ **rizzcomic** - Fetches but returns 0 manga
  - Action: Debug HTML structure, check selectors
- ⚠️ **asmotoon** - Fetches but returns 0 manga
  - Action: Debug HTML structure
- ⚠️ **witchscans** - Fetches but returns 0 manga
  - Action: Debug HTML structure
- ⚠️ **rokaricomics** - Fetches but returns 0 manga
  - Action: Debug HTML structure
- ⚠️ **hivetoons** - Fetches but returns 0 manga
  - Action: Debug HTML structure

#### Quick Endpoint Doesn't Support
- ⚠️ **mangadex** - "unknown or non-wp source"
  - Action: Add mangadex support to quick endpoint
- ⚠️ **kagane** - "unknown or non-wp source"
  - Action: Add kagane support to quick endpoint

## Fix Priority

### Phase 1: Easy Wins (No Data sources)
Test these 5 sources with detail script to see HTML structure:
1. rizzcomic
2. asmotoon
3. witchscans
4. rokaricomics
5. hivetoons

### Phase 2: URL Fixes (404 errors)
6. firescans - Try different URL patterns
7. temple-scan - Verify domain
8. kenscans - Wrong domain
9. qiscans - Wrong domain
10. stonescape - Verify active
11. nyxscans - Check URL

### Phase 3: Anti-Bot Bypass (403 errors)
12. drakecomic - Better headers
13. madarascans - Cloudflare bypass
14. rizzfables - Fix URL format

### Phase 4: Infrastructure Issues (DNS/SSL)
15. thunderscans - Domain check
16. asurascans - Certificate/domain check
17. sirenscans - Certificate check
18. vortexscans - Certificate check
19. grimscans - Domain down

### Phase 5: Quick Endpoint Enhancement
20. Add mangadex to quick endpoint
21. Add kagane to quick endpoint

## Testing Commands

Test individual source:
```bash
curl "http://127.0.0.1:8080/import/source/{source}/quick?limit=10&chapters=1"
```

Test HTML structure:
```bash
pwsh scripts/test_source_detail.ps1 -Source {source}
```

Test all sources:
```bash
pwsh scripts/test_all_sources.ps1
```

## Success Criteria

Goal: 20+ working sources out of 23 tested
- Phase 1: +5 sources = 7 total
- Phase 2: +4 sources = 11 total
- Phase 3: +2 sources = 13 total  
- Phase 4: +2 sources = 15 total
- Phase 5: +2 sources = 17 total

Target: 17/23 (74%) working sources
