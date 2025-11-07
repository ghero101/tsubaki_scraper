use rust_manga_scraper::browser::{BrowserConfig, BrowserManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing QIScans Domain ===\n");

    let config = BrowserConfig::default();
    let manager = BrowserManager::new(config)?;

    // Test .com domain
    println!("Testing https://qiscans.com/series ...");
    let tab_com = manager.new_tab()?;
    match tab_com.navigate_to("https://qiscans.com/series") {
        Ok(_) => {
            std::thread::sleep(std::time::Duration::from_secs(2));
            match tab_com.get_content() {
                Ok(html) => println!("  ✓ .com works! HTML length: {}", html.len()),
                Err(e) => println!("  ✗ .com error getting content: {}", e),
            }
        }
        Err(e) => println!("  ✗ .com navigation error: {}", e),
    }

    // Test .org domain
    println!("\nTesting https://qiscans.org/series ...");
    let tab_org = manager.new_tab()?;
    match tab_org.navigate_to("https://qiscans.org/series") {
        Ok(_) => {
            std::thread::sleep(std::time::Duration::from_secs(2));
            match tab_org.get_content() {
                Ok(html) => println!("  ✓ .org works! HTML length: {}", html.len()),
                Err(e) => println!("  ✗ .org error getting content: {}", e),
            }
        }
        Err(e) => println!("  ✗ .org navigation error: {}", e),
    }

    println!("\n✅ Domain test complete");
    Ok(())
}
