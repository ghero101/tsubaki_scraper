/// Comprehensive source validation test
/// Tests all sources by collecting 10 manga and their chapters
/// This helps identify which sources are still working and which need fixes
use reqwest::Client;
use rust_manga_scraper::models::Manga;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SourceTestResult {
    source_name: String,
    status: String,
    manga_count: usize,
    total_chapters: usize,
    duration_ms: u128,
    error: Option<String>,
    sample_manga: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestReport {
    timestamp: String,
    total_sources: usize,
    working_sources: usize,
    failed_sources: usize,
    results: Vec<SourceTestResult>,
}

// Helper macro to test a source
macro_rules! test_source {
    ($source_mod:ident, $source_name:expr) => {{
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
            .build()
            .expect("Failed to create HTTP client");

        let start = Instant::now();

        match rust_manga_scraper::sources::$source_mod::search_manga_with_urls(&client, "").await {
            Ok(results) => {
                let manga_list: Vec<(Manga, String)> = results.into_iter().take(10).collect();
                let manga_count = manga_list.len();

                if manga_count == 0 {
                    SourceTestResult {
                        source_name: $source_name.to_string(),
                        status: "NO_DATA".to_string(),
                        manga_count: 0,
                        total_chapters: 0,
                        duration_ms: start.elapsed().as_millis(),
                        error: Some("No manga returned from source".to_string()),
                        sample_manga: vec![],
                    }
                } else {
                    // Try to get chapters for the first manga
                    let mut total_chapters = 0;
                    let mut sample_manga = Vec::new();

                    for (manga, url) in manga_list.iter().take(3) {
                        sample_manga.push(manga.title.clone());

                        if let Ok(chapters) =
                            rust_manga_scraper::sources::$source_mod::get_chapters(&client, &url)
                                .await
                        {
                            total_chapters += chapters.len();
                        }
                    }

                    SourceTestResult {
                        source_name: $source_name.to_string(),
                        status: "WORKING".to_string(),
                        manga_count,
                        total_chapters,
                        duration_ms: start.elapsed().as_millis(),
                        error: None,
                        sample_manga,
                    }
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                let status = if error_msg.contains("403") {
                    "FORBIDDEN"
                } else if error_msg.contains("404") {
                    "NOT_FOUND"
                } else if error_msg.contains("timeout") {
                    "TIMEOUT"
                } else if error_msg.contains("dns") || error_msg.contains("resolve") {
                    "DNS_ERROR"
                } else if error_msg.contains("ssl")
                    || error_msg.contains("tls")
                    || error_msg.contains("certificate")
                {
                    "SSL_ERROR"
                } else {
                    "ERROR"
                };

                SourceTestResult {
                    source_name: $source_name.to_string(),
                    status: status.to_string(),
                    manga_count: 0,
                    total_chapters: 0,
                    duration_ms: start.elapsed().as_millis(),
                    error: Some(error_msg),
                    sample_manga: vec![],
                }
            }
        }
    }};
}

#[tokio::test]
#[ignore] // Run with: cargo test --test source_validation_test -- --ignored --nocapture
async fn test_all_sources_comprehensive() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║         COMPREHENSIVE SOURCE VALIDATION TEST               ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let mut results = Vec::new();

    // Test each source
    println!("Testing MangaDex...");
    results.push(test_mangadex().await);

    println!("Testing FireScans...");
    results.push(test_source!(firescans, "FireScans"));

    println!("Testing ResetScans...");
    results.push(test_source!(reset_scans, "ResetScans"));

    println!("Testing RizzComic...");
    results.push(test_source!(rizzcomic, "RizzComic"));

    println!("Testing Manhuaus...");
    results.push(test_source!(manhuaus, "Manhuaus"));

    println!("Testing RokariComics...");
    results.push(test_source!(rokaricomics, "RokariComics"));

    println!("Testing StoneScape...");
    results.push(test_source!(stonescape, "StoneScape"));

    println!("Testing WitchScans...");
    results.push(test_source!(witchscans, "WitchScans"));

    println!("Testing Asmotoon...");
    results.push(test_source!(asmotoon, "Asmotoon"));

    println!("Testing HiveToons...");
    results.push(test_source!(hivetoons, "HiveToons"));

    println!("Testing KenScans...");
    results.push(test_source!(kenscans, "KenScans"));

    println!("Testing QIScans...");
    results.push(test_source!(qiscans, "QIScans"));

    println!("Testing NyxScans...");
    results.push(test_source!(nyxscans, "NyxScans"));

    println!("Testing DrakeComic...");
    results.push(test_source!(drakecomic, "DrakeComic"));

    println!("Testing MadaraScans...");
    results.push(test_source!(madarascans, "MadaraScans"));

    println!("Testing RizzFables...");
    results.push(test_source!(rizzfables, "RizzFables"));

    println!("Testing ThunderScans...");
    results.push(test_source!(thunderscans, "ThunderScans"));

    println!("Testing AsuraScans...");
    results.push(test_source!(asurascans, "AsuraScans"));

    println!("Testing SirenScans...");
    results.push(test_source!(sirenscans, "SirenScans"));

    println!("Testing VortexScans...");
    results.push(test_source!(vortexscans, "VortexScans"));

    println!("Testing GrimScans...");
    results.push(test_source!(grimscans, "GrimScans"));

    println!("Testing TempleScan...");
    results.push(test_source!(temple_scan, "TempleScan"));

    println!("Testing Kagane...");
    results.push(test_source!(kagane, "Kagane"));

    println!("Testing MavinTranslations...");
    results.push(test_source!(mavintranslations, "MavinTranslations"));

    println!("Testing KDTNovels...");
    results.push(test_kdtnovels().await);

    // Free scanlation sites
    println!("Testing FlameComics...");
    results.push(test_source!(flamecomics, "FlameComics"));

    println!("Testing DayComics...");
    results.push(test_source!(daycomics, "DayComics"));

    println!("Testing LunaToons...");
    results.push(test_source!(lunatoons, "LunaToons"));

    println!("Testing KodokuStudio...");
    results.push(test_source!(kodoku_studio, "KodokuStudio"));

    println!("Testing VASTVisual...");
    results.push(test_source!(vast_visual, "VASTVisual"));

    // Free web platforms
    println!("Testing Webtoon...");
    results.push(test_source!(webtoon, "Webtoon"));

    println!("Testing Tapas...");
    results.push(test_source!(tapas, "Tapas"));

    println!("Testing Webcomics...");
    results.push(test_source!(webcomics, "Webcomics"));

    println!("Testing MediBang...");
    results.push(test_source!(medibang, "MediBang"));

    // API/Metadata sources
    println!("Testing MyAnimeList...");
    results.push(test_source!(myanimelist, "MyAnimeList"));

    println!("Testing AniList...");
    results.push(test_source!(anilist, "AniList"));

    // Commercial publishers (stub implementations - will return NO_DATA)
    println!("Testing VizMedia...");
    results.push(test_source!(viz_media, "VizMedia"));

    println!("Testing KodanshaComics...");
    results.push(test_source!(kodansha_comics, "KodanshaComics"));

    println!("Testing YenPress...");
    results.push(test_source!(yen_press, "YenPress"));

    println!("Testing DarkHorseComics...");
    results.push(test_source!(dark_horse_comics, "DarkHorseComics"));

    println!("Testing SevenSeas...");
    results.push(test_source!(seven_seas, "SevenSeas"));

    println!("Testing JNovelClub...");
    results.push(test_source!(jnovel_club, "JNovelClub"));

    println!("Testing DenpaBooks...");
    results.push(test_source!(denpa_books, "DenpaBooks"));

    println!("Testing IrodoriComics...");
    results.push(test_source!(irodori_comics, "IrodoriComics"));

    println!("Testing OnePeaceBooks...");
    results.push(test_source!(one_peace_books, "OnePeaceBooks"));

    println!("Testing Tokyopop...");
    results.push(test_source!(tokyopop, "Tokyopop"));

    println!("Testing TitanManga...");
    results.push(test_source!(titan_manga, "TitanManga"));

    println!("Testing UdonEntertainment...");
    results.push(test_source!(udon_entertainment, "UdonEntertainment"));

    println!("Testing SquareEnixManga...");
    results.push(test_source!(square_enix_manga, "SquareEnixManga"));

    println!("Testing Kana...");
    results.push(test_source!(kana, "Kana"));

    println!("Testing Shueisha...");
    results.push(test_source!(shueisha, "Shueisha"));

    // Paid platforms (stub implementations)
    println!("Testing Lezhin...");
    results.push(test_source!(lezhin, "Lezhin"));

    println!("Testing PocketComics...");
    results.push(test_source!(pocket_comics, "PocketComics"));

    println!("Testing Toomics...");
    results.push(test_source!(toomics, "Toomics"));

    println!("Testing Tappytoon...");
    results.push(test_source!(tappytoon, "Tappytoon"));

    println!("Testing Manta...");
    results.push(test_source!(manta, "Manta"));

    println!("Testing Comikey...");
    results.push(test_source!(comikey, "Comikey"));

    println!("Testing InkrComics...");
    results.push(test_source!(inkr_comics, "InkrComics"));

    println!("Testing BookLive...");
    results.push(test_source!(booklive, "BookLive"));

    println!("Testing Fakku...");
    results.push(test_source!(fakku, "Fakku"));

    println!("Testing Others...");
    results.push(test_source!(others, "Others"));

    // Generate report
    let working = results.iter().filter(|r| r.status == "WORKING").count();
    let total = results.len();

    let report = TestReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_sources: total,
        working_sources: working,
        failed_sources: total - working,
        results: results.clone(),
    };

    // Print summary table
    print_summary_table(&results);

    // Save report to JSON
    if let Ok(json) = serde_json::to_string_pretty(&report) {
        if let Err(e) = std::fs::write("source_validation_report.json", json) {
            eprintln!("Failed to write report: {}", e);
        } else {
            println!("\n📊 Full report saved to: source_validation_report.json");
        }
    }

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                    FINAL RESULTS                           ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!(
        "║  ✅ Working Sources:  {:2}/{:2} ({:3.0}%)                      ║",
        working,
        total,
        (working as f32 / total as f32) * 100.0
    );
    println!(
        "║  ❌ Failed Sources:   {:2}/{:2}                               ║",
        total - working,
        total
    );
    println!("╚════════════════════════════════════════════════════════════╝\n");
}

async fn test_kdtnovels() -> SourceTestResult {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()
        .expect("Failed to create HTTP client");

    let start = Instant::now();

    // KDTNovels doesn't have get_chapters, just search
    match rust_manga_scraper::sources::kdtnovels::search_manga_with_urls(&client, "").await {
        Ok(results) => {
            let manga_count = results.len().min(10);
            let sample_manga: Vec<String> = results
                .iter()
                .take(3)
                .map(|(m, _)| m.title.clone())
                .collect();

            if manga_count == 0 {
                SourceTestResult {
                    source_name: "KDTNovels".to_string(),
                    status: "NO_DATA".to_string(),
                    manga_count: 0,
                    total_chapters: 0,
                    duration_ms: start.elapsed().as_millis(),
                    error: Some("No novels returned from source".to_string()),
                    sample_manga: vec![],
                }
            } else {
                SourceTestResult {
                    source_name: "KDTNovels".to_string(),
                    status: "WORKING".to_string(),
                    manga_count,
                    total_chapters: 0, // No chapters for novel source
                    duration_ms: start.elapsed().as_millis(),
                    error: None,
                    sample_manga,
                }
            }
        }
        Err(e) => SourceTestResult {
            source_name: "KDTNovels".to_string(),
            status: "ERROR".to_string(),
            manga_count: 0,
            total_chapters: 0,
            duration_ms: start.elapsed().as_millis(),
            error: Some(e.to_string()),
            sample_manga: vec![],
        },
    }
}

async fn test_mangadex() -> SourceTestResult {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()
        .expect("Failed to create HTTP client");

    let start = Instant::now();

    match rust_manga_scraper::sources::mangadex::search_all_manga(
        &client,
        rust_manga_scraper::sources::mangadex::BASE_URL,
    )
    .await
    {
        Ok(results) => {
            let manga_list: Vec<Manga> = results.into_iter().take(10).collect();
            let manga_count = manga_list.len();

            let mut total_chapters = 0;
            let mut sample_manga = Vec::new();

            for manga in manga_list.iter().take(3) {
                sample_manga.push(manga.title.clone());

                if let Ok(chapters) =
                    rust_manga_scraper::sources::mangadex::get_chapters(&client, &manga.id).await
                {
                    total_chapters += chapters.len();
                }
            }

            SourceTestResult {
                source_name: "MangaDex".to_string(),
                status: if manga_count > 0 {
                    "WORKING"
                } else {
                    "NO_DATA"
                }
                .to_string(),
                manga_count,
                total_chapters,
                duration_ms: start.elapsed().as_millis(),
                error: None,
                sample_manga,
            }
        }
        Err(e) => SourceTestResult {
            source_name: "MangaDex".to_string(),
            status: "ERROR".to_string(),
            manga_count: 0,
            total_chapters: 0,
            duration_ms: start.elapsed().as_millis(),
            error: Some(e.to_string()),
            sample_manga: vec![],
        },
    }
}

fn print_summary_table(results: &[SourceTestResult]) {
    println!("\n╔═══════════════════╦══════════════╦═══════════╦═══════════╦═══════════╗");
    println!("║ Source            ║ Status       ║ Manga     ║ Chapters  ║ Time (ms) ║");
    println!("╠═══════════════════╬══════════════╬═══════════╬═══════════╬═══════════╣");

    for result in results {
        let status_symbol = match result.status.as_str() {
            "WORKING" => "✅",
            "NO_DATA" => "⚠️ ",
            "FORBIDDEN" => "🚫",
            "NOT_FOUND" => "❌",
            "TIMEOUT" => "⏱️ ",
            "DNS_ERROR" => "🌐",
            "SSL_ERROR" => "🔒",
            _ => "❓",
        };

        println!(
            "║ {:<17} ║ {}{:<11} ║ {:>9} ║ {:>9} ║ {:>9} ║",
            truncate(&result.source_name, 17),
            status_symbol,
            truncate(&result.status, 10),
            result.manga_count,
            result.total_chapters,
            result.duration_ms
        );

        if result.status == "WORKING" && !result.sample_manga.is_empty() {
            for (i, manga_title) in result.sample_manga.iter().enumerate().take(2) {
                if i == 0 {
                    println!("║   └─ {}...", truncate(manga_title, 60));
                } else {
                    println!("║      {}...", truncate(manga_title, 60));
                }
            }
        } else if let Some(err) = &result.error {
            let err_msg = truncate(err, 70);
            println!("║   └─ Error: {}...", err_msg);
        }
    }

    println!("╚═══════════════════╩══════════════╩═══════════╩═══════════╩═══════════╝");
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
