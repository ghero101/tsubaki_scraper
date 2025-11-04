use reqwest::Client;
use rusqlite::Connection;
use serde_json::Value;
use std::error::Error;

fn normalize(s: &str) -> String {
    s.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "")
}

pub async fn resolve_id(client: &Client, title: &str, alts: &str) -> Result<Option<String>, Box<dyn Error>> {
    let mut candidates: Vec<String> = vec![title.to_string()];
    for t in alts.split(", ") { if !t.trim().is_empty() { candidates.push(t.trim().to_string()); } }
    for q in candidates {
        let url = format!("https://mangabaka.dev/api/search?query={}", urlencoding::encode(&q));
        let res = client.get(&url).send().await?;
        if !res.status().is_success() { continue; }
        let txt = res.text().await?;
        if let Ok(json) = serde_json::from_str::<Value>(&txt) {
            let target_norm = normalize(&q);
            let arr_opt = json.get("results").and_then(|v| v.as_array()).or_else(|| json.get("data").and_then(|v| v.as_array()));
            if let Some(arr) = arr_opt {
                for item in arr {
                    let name = item.get("name").or_else(|| item.get("title")).and_then(|v| v.as_str()).unwrap_or("");
                    let id = item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
                        .or_else(|| item.get("id").and_then(|v| v.as_i64().map(|n| n.to_string())))
                        .unwrap_or_default();
                    if !id.is_empty() && !name.is_empty() && normalize(name) == target_norm { return Ok(Some(id)); }
                }
            }
        }
    }
    Ok(None)
}

pub async fn fetch_details(client: &Client, pid: &str) -> Result<(Option<String>, Vec<String>), Box<dyn Error>> {
    // best effort; ignore errors
    let url = format!("https://mangabaka.dev/api/manga/{}", pid);
    let res = client.get(&url).send().await?;
    if !res.status().is_success() { return Ok((None, vec![])); }
    let txt = res.text().await?;
    let json: Value = serde_json::from_str(&txt).unwrap_or(Value::Null);
    let desc = json.get("description").or_else(|| json.get("synopsis")).and_then(|v| v.as_str()).map(|s| s.to_string());
    let mut genres = Vec::new();
    if let Some(arr) = json.get("genres").and_then(|v| v.as_array()) { for g in arr { if let Some(n)=g.as_str() { genres.push(n.to_string()); } else if let Some(n)=g.get("name").and_then(|v| v.as_str()) { genres.push(n.to_string()); } } }
    Ok((desc, genres))
}

pub async fn sync_all(conn: &Connection, client: &Client) -> Result<usize, Box<dyn Error>> {
    // Get manga needing mangabaka_id
    let mut stmt = conn.prepare(
        "SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE mangabaka_id IS NULL OR mangabaka_id = ''",
    )?;
    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let alts: String = row.get(2)?;
        Ok((id, title, alts))
    })?;

    let mut updated = 0usize;
    for row in rows {
        let (manga_id, title, alts) = row?;
        let mut candidates: Vec<String> = vec![title.clone()];
        for t in alts.split(", ") {
            if !t.trim().is_empty() {
                candidates.push(t.trim().to_string());
            }
        }
        let mut found: Option<String> = None;
        for q in candidates {
            let url = format!("https://mangabaka.dev/api/search?query={}", urlencoding::encode(&q));
            let res = client.get(&url).send().await?;
            if !res.status().is_success() { continue; }
            let txt = res.text().await?;
            if let Ok(json) = serde_json::from_str::<Value>(&txt) {
                let target_norm = normalize(&q);
                let arr_opt = json.get("results").and_then(|v| v.as_array()).or_else(|| json.get("data").and_then(|v| v.as_array()));
                if let Some(arr) = arr_opt {
                    for item in arr {
                        let name = item.get("name").or_else(|| item.get("title")).and_then(|v| v.as_str()).unwrap_or("");
let id = item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
    .or_else(|| item.get("id").and_then(|v| v.as_i64().map(|n| n.to_string())))
    .unwrap_or_default();
                        if !id.is_empty() && !name.is_empty() {
                            if normalize(name) == target_norm {
                                found = Some(id);
                                break;
                            }
                        }
                    }
                }
            }
            if found.is_some() { break; }
        }
        if let Some(pid) = found {
            let _ = conn.execute(
                "UPDATE manga SET mangabaka_id = ?1 WHERE id = ?2",
                rusqlite::params![pid, manga_id],
            )?;
            let _ = conn.execute(
                "INSERT OR REPLACE INTO provider_ids (manga_id, provider, provider_id) VALUES (?1, 'mangabaka', ?2)",
                rusqlite::params![manga_id, pid],
            )?;
            updated += 1;
        }
    }

    Ok(updated)
}
