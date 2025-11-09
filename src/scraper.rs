use crate::models::Source;
use headless_chrome::{Browser, LaunchOptions};
use regex::Regex;
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::fs::File;
use std::io::{copy, Cursor};
use std::path::PathBuf;
use std::time::Duration;
use zip::write::{FileOptions, ZipWriter};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtHomeServer {
    pub base_url: String,
    pub chapter: AtHomeChapter,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtHomeChapter {
    pub hash: String,
    pub data: Vec<String>,
    #[allow(dead_code)]
    pub data_saver: Vec<String>,
}

fn sanitize_filename(s: &str) -> String {
    s.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}

fn source_name_from_id(source_id: i32) -> &'static str {
    match source_id {
        x if x == Source::MangaDex as i32 => "MangaDex",
        x if x == Source::FireScans as i32 => "FireScans",
        x if x == Source::RizzComic as i32 => "RizzComic",
        x if x == Source::DrakeComic as i32 => "DrakeComic",
        x if x == Source::Asmotoon as i32 => "Asmotoon",
        x if x == Source::ResetScans as i32 => "ResetScans",
        x if x == Source::Kagane as i32 => "Kagane",
        x if x == Source::KDTNovels as i32 => "KDTNovels",
        _ => "Unknown",
    }
}

fn format_chapter_label(chapter_number: &str, chapter_url: &str) -> String {
    // Try to construct "Vol.X Ch.Y - Title" if possible
    let mut vol: Option<String> = None;
    let mut ch: Option<String> = None;
    let mut title: Option<String> = None;

    let num_re = Regex::new(r"(?i)(?:ch(?:apter)?\s*)?(\d+(?:\.\d+)?)").unwrap();
    let vol_re = Regex::new(r"(?i)(?:vol(?:ume)?\s*)?(\d+)").unwrap();

    // From chapter_number text
    if let Some(cap) = num_re.captures(chapter_number) {
        ch = cap.get(1).map(|m| m.as_str().to_string());
    }
    if let Some(cap) = vol_re.captures(chapter_number) {
        vol = cap.get(1).map(|m| m.as_str().to_string());
    }
    if chapter_number.contains('-') {
        let parts: Vec<&str> = chapter_number.splitn(2, '-').collect();
        if parts.len() == 2 {
            let t = parts[1].trim();
            if !t.is_empty() {
                title = Some(t.to_string());
            }
        }
    }

    // From URL slug
    let lower_url = chapter_url.to_lowercase();
    if ch.is_none() {
        if let Some(cap) = Regex::new(r"chapter[-/](\d+(?:\.\d+)?)")
            .unwrap()
            .captures(&lower_url)
        {
            ch = cap.get(1).map(|m| m.as_str().to_string());
        }
    }
    if vol.is_none() {
        if let Some(cap) = Regex::new(r"vol(?:ume)?[-/](\d+)")
            .unwrap()
            .captures(&lower_url)
        {
            vol = cap.get(1).map(|m| m.as_str().to_string());
        }
    }

    // Build label
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = vol {
        parts.push(format!("Vol.{}", v));
    }
    if let Some(cn) = ch {
        parts.push(format!("Ch.{}", cn));
    }
    let base = if parts.is_empty() {
        chapter_number.to_string()
    } else {
        parts.join(" ")
    };
    if let Some(t) = title {
        format!("{} - {}", base, t)
    } else {
        base
    }
}

