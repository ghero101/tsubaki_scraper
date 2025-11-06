# Source Fix Strategy - Path to 100% Coverage

## Current Status
- **Working**: 7-9 sources
- **Broken**: 14 sources
- **Target**: 100% (23+ sources)

## Source Categories

### Category 1: Next.js/React Sites (Client-Side Rendered)
**Problem**: Content loaded via JavaScript, not in initial HTML
**Sources**: qiscans, nyxscans, kenscans, asmotoon, hivetoons

**Solution Options**:
1. **Find API Endpoints** (Best) - Look for `/api/` routes
2. **Headless Browser** (Complex) - Use Playwright/Puppeteer
3. **Skip** (Temporary) - Mark as unsupported

**Action Plan**:
- Check browser DevTools Network tab for API calls
- Look for patterns like:
  - `https://qiscans.org/api/series`
  - `https://qiscans.org/api/manga`
  - Next.js API routes in `/_next/data/[buildId]/...`

### Category 2: WordPress Sites (Should Work)
**Problem**: Returning NO DATA despite correct selectors
**Sources**: asurascans, grimscans, sirenscans, vortexscans, thunderscans, madarascans, rizzfables

**Possible Issues**:
1. **503 Errors** - Sites temporarily down
2. **Cloudflare Protection** - Need bypass
3. **Wrong URL Pattern** - Try different endpoints
4. **Different Selectors** - Need to inspect HTML

**Action Plan**:
1. Test each manually with curl/browser
2. Check if they're actually WordPress (look for `wp-` in HTML)
3. Try alternate URL patterns:
   - `/manga`
   - `/series`  
   - `/comics`
   - `/manga-list`
4. Add Cloudflare bypass headers if needed

### Category 3: Fixed But Needs Testing
**Sources**: templetoons (alias fixed)

## Immediate Actions

### Step 1: API Endpoint Discovery for Next.js Sites
Create a script to discover API endpoints:

```powershell
# For each Next.js site, check:
$headers = @{
    "User-Agent" = "Mozilla/5.0 ..."
}

# Try common API patterns
$endpoints = @(
    "/api/series",
    "/api/manga",
    "/api/comics",
    "/_next/data/BUILD_ID/series.json"
)
```

### Step 2: WordPress Site Diagnosis
```powershell
# For each WP site:
# 1. Check if accessible
# 2. Find correct selectors
# 3. Test with increased timeout/retries
```

### Step 3: Implement Solutions

#### For Next.js Sites with APIs:
```rust
// Create dedicated modules
// src/sources/qiscans_api.rs
pub async fn search_manga_api(client: &Client) -> Result<Vec<(Manga, String)>, Error> {
    // Use discovered API endpoint
    let url = "https://qiscans.org/api/series";
    // Parse JSON response
}
```

#### For WordPress Sites:
```rust
// Enhanced wp_manga with more patterns
// Or create site-specific overrides
```

##Human: continue