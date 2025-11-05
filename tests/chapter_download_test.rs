/// Chapter download validation test
/// Tests downloading 1 chapter from each source
/// This validates that the actual scraping and downloading works end-to-end

use reqwest::Client;
use rust_manga_scraper::models::{Manga, Chapter, Source};
use rust_manga_scraper::scraper::download_chapter;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DownloadTestResult {
    source_name: String,
    status: String,
    manga_title: String,
    chapter_number: String,
    file_size_kb: Option<u64>,
    duration_ms: u128,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DownloadTestReport {
    timestamp: String,
    total_sources_tested: usize,
    successful_downloads: usize,
    failed_downloads: usize,
    results: Vec<DownloadTestResult>,
}

// Helper macro to test chapter download for a source
macro_rules! test_download {
    ($source_mod:ident, $source_name:expr, $source_id:expr) => {{
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        let start = Instant::now();

        // First, get some manga
        match rust_manga_scraper::sources::$source_mod::search_manga_with_urls(&client, "").await {
            Ok(results) => {
                if results.is_empty() {
                    DownloadTestResult {
                        source_name: $source_name.to_string(),
                        status: "NO_MANGA".to_string(),
                        manga_title: String::new(),
                        chapter_number: String::new(),
                        file_size_kb: None,
                        duration_ms: start.elapsed().as_millis(),
                        error: Some("No manga available to test".to_string()),
                    }
                } else {
                    // Try first manga that has chapters
                    let mut found = false;
                    let mut result = DownloadTestResult {
                        source_name: $source_name.to_string(),
                        status: "NO_CHAPTERS".to_string(),
                        manga_title: String::new(),
                        chapter_number: String::new(),
                        file_size_kb: None,
                        duration_ms: start.elapsed().as_millis(),
                        error: Some("No chapters found in any manga".to_string()),
                    };

                    for (manga, series_url) in results.iter().take(5) {
                        if found { break; }

                        // Get chapters for this manga
                        match rust_manga_scraper::sources::$source_mod::get_chapters(&client, &series_url).await {
                            Ok(chapters) => {
                                if !chapters.is_empty() {
                                    // Try to download the first chapter
                                    let chapter = &chapters[0];
                                    let manga_title = &manga.title;
                                    let chapter_number = &chapter.chapter_number;
                                    let chapter_url = &chapter.url;

                                    let test_dir = "test_downloads";
                                    std::fs::create_dir_all(test_dir).ok();

                                    match download_chapter(
                                        &client,
                                        $source_id,
                                        chapter_url,
                                        manga_title,
                                        chapter_number,
                                        test_dir,
                                        None,
                                    ).await {
                                        Ok(file_path) => {
                                            let file_size_kb = std::fs::metadata(&file_path)
                                                .ok()
                                                .map(|m| m.len() / 1024);

                                            result = DownloadTestResult {
                                                source_name: $source_name.to_string(),
                                                status: "SUCCESS".to_string(),
                                                manga_title: manga_title.clone(),
                                                chapter_number: chapter_number.clone(),
                                                file_size_kb,
                                                duration_ms: start.elapsed().as_millis(),
                                                error: None,
                                            };
                                            found = true;

                                            // Clean up test file
                                            std::fs::remove_file(&file_path).ok();
                                        }
                                        Err(e) => {
                                            result = DownloadTestResult {
                                                source_name: $source_name.to_string(),
                                                status: "DOWNLOAD_FAILED".to_string(),
                                                manga_title: manga_title.clone(),
                                                chapter_number: chapter_number.clone(),
                                                file_size_kb: None,
                                                duration_ms: start.elapsed().as_millis(),
                                                error: Some(e.to_string()),
                                            };
                                            // Try next manga instead of breaking
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Try next manga
                                continue;
                            }
                        }
                    }
                    result
                }
            }
            Err(e) => DownloadTestResult {
                source_name: $source_name.to_string(),
                status: "SOURCE_ERROR".to_string(),
                manga_title: String::new(),
                chapter_number: String::new(),
                file_size_kb: None,
                duration_ms: start.elapsed().as_millis(),
                error: Some(e.to_string()),
            },
        }
    }};
}

#[tokio::test]
#[ignore] // Run with: cargo test --test chapter_download_test -- --ignored --nocapture
async fn test_all_chapter_downloads() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║        COMPREHENSIVE CHAPTER DOWNLOAD TEST                ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let mut results = Vec::new();

    // Test each source
    println!("Testing MangaDex download...");
    results.push(test_download_mangadex().await);

    println!("Testing FireScans download...");
    results.push(test_download!(firescans, "FireScans", Source::FireScans as i32));

    println!("Testing ResetScans download...");
    results.push(test_download!(reset_scans, "ResetScans", Source::ResetScans as i32));

    println!("Testing RizzComic download...");
    results.push(test_download!(rizzcomic, "RizzComic", Source::RizzComic as i32));

    println!("Testing Asmotoon download...");
    results.push(test_download!(asmotoon, "Asmotoon", Source::Asmotoon as i32));

    println!("Testing DrakeComic download...");
    results.push(test_download!(drakecomic, "DrakeComic", Source::DrakeComic as i32));

    println!("Testing Kagane download...");
    results.push(test_download!(kagane, "Kagane", Source::Kagane as i32));

    println!("Testing TempleScan download...");
    results.push(test_download!(temple_scan, "TempleScan", Source::TempleScan as i32));

    println!("Testing ThunderScans download...");
    results.push(test_download!(thunderscans, "ThunderScans", Source::ThunderScans as i32));

    // Clean up test directory
    std::fs::remove_dir_all("test_downloads").ok();

    // Generate report
    let successful = results.iter().filter(|r| r.status == "SUCCESS").count();
    let total = results.len();

    let report = DownloadTestReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_sources_tested: total,
        successful_downloads: successful,
        failed_downloads: total - successful,
        results: results.clone(),
    };

    // Print summary table
    print_download_summary(&results);

    // Save report to JSON
    if let Ok(json) = serde_json::to_string_pretty(&report) {
        if let Err(e) = std::fs::write("chapter_download_report.json", json) {
            eprintln!("Failed to write report: {}", e);
        } else {
            println!("\n📊 Full report saved to: chapter_download_report.json");
        }
    }

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                DOWNLOAD TEST RESULTS                       ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  ✅ Successful Downloads: {:2}/{:2} ({:3.0}%)                   ║",
        successful, total, (successful as f32 / total as f32) * 100.0);
    println!("║  ❌ Failed Downloads:     {:2}/{:2}                            ║",
        total - successful, total);
    println!("╚════════════════════════════════════════════════════════════╝\n");
}

async fn test_download_mangadex() -> DownloadTestResult {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to create HTTP client");

    let start = Instant::now();

    match rust_manga_scraper::sources::mangadex::search_all_manga(&client, rust_manga_scraper::sources::mangadex::BASE_URL).await {
        Ok(results) => {
            if results.is_empty() {
                return DownloadTestResult {
                    source_name: "MangaDex".to_string(),
                    status: "NO_MANGA".to_string(),
                    manga_title: String::new(),
                    chapter_number: String::new(),
                    file_size_kb: None,
                    duration_ms: start.elapsed().as_millis(),
                    error: Some("No manga available".to_string()),
                };
            }

            // Try first few manga
            for manga in results.iter().take(5) {
                match rust_manga_scraper::sources::mangadex::get_chapters(&client, &manga.id).await {
                    Ok(chapters) => {
                        if !chapters.is_empty() {
                            let chapter = &chapters[0];
                            let manga_title = &manga.title;
                            let chapter_number = &chapter.chapter_number;
                            let chapter_url = &chapter.url;

                            let test_dir = "test_downloads";
                            std::fs::create_dir_all(test_dir).ok();

                            match download_chapter(
                                &client,
                                Source::MangaDex as i32,
                                chapter_url,
                                manga_title,
                                chapter_number,
                                test_dir,
                                None,
                            ).await {
                                Ok(file_path) => {
                                    let file_size_kb = std::fs::metadata(&file_path)
                                        .ok()
                                        .map(|m| m.len() / 1024);

                                    std::fs::remove_file(&file_path).ok();

                                    return DownloadTestResult {
                                        source_name: "MangaDex".to_string(),
                                        status: "SUCCESS".to_string(),
                                        manga_title: manga_title.clone(),
                                        chapter_number: chapter_number.clone(),
                                        file_size_kb,
                                        duration_ms: start.elapsed().as_millis(),
                                        error: None,
                                    };
                                }
                                Err(_) => continue,
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }

            DownloadTestResult {
                source_name: "MangaDex".to_string(),
                status: "NO_CHAPTERS".to_string(),
                manga_title: String::new(),
                chapter_number: String::new(),
                file_size_kb: None,
                duration_ms: start.elapsed().as_millis(),
                error: Some("No downloadable chapters found".to_string()),
            }
        }
        Err(e) => DownloadTestResult {
            source_name: "MangaDex".to_string(),
            status: "SOURCE_ERROR".to_string(),
            manga_title: String::new(),
            chapter_number: String::new(),
            file_size_kb: None,
            duration_ms: start.elapsed().as_millis(),
            error: Some(e.to_string()),
        },
    }
}

fn print_download_summary(results: &[DownloadTestResult]) {
    println!("\n╔═══════════════════╦══════════════════╦══════════════════════════════╦══════════╦════════════╗");
    println!("║ Source            ║ Status           ║ Manga / Chapter              ║ Size (KB)║ Time (ms)  ║");
    println!("╠═══════════════════╬══════════════════╬══════════════════════════════╬══════════╬════════════╣");

    for result in results {
        let status_symbol = match result.status.as_str() {
            "SUCCESS" => "✅",
            "NO_MANGA" => "⚠️ ",
            "NO_CHAPTERS" => "⚠️ ",
            "DOWNLOAD_FAILED" => "❌",
            "SOURCE_ERROR" => "🚫",
            _ => "❓",
        };

        let size_str = if let Some(kb) = result.file_size_kb {
            format!("{:>8}", kb)
        } else {
            "   N/A  ".to_string()
        };

        println!("║ {:<17} ║ {}{:<15} ║ {:<28} ║ {} ║ {:>10} ║",
            truncate(&result.source_name, 17),
            status_symbol,
            truncate(&result.status, 14),
            if result.manga_title.is_empty() { "-".to_string() } else { truncate(&result.manga_title, 28) },
            size_str,
            result.duration_ms
        );

        if !result.chapter_number.is_empty() {
            println!("║                   ║                  ║   └─ Chapter: {:<17} ║          ║            ║",
                truncate(&result.chapter_number, 17));
        }

        if let Some(err) = &result.error {
            let err_msg = truncate(err, 70);
            println!("║                   └─ Error: {}...", err_msg);
        }
    }

    println!("╚═══════════════════╩══════════════════╩══════════════════════════════╩══════════╩════════════╝");
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
