# Source Scraper Status Tracker

Last updated: 2025-11-07 (After Next.js sources completed)

## ✅ WORKING WELL (Good chapter counts)

| Source | Manga | Chapters | Status | Notes |
|--------|-------|----------|--------|-------|
| MangaDex | 10 | 1983 | ✅ EXCELLENT | API-based |
| Manhuaus | 10 | 865 | ✅ EXCELLENT | |
| RizzFables | 10 | 536 | ✅ EXCELLENT | |
| **AsuraScans** | 10 | **533** | ✅ **EXCELLENT** | **FIXED! Was 3→533** |
| **FlameComics** | 10 | **382** | ✅ **EXCELLENT** | **FIXED! Was 3→382** |
| WitchScans | 10 | 181 | ✅ EXCELLENT | |
| RizzComic | 10 | 144 | ✅ EXCELLENT | |
| ResetScans | 10 | 111 | ✅ EXCELLENT | |
| FireScans | 10 | 110 | ✅ EXCELLENT | |
| RokariComics | 10 | 73 | ✅ GOOD | |
| StoneScape | 10 | 36 | ✅ ACCEPTABLE | |
| VizMedia | 10 | 30 | ✅ ACCEPTABLE | |
| KodokuStudio | 6 | 21 | ✅ ACCEPTABLE | |

## 🎯 NEXT.JS SOURCES - COMPLETE!

| Source | Implementation | Status | Notes |
|--------|---------------|---------|-------|
| **AsuraScans** | HTML scraping | ✅ **FIXED!** | 3→533 chapters |
| **FlameComics** | JSON parser | ✅ **FIXED!** | 3→382 chapters (JSON parser was already working!) |
| Kagane | JSON + Browser | ⚠️ **NEEDS BROWSER** | Browser automation not yet implemented |

**Result:** 2 of 3 Next.js sources now working! Combined improvement: **912 new chapters!**

## 🔧 WP_MANGA SOURCES (HIGH PRIORITY - Single fix = 6+ sources!)

These ALL use `wp_manga::get_chapters_base()` which is broken (only finds 3 chapters):

| Source | Current Chapters | Expected | Priority |
|--------|------------------|----------|----------|
| HiveToons | 3 | ~100+ | 🔴 HIGH |
| KenScans | 3 | ~100+ | 🔴 HIGH |
| QIScans | 3 | ~100+ | 🔴 HIGH |
| MavinTranslations | 3 | ~100+ | 🔴 HIGH |
| Asmotoon | 3 | ~100+ | 🔴 HIGH |
| NyxScans | 3 | ~100+ | 🔴 HIGH |

**IMPACT:** Fixing `wp_manga::get_chapters_base()` will improve 6+ sources at once!

## 🔧 BROWSER-REQUIRED SOURCES

These need browser automation (future work):

| Source | Status | Implementation Needed |
|--------|--------|----------------------|
| Kagane | NO_DATA | Browser module (`sources_browser/`) |
| NyxScans | 3 chapters | Has browser fallback stub, needs implementation |

## 📊 METADATA ONLY (No chapters expected)

| Source | Manga | Chapters | Notes |
|--------|-------|----------|-------|
| MyAnimeList | 5 | 0 | Metadata aggregator |
| AniList | 10 | 0 | Metadata aggregator |
| JNovelClub | 5 | 0 | Publisher/licensing info |
| SquareEnixManga | 10 | 0 | Publisher catalog |
| Comikey | 5 | 0 | Publisher platform |
| InkrComics | 2 | 0 | Publisher platform |
| Toomics | 1 | 0 | Publisher platform |

## ❌ NO_DATA (32 sources - Need investigation)

High priority (popular scanlation sites):
- DrakeComic, MadaraScans, ThunderScans, SirenScans, VortexScans
- TempleScan, DayComics, LunaToons, Webtoon, Tapas, Webcomics, MediBang

Lower priority (publishers/platforms):
- VASTVisual, KDTNovels, KodanshaComics, YenPress, DarkHorseComics
- SevenSeas, DenpaBooks, IrodoriComics, OnePeaceBooks, Tokyopop
- TitanManga, UdonEntertainment, Shueisha, Lezhin, PocketComics
- Tappytoon, Manta, BookLive, Fakku, Others

## 🚨 ERROR (Critical issues)

| Source | Error Type | Notes |
|--------|------------|-------|
| GrimScans | Connection | "message unexpected or badly formatted" |
| Kana | SSL_ERROR | Untrusted root certificate |

---

## Completed Work ✅

### Phase 1: Next.js Sites - DONE! ✅
1. ✅ AsuraScans - FIXED (3→533 chapters)
2. ✅ FlameComics - FIXED (3→382 chapters) 
3. ⚠️ Kagane - Requires browser (future work)

**Total improvement: +912 chapters!**

---

## Priority Action Plan - NEXT STEPS

### 🔥 Phase 2: Fix wp_manga Module (HIGHEST IMPACT!)
**Single fix → solves 6+ sources instantly!**

The `wp_manga::get_chapters_base()` function is broken. Fixing it will improve:
- HiveToons (3→~100+ chapters)
- KenScans (3→~100+ chapters)
- QIScans (3→~100+ chapters)
- MavinTranslations (3→~100+ chapters)
- Asmotoon (3→~100+ chapters)
- NyxScans (3→~100+ chapters)

**Expected total improvement: ~600+ chapters across 6 sources!**

### Phase 3: Investigate NO_DATA Sources
- Check if sites are still online
- Verify URL patterns
- Update selectors if needed

### Phase 4: Browser Automation (Future)
- Implement `sources_browser/` module
- Add Kagane browser support
- Add NyxScans browser support

### Phase 5: Fix ERROR Sources
- GrimScans: Connection issues
- Kana: SSL certificate problems
