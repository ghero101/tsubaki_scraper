use rust_manga_scraper::browser::{BrowserConfig, BrowserManager};
use rust_manga_scraper::sources::{asmotoon_browser, mavintranslations_browser, qiscans_browser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Multiple Next.js Sources with Browser Automation ===\n");

    let config = BrowserConfig::default();
    let manager = BrowserManager::new(config)?;
    println!("✓ Browser manager created\n");

    let sources = vec![
        ("QIScans", "https://qiscans.com"),
        ("MavinTranslations", "https://mavintranslations.com"),
        ("Asmotoon", "https://asmotoon.com"),
    ];

    let mut results_summary = Vec::new();

    for (name, base_url) in &sources {
        println!("=== Testing {} ===", name);

        // Test based on source
        let (manga_count, chapter_count) = match name.as_ref() {
            "QIScans" => match qiscans_browser::search_manga_with_urls_browser(&manager, "") {
                Ok(results) => {
                    let manga = results.len();
                    println!("  ✓ Found {} manga", manga);

                    let mut chapters = 0;
                    for (_, url) in results.iter().take(2) {
                        if let Ok(ch) = qiscans_browser::get_chapters_browser(&manager, url) {
                            chapters += ch.len();
                        }
                    }
                    (manga, chapters)
                }
                Err(e) => {
                    println!("  ✗ Error: {}", e);
                    (0, 0)
                }
            },
            "MavinTranslations" => {
                match mavintranslations_browser::search_manga_with_urls_browser(&manager, "") {
                    Ok(results) => {
                        let manga = results.len();
                        println!("  ✓ Found {} manga", manga);

                        let mut chapters = 0;
                        for (_, url) in results.iter().take(2) {
                            if let Ok(ch) =
                                mavintranslations_browser::get_chapters_browser(&manager, url)
                            {
                                chapters += ch.len();
                            }
                        }
                        (manga, chapters)
                    }
                    Err(e) => {
                        println!("  ✗ Error: {}", e);
                        (0, 0)
                    }
                }
            }
            "Asmotoon" => match asmotoon_browser::search_manga_with_urls_browser(&manager, "") {
                Ok(results) => {
                    let manga = results.len();
                    println!("  ✓ Found {} manga", manga);

                    let mut chapters = 0;
                    for (_, url) in results.iter().take(2) {
                        if let Ok(ch) = asmotoon_browser::get_chapters_browser(&manager, url) {
                            chapters += ch.len();
                        }
                    }
                    (manga, chapters)
                }
                Err(e) => {
                    println!("  ✗ Error: {}", e);
                    (0, 0)
                }
            },
            _ => (0, 0),
        };

        println!("  Chapters from 2 manga: {}\n", chapter_count);
        results_summary.push((name.to_string(), manga_count, chapter_count));
    }

    println!("\n============================================");
    println!("SUMMARY:");
    println!("============================================");
    for (name, manga, chapters) in &results_summary {
        let status = if *chapters > 2 { "✅" } else { "⚠️" };
        println!(
            "{}  {}: {} manga, {} chapters (from 2 manga)",
            status, name, manga, chapters
        );
    }
    println!("============================================");

    Ok(())
}