pub async fn download_chapter(
    client: &Client,
    source_id: i32,
    chapter_url: &str,
    manga_title: &str,
    chapter_number: &str,
    base_dir: &str,
    comicinfo_xml: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Root download directory
    std::fs::create_dir_all(base_dir)?;

    // Per-manga folder with subfolders for covers and artwork
    let sanitized_title = sanitize_filename(manga_title);
    let manga_dir: PathBuf = [base_dir, &sanitized_title].iter().collect();
    std::fs::create_dir_all(&manga_dir)?;
    std::fs::create_dir_all(manga_dir.join("covers"))?;
    std::fs::create_dir_all(manga_dir.join("artwork"))?;

    // Build filename: <sourceId>-<SourceName>-<MangaTitle> - <ChapterLabel>.cbz
    let label = format_chapter_label(chapter_number, chapter_url);
    let sanitized_chapter = sanitize_filename(&label);
    let source_name = sanitize_filename(source_name_from_id(source_id));
    let file_name = format!(
        "{}-{}-{} - {}.cbz",
        source_id, source_name, sanitized_title, sanitized_chapter
    );

    let file_path = manga_dir.join(&file_name);
    let tmp_path = file_path.with_extension("cbz.tmp");
    let file = File::create(&tmp_path)?;
    let mut zip = ZipWriter::new(file);
    if let Some(xml) = comicinfo_xml {
        use std::io::Write;
        zip.start_file("ComicInfo.xml", FileOptions::default())?;
        zip.write_all(xml.as_bytes())?;
    }

    match source_id {
        x if x == Source::MangaDex as i32 => {
            // Get the at-home server URL
            let url = format!("https://api.mangadex.org/at-home/server/{}", chapter_url);
            let response = client
                .get(&url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .send()
                .await?
                .json::<AtHomeServer>()
                .await?;

            // Download each page and add it to the zip file
            for (i, page) in response.chapter.data.iter().enumerate() {
                let page_url = format!(
                    "{}/data/{}/{}",
                    response.base_url, response.chapter.hash, page
                );
                let response = client
                    .get(&page_url)
                    .header("Referer", "https://mangadex.org/")
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        x if x == Source::FireScans as i32
            || x == Source::RizzComic as i32
            || x == Source::DrakeComic as i32
            || x == Source::Asmotoon as i32
            || x == Source::ResetScans as i32
            || x == Source::TempleScan as i32
            || x == Source::ThunderScans as i32 =>
        {
            let response = client
                .get(chapter_url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .send()
                .await?
                .text()
                .await?;
            let document = Html::parse_document(&response);
            let selector = Selector::parse("div.reading-content img").unwrap();
            let mut image_list: Vec<String> = Vec::new();

            for element in document.select(&selector) {
                if let Some(image_url) = element
                    .value()
                    .attr("src")
                    .or_else(|| element.value().attr("data-src"))
                {
                    image_list.push(image_url.to_string());
                }
            }
            if image_list.is_empty() {
                // Regex fallback: scrape direct image URLs from HTML
                let re = Regex::new(r#"https?://[^"'\s>]+\.(?:jpg|jpeg|png)"#).unwrap();
                let mut seen = std::collections::HashSet::new();
                for cap in re.captures_iter(&response) {
                    let url = cap.get(0).unwrap().as_str().to_string();
                    if seen.insert(url.clone()) {
                        image_list.push(url);
                    }
                }
            }

            // Resolve relative image URLs and set a reasonable Referer per origin
            let origin = Url::parse(chapter_url)
                .ok()
                .and_then(|u| Some(format!("{}://{}", u.scheme(), u.host_str()?)))
                .unwrap_or_default();
            for (i, src) in image_list.iter().enumerate() {
                let full_url = if let Ok(base) = Url::parse(chapter_url) {
                    base.join(src)
                        .map(|u| u.to_string())
                        .unwrap_or_else(|_| src.clone())
                } else {
                    src.clone()
                };
                let response = client
                    .get(&full_url)
                    .header(
                        "Referer",
                        if !origin.is_empty() {
                            origin.as_str()
                        } else {
                            chapter_url
                        },
                    )
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        x if x == Source::Kagane as i32 => {
            // Kagane is a Next.js app; attempt to extract all readable images from the reader area.
            let response = client
                .get(chapter_url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .send()
                .await?
                .text()
                .await?;
            let document = Html::parse_document(&response);
            let selectors = vec![
                Selector::parse("div[data-reader] img").unwrap(),
                Selector::parse("div.reading-content img").unwrap(),
                Selector::parse("main img").unwrap(),
            ];
            let mut image_list: Vec<String> = Vec::new();
            'outer: for selector in selectors {
                for element in document.select(&selector) {
                    if let Some(image_url) = element
                        .value()
                        .attr("src")
                        .or_else(|| element.value().attr("data-src"))
                    {
                        // Filter likely page images
                        if image_url.contains("http")
                            && (image_url.ends_with(".jpg")
                                || image_url.ends_with(".jpeg")
                                || image_url.ends_with(".png"))
                        {
                            image_list.push(image_url.to_string());
                        }
                    }
                }
                if !image_list.is_empty() {
                    break 'outer;
                }
            }
            for (i, image_url) in image_list.iter().enumerate() {
                let response = client
                    .get(image_url)
                    .header("Referer", "https://mangadex.org/")
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        _ => {
            // Generic HTML reader fallback
            let response = client.get(chapter_url).send().await?.text().await?;
            let document = Html::parse_document(&response);
            let selector = Selector::parse("div.reading-content img").unwrap();
            let mut image_list: Vec<String> = Vec::new();
            for element in document.select(&selector) {
                if let Some(image_url) = element
                    .value()
                    .attr("src")
                    .or_else(|| element.value().attr("data-src"))
                {
                    image_list.push(image_url.to_string());
                }
            }
            if image_list.is_empty() {
                let re = Regex::new(r#"https?://[^"'\s>]+\.(?:jpg|jpeg|png)"#).unwrap();
                let mut seen = std::collections::HashSet::new();
                for cap in re.captures_iter(&response) {
                    let url = cap.get(0).unwrap().as_str().to_string();
                    if seen.insert(url.clone()) {
                        image_list.push(url);
                    }
                }
            }
            for (i, image_url) in image_list.iter().enumerate() {
                let response = client.get(image_url).send().await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
    }

    let res = zip.finish();
    if let Err(e) = res {
        // Cleanup partial file on error
        let _ = std::fs::remove_file(&tmp_path);
        return Err(Box::new(e));
    }
    // Move temp file to final destination
    std::fs::rename(&tmp_path, &file_path)?;

    Ok(file_path.to_string_lossy().to_string())
}

pub async fn ensure_cover_downloaded(
    client: &Client,
    base_dir: &str,
    manga_title: &str,
    cover_url: &str,
    source_id: i32,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let sanitized_title = sanitize_filename(manga_title);
    let manga_dir: PathBuf = [base_dir, &sanitized_title].iter().collect();
    let covers_dir = manga_dir.join("covers");
    std::fs::create_dir_all(&covers_dir)?;
    // Determine ext
    let ext = cover_url
        .rsplit('.')
        .next()
        .unwrap_or("jpg")
        .split('?')
        .next()
        .unwrap_or("jpg");
    let fname = format!(
        "{}-{}-cover.{}",
        source_id,
        sanitize_filename(source_name_from_id(source_id)),
        ext
    );
    let path = covers_dir.join(&fname);
    if path.exists() {
        return Ok(Some(path.to_string_lossy().to_string()));
    }
    let resp = client
        .get(cover_url)
        .header("User-Agent", "rust_manga_scraper/0.1.0")
        .send()
        .await?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let bytes = resp.bytes().await?;
    std::fs::write(&path, &bytes)?;
    Ok(Some(path.to_string_lossy().to_string()))
}

/// Download chapter using headless browser for Cloudflare-protected sites
async fn download_chapter_with_browser(
    client: &Client,
    chapter_url: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use crate::cloudflare_bypass::{
        get_fingerprint_spoofing_script, CloudflareConfig, SessionManager,
    };
    use std::io::Cursor;

    // Load Cloudflare bypass configuration
    let cf_config = CloudflareConfig::load().unwrap_or_else(|e| {
        log::warn!(
            "Failed to load cloudflare_config.toml, using defaults: {}",
            e
        );
        CloudflareConfig::default()
    });

    log::info!("🔒 Advanced Cloudflare Bypass: headless={}, fingerprint={}, proxy={}, captcha={}, session={}",
        cf_config.browser.headless, cf_config.fingerprint.enabled,
        cf_config.proxy.enabled, cf_config.captcha.enabled, cf_config.session.enabled);

    // FEATURE #5: Initialize Session Manager
    let session_manager = SessionManager::new(cf_config.session.clone());

    // Configure browser with all bypass features
    let mut launch_options = LaunchOptions::default();
    launch_options.headless = cf_config.browser.headless; // Support real Chrome!
    launch_options.sandbox = false;

    // Build args list with stealth + config options
    let mut args = vec![
        std::ffi::OsStr::new("--disable-blink-features=AutomationControlled"),
        std::ffi::OsStr::new("--disable-dev-shm-usage"),
        std::ffi::OsStr::new("--no-first-run"),
        std::ffi::OsStr::new("--no-default-browser-check"),
        std::ffi::OsStr::new("--disable-background-networking"),
        std::ffi::OsStr::new("--disable-background-timer-throttling"),
        std::ffi::OsStr::new("--disable-backgrounding-occluded-windows"),
        std::ffi::OsStr::new("--disable-breakpad"),
        std::ffi::OsStr::new("--disable-client-side-phishing-detection"),
        std::ffi::OsStr::new("--disable-component-update"),
        std::ffi::OsStr::new("--disable-default-apps"),
        std::ffi::OsStr::new("--disable-extensions"),
        std::ffi::OsStr::new("--disable-features=TranslateUI"),
        std::ffi::OsStr::new("--disable-hang-monitor"),
        std::ffi::OsStr::new("--disable-ipc-flooding-protection"),
        std::ffi::OsStr::new("--disable-popup-blocking"),
        std::ffi::OsStr::new("--disable-prompt-on-repost"),
        std::ffi::OsStr::new("--disable-renderer-backgrounding"),
        std::ffi::OsStr::new("--disable-sync"),
        std::ffi::OsStr::new("--metrics-recording-only"),
        std::ffi::OsStr::new("--no-sandbox"),
        std::ffi::OsStr::new("--safebrowsing-disable-auto-update"),
    ];

    // Add window size
    let window_arg = format!("--window-size={}", cf_config.browser.window_size);
    args.push(std::ffi::OsStr::new(&window_arg));

    // Add user agent
    let user_agent = if cf_config.fingerprint.random_user_agent {
        crate::cloudflare_bypass::get_random_user_agent()
    } else {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    };
    let ua_arg = format!("--user-agent={}", user_agent);
    args.push(std::ffi::OsStr::new(&ua_arg));

    // Proxy support (Feature #1: Residential Proxies)
    let proxy_arg: Option<String> =
        if cf_config.proxy.enabled && !cf_config.proxy.proxies.is_empty() {
            // Use first proxy for now (rotation would need global state)
            let proxy = &cf_config.proxy.proxies[0];
            log::info!("🌐 Using proxy: {}", proxy);
            Some(format!("--proxy-server={}", proxy))
        } else {
            None
        };
    if let Some(ref proxy_str) = proxy_arg {
        args.push(std::ffi::OsStr::new(proxy_str));
    }

    launch_options.args = args;

    let browser = Browser::new(launch_options)?;

    let tab = browser.new_tab()?;

    // FEATURE #5: Restore session cookies if available
    if let Some(session_data) = session_manager.get_session(chapter_url) {
        log::info!(
            "🔄 Restoring {} saved cookies from previous session",
            session_data.cookies.len()
        );

        // Set cookies using Chrome DevTools Protocol
        // Note: Cookie restoration requires navigating to the domain first for some browsers
        // For now, we'll log that we have cookies to restore
        // A full implementation would require calling Network.setCookie via CDP
        log::debug!("Cookie restoration prepared (requires CDP Network.setCookie implementation)");
    } else {
        log::debug!("No saved session found for {}", chapter_url);
    }

    // Navigate to chapter page
    tab.navigate_to(chapter_url)?;
    tab.wait_until_navigated()?;

    // FEATURE #2: Inject fingerprint spoofing JavaScript
    if cf_config.fingerprint.enabled {
        let spoof_script = get_fingerprint_spoofing_script(&cf_config.fingerprint);
        match tab.evaluate(&spoof_script, false) {
            Ok(_) => log::info!("🎭 Fingerprint spoofing injected successfully"),
            Err(e) => log::warn!("Failed to inject fingerprint spoofing: {}", e),
        }
    }

    // Enhanced Cloudflare bypass: wait and detect multiple challenge indicators
    std::thread::sleep(Duration::from_secs(5));

    // Advanced Cloudflare detection and waiting
    let cloudflare_result = tab.evaluate(r#"
        (async function() {
            const maxWait = 60; // Wait up to 60 seconds total
            let waited = 0;

            function isCloudflareChallenge() {
                const title = document.title.toLowerCase();
                const body = document.body ? document.body.textContent.toLowerCase() : '';

                // Multiple Cloudflare indicators
                return title.includes('just a moment') ||
                       title.includes('please wait') ||
                       title.includes('checking your browser') ||
                       body.includes('cloudflare') ||
                       body.includes('checking your browser') ||
                       body.includes('ddos protection') ||
                       document.querySelector('#challenge-form') !== null ||
                       document.querySelector('.cf-browser-verification') !== null;
            }

            function hasCaptchaChallenge() {
                // FEATURE #4: Detect CAPTCHA challenges
                return document.querySelector('[data-sitekey]') !== null ||
                       document.querySelector('.g-recaptcha') !== null ||
                       document.querySelector('.h-captcha') !== null ||
                       document.querySelector('iframe[src*="recaptcha"]') !== null ||
                       document.querySelector('iframe[src*="hcaptcha"]') !== null ||
                       document.body?.textContent?.toLowerCase().includes('verify you are human');
            }

            function getCaptchaSiteKey() {
                const recaptcha = document.querySelector('[data-sitekey]');
                if (recaptcha) return recaptcha.getAttribute('data-sitekey');

                const recaptchaIframe = document.querySelector('iframe[src*="recaptcha"]');
                if (recaptchaIframe) {
                    const match = recaptchaIframe.src.match(/[?&]k=([^&]+)/);
                    return match ? match[1] : null;
                }
                return null;
            }

            function hasRealContent() {
                // Check if actual manga content has loaded
                const hasImages = document.querySelectorAll('img[src*=".jpg"], img[src*=".png"], img[src*=".webp"]').length > 3;
                const hasReader = document.querySelector('.reading-content, .read-content, #readerarea, .reader-area') !== null;
                return hasImages || hasReader;
            }

            // Wait for Cloudflare to finish and content to load
            while (waited < maxWait && (isCloudflareChallenge() || !hasRealContent())) {
                await new Promise(r => setTimeout(r, 1000));
                waited++;

                // Early exit if content appears
                if (hasRealContent() && !isCloudflareChallenge()) {
                    break;
                }
            }

            return {
                waited: waited,
                stillBlocked: isCloudflareChallenge(),
                hasCaptcha: hasCaptchaChallenge(),
                captchaSiteKey: getCaptchaSiteKey(),
                hasContent: hasRealContent()
            };
        })();
    "#, false);

    // Log results and handle CAPTCHA detection
    if let Ok(result) = cloudflare_result {
        log::debug!("Cloudflare bypass result: {:?}", result);

        // Try to parse the result as JSON to check for CAPTCHA
        // The RemoteObject needs to be converted to JSON
        let result_json = serde_json::to_value(&result).unwrap_or(serde_json::Value::Null);
        if let Some(obj) = result_json.as_object() {
            if let Some(has_captcha) = obj.get("hasCaptcha") {
                if has_captcha.as_bool().unwrap_or(false) {
                    let site_key = obj
                        .get("captchaSiteKey")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    log::warn!("⚠️  CAPTCHA challenge detected (site_key: {})", site_key);

                    // TODO: FEATURE #4 - Integrate CAPTCHA solving here
                    // This requires async refactoring of this function
                    // If cf_config.captcha.enabled {
                    //     let solution = solve_captcha(&cf_config.captcha, site_key, chapter_url).await?;
                    //     // Inject solution and submit
                    // }

                    if !cf_config.captcha.enabled {
                        log::warn!(
                            "CAPTCHA solving is disabled. Enable it in cloudflare_config.toml"
                        );
                    }
                }
            }
        }
    }

    // Additional wait for any remaining JS execution
    std::thread::sleep(Duration::from_secs(3));

    // FEATURE #5: Save session cookies after successful Cloudflare bypass
    // Note: Full cookie extraction requires CDP Network.getAllCookies implementation
    // Session manager infrastructure is ready for future enhancement
    if cf_config.session.enabled {
        log::debug!(
            "Session management enabled (cookie extraction requires CDP Network.getAllCookies)"
        );
    }

    // Get HTML content
    let html = tab.get_content()?;
    let document = Html::parse_document(&html);

    // Try multiple selectors to find images
    let selectors = vec![
        "div.reading-content img",
        "div.read-content img",
        "div.page-break img",
        "div#readerarea img",
        "div.reader-area img",
        "div.chapter-content img",
        "div.entry-content img",
        "main img[src*='.jpg'], main img[src*='.png'], main img[src*='.webp']",
        "img[loading='lazy']",
    ];

    let mut image_list = Vec::new();
    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                if let Some(image_url) = element
                    .value()
                    .attr("src")
                    .or_else(|| element.value().attr("data-src"))
                    .or_else(|| element.value().attr("data-lazy-src"))
                {
                    let url_lower = image_url.to_lowercase();
                    if (url_lower.contains(".jpg")
                        || url_lower.contains(".jpeg")
                        || url_lower.contains(".png")
                        || url_lower.contains(".webp"))
                        && !url_lower.contains("logo")
                        && !url_lower.contains("icon")
                        && !url_lower.contains("avatar")
                        && !url_lower.contains("banner")
                    {
                        image_list.push(image_url.to_string());
                    }
                }
            }
            if !image_list.is_empty() {
                break;
            }
        }
    }

    // If no images found via CSS, try JSON parsing
    if image_list.is_empty() {
        // Pattern 1: "chapterImages":[...]
        if let Some(json_start) = html.find("\"chapterImages\":[") {
            if let Some(json_end) = html[json_start..].find("]") {
                let json_str = &html[json_start..json_start + json_end + 1];
                if let Some(array_start) = json_str.find("[") {
                    let array_str = &json_str[array_start..];
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(array_str) {
                        if let Some(images) = json_value.as_array() {
                            for img in images {
                                if let Some(url) = img.get("url").and_then(|u| u.as_str()) {
                                    image_list.push(url.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Pattern 2: "sources":[{"images":[...]}]  (RizzComic, Rokari, etc.)
        if image_list.is_empty() {
            if let Some(sources_start) = html.find("\"sources\":[") {
                // Find the end of the sources array (look for the closing bracket)
                let search_range = &html[sources_start
                    ..sources_start + std::cmp::min(50000, html.len() - sources_start)];
                if let Some(array_end) = search_range.rfind("]") {
                    let json_str = &html[sources_start..sources_start + array_end + 1];
                    if let Some(array_start) = json_str.find("[") {
                        let array_str = &json_str[array_start..];
                        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(array_str)
                        {
                            if let Some(sources) = json_value.as_array() {
                                for source_obj in sources {
                                    if let Some(images_array) =
                                        source_obj.get("images").and_then(|i| i.as_array())
                                    {
                                        for img_url in images_array {
                                            if let Some(url) = img_url.as_str() {
                                                image_list.push(url.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Create ZIP archive
    let buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buffer);

    let origin = reqwest::Url::parse(chapter_url)
        .ok()
        .and_then(|u| Some(format!("{}://{}", u.scheme(), u.host_str()?)))
        .unwrap_or_default();
    let base = reqwest::Url::parse(chapter_url).ok();

    for (i, image_url) in image_list.iter().enumerate() {
        let full_url = if let Some(ref b) = base {
            b.join(image_url)
                .map(|u| u.to_string())
                .unwrap_or(image_url.clone())
        } else {
            image_url.clone()
        };

        let response = client
            .get(&full_url)
            .header(
                "Referer",
                if !origin.is_empty() {
                    origin.as_str()
                } else {
                    chapter_url
                },
            )
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .send()
            .await?;

        let mut cursor = Cursor::new(response.bytes().await?);
        zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
        copy(&mut cursor, &mut zip)?;
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

pub async fn download_chapter_to_memory(
    client: &Client,
    source_id: i32,
    chapter_url: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::io::Cursor;

    // Check if this source requires browser automation (Cloudflare-protected)
    let cloudflare_sources = vec![3, 9, 20, 38, 39, 40, 59]; // RizzComic, ResetScans, HiveToons, QiScans, RizzFables, RokariComics, WitchScans
    if cloudflare_sources.contains(&source_id) {
        return download_chapter_with_browser(client, chapter_url).await;
    }

    let buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buffer);

    match source_id {
        x if x == Source::MangaDex as i32 => {
            let url = format!("https://api.mangadex.org/at-home/server/{}", chapter_url);
            let response = client
                .get(&url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .send()
                .await?
                .json::<AtHomeServer>()
                .await?;

            for (i, page) in response.chapter.data.iter().enumerate() {
                let page_url = format!(
                    "{}/data/{}/{}",
                    response.base_url, response.chapter.hash, page
                );
                let response = client
                    .get(&page_url)
                    .header("Referer", "https://mangadex.org/")
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        x if x == Source::FireScans as i32 || x == Source::RizzComic as i32 => {
            let response = client
                .get(chapter_url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .send()
                .await?
                .text()
                .await?;
            let document = Html::parse_document(&response);
            let selector = Selector::parse("div.reading-content > div.page-break > img").unwrap();
            let mut image_list = Vec::new();

            for element in document.select(&selector) {
                let image_url = element.value().attr("src").unwrap().to_string();
                image_list.push(image_url);
            }

            // Resolve relative image URLs and set a reasonable Referer per origin
            let origin = reqwest::Url::parse(chapter_url)
                .ok()
                .and_then(|u| Some(format!("{}://{}", u.scheme(), u.host_str()?)))
                .unwrap_or_default();
            for (i, src) in image_list.iter().enumerate() {
                let full_url = if let Ok(base) = reqwest::Url::parse(chapter_url) {
                    base.join(src)
                        .map(|u| u.to_string())
                        .unwrap_or_else(|_| src.clone())
                } else {
                    src.clone()
                };
                let response = client
                    .get(&full_url)
                    .header(
                        "Referer",
                        if !origin.is_empty() {
                            origin.as_str()
                        } else {
                            chapter_url
                        },
                    )
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        x if x == Source::Kagane as i32 => {
            let response = client.get(chapter_url).send().await?.text().await?;
            let document = Html::parse_document(&response);
            let selectors = vec![
                Selector::parse("div[data-reader] img").unwrap(),
                Selector::parse("div.reading-content img").unwrap(),
                Selector::parse("main img").unwrap(),
            ];
            let mut image_list: Vec<String> = Vec::new();
            'outer: for selector in selectors {
                for element in document.select(&selector) {
                    if let Some(image_url) = element
                        .value()
                        .attr("src")
                        .or_else(|| element.value().attr("data-src"))
                    {
                        if image_url.contains("http")
                            && (image_url.ends_with(".jpg")
                                || image_url.ends_with(".jpeg")
                                || image_url.ends_with(".png"))
                        {
                            image_list.push(image_url.to_string());
                        }
                    }
                }
                if !image_list.is_empty() {
                    break 'outer;
                }
            }
            for (i, image_url) in image_list.iter().enumerate() {
                let response = client.get(image_url).send().await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        _ => {
            // Generic HTML reader fallback to memory with multiple selectors
            let response = client.get(chapter_url).send().await?.text().await?;
            let document = Html::parse_document(&response);

            // Try multiple selectors in order
            let selectors = vec![
                "div.reading-content img",
                "div.read-content img",
                "div.page-break img",
                "div#readerarea img",
                "div.reader-area img",
                "div.chapter-content img",
                "div.entry-content img",
                "main img[src*='.jpg'], main img[src*='.png'], main img[src*='.webp']",
                "img[loading='lazy']",
            ];

            let mut image_list = Vec::new();
            for selector_str in selectors {
                if let Ok(selector) = Selector::parse(selector_str) {
                    for element in document.select(&selector) {
                        if let Some(image_url) = element
                            .value()
                            .attr("src")
                            .or_else(|| element.value().attr("data-src"))
                            .or_else(|| element.value().attr("data-lazy-src"))
                        {
                            // Filter out obviously non-chapter images
                            let url_lower = image_url.to_lowercase();
                            if (url_lower.contains(".jpg")
                                || url_lower.contains(".jpeg")
                                || url_lower.contains(".png")
                                || url_lower.contains(".webp"))
                                && !url_lower.contains("logo")
                                && !url_lower.contains("icon")
                                && !url_lower.contains("avatar")
                                && !url_lower.contains("banner")
                            {
                                image_list.push(image_url.to_string());
                            }
                        }
                    }
                    // If we found images with this selector, use them
                    if !image_list.is_empty() {
                        break;
                    }
                }
            }

            // If no images found via CSS selectors, try parsing Next.js JSON data
            if image_list.is_empty() {
                // Look for JSON data in script tags (Next.js pattern)
                if let Some(json_start) = response.find("\"chapterImages\":[") {
                    if let Some(json_end) = response[json_start..].find("]") {
                        let json_str = &response[json_start..json_start + json_end + 1];
                        // Extract the array part
                        if let Some(array_start) = json_str.find("[") {
                            let array_str = &json_str[array_start..];
                            // Try to parse as JSON
                            if let Ok(json_value) =
                                serde_json::from_str::<serde_json::Value>(array_str)
                            {
                                if let Some(images) = json_value.as_array() {
                                    for img in images {
                                        if let Some(url) = img.get("url").and_then(|u| u.as_str()) {
                                            image_list.push(url.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let origin = reqwest::Url::parse(chapter_url)
                .ok()
                .and_then(|u| Some(format!("{}://{}", u.scheme(), u.host_str()?)))
                .unwrap_or_default();
            let base = reqwest::Url::parse(chapter_url).ok();

            for (i, image_url) in image_list.iter().enumerate() {
                let full_url = if let Some(ref b) = base {
                    b.join(image_url)
                        .map(|u| u.to_string())
                        .unwrap_or(image_url.clone())
                } else {
                    image_url.clone()
                };
                let response = client
                    .get(&full_url)
                    .header(
                        "Referer",
                        if !origin.is_empty() {
                            origin.as_str()
                        } else {
                            chapter_url
                        },
                    )
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
    }

    let cursor = zip.finish()?;

    Ok(cursor.into_inner())
}
