# Browser Setup Guide for Manga Scraper

## Overview

Some manga sources require JavaScript rendering or bypass Cloudflare protection. These sources need a headless browser (Chrome/Chromium).

## Sources Requiring Browser (7 total)

1. **DrakeComic** - Cloudflare protection
2. **MadaraScans** - Cloudflare protection
3. **ThunderScans** - JavaScript rendering
4. **SirenScans** - JavaScript rendering
5. **VortexScans** - JavaScript rendering
6. **TempleScan** - JavaScript rendering
7. **Kagane** - Next.js app (client-side rendering)

## Quick Setup

### Option 1: Install Chrome/Chromium (Recommended)

**Linux (Debian/Ubuntu):**
```bash
# Install Chromium
sudo apt update
sudo apt install chromium-browser

# Or install Google Chrome
wget https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb
sudo dpkg -i google-chrome-stable_current_amd64.deb
sudo apt --fix-broken install
```

**macOS:**
```bash
# Using Homebrew
brew install --cask google-chrome
# or
brew install --cask chromium
```

**Windows:**
- Download from: https://www.google.com/chrome/
- Or download Chromium: https://download-chromium.appspot.com/

### Option 2: Auto-Download Chromium (Automatic)

The scraper will automatically download Chromium on first run if not found. This takes a few minutes.

### Option 3: Disable Browser Sources (Testing Only)

To skip browser sources and test faster:

**Linux/macOS:**
```bash
export MANGA_SCRAPER_USE_BROWSER=0
cargo test --test source_validation_test -- --ignored
```

**Windows PowerShell:**
```powershell
$env:MANGA_SCRAPER_USE_BROWSER = "0"
cargo test --test source_validation_test -- --ignored
```

## Verifying Browser Setup

Run this to test browser availability:

```bash
# Check if Chrome/Chromium is installed
which chromium-browser  # Linux
which google-chrome     # Linux
which chromium          # macOS
where chrome.exe        # Windows
```

## Troubleshooting

### Error: "Chrome not found"
- Install Chrome/Chromium using instructions above
- Or wait for automatic download (first run only)

### Error: "Browser timeout"
- Increase timeout in `src/browser_client.rs` (default: 30s)
- Check your internet connection
- Some sources may genuinely require manual setup

### Error: "Cloudflare detected"
- This is expected - browser bypass should handle it
- If it fails, the source may have updated their protection

## Performance Notes

- **First run**: Slower (browser initialization ~30s per source)
- **Subsequent runs**: Faster (browser reuse)
- **With browser disabled**: Much faster (HTTP only, some sources fail)

## HTTP Fallbacks

All browser-based sources have HTTP fallbacks that attempt standard scraping first. If the browser fails, they fall back to HTTP. This means:

- ✅ Sites without Cloudflare may work via HTTP
- ✅ Faster if browser unavailable
- ❌ JavaScript-heavy sites will return NO_DATA

## Recommended Testing Workflow

1. **First test** - Disable browser for speed:
   ```bash
   MANGA_SCRAPER_USE_BROWSER=0 cargo test --test source_validation_test -- --ignored
   ```

2. **Second test** - Enable browser for full coverage:
   ```bash
   cargo test --test source_validation_test -- --ignored
   ```

This two-pass approach identifies which sources work via HTTP vs. need browser.
