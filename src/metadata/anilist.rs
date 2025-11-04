use reqwest::Client;
use rusqlite::Connection;
use serde_json::Value;
use std::error::Error;

fn normalize(s: &str) -> String { s.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "") }

pub async fn resolve_id(client: &Client, title: &str, alts: &str) -> Result<Option<i64>, Box<dyn Error>> {
    let mut candidates: Vec<String> = vec![title.to_string()];
    for t in alts.split(", ") { if !t.trim().is_empty() { candidates.push(t.trim().to_string()); } }
    for q in candidates {
        let query = serde_json::json!({
            "query": "query ($search: String) { Media(search: $search, type: MANGA) { id title { romaji english native } isAdult } }",
            "variables": {"search": q}
        });
        let res = client.post("https://graphql.anilist.co").json(&query).send().await?;
        if !res.status().is_success() { continue; }
        let txt = res.text().await?;
        if let Ok(json) = serde_json::from_str::<Value>(&txt) {
            if let Some(media) = json.get("data").and_then(|d| d.get("Media")) {
                let mtitle = media.get("title").and_then(|t| t.get("english").or_else(|| t.get("romaji")).or_else(|| t.get("native"))).and_then(|v| v.as_str()).unwrap_or("");
                let id = media.get("id").and_then(|v| v.as_i64());
                if let (Some(id), true) = (id, !mtitle.is_empty()) { if normalize(mtitle) == normalize(&q) { return Ok(Some(id)); } }
            }
        }
    }
    Ok(None)
}

pub async fn sync_all(conn: &Connection, client: &Client) -> Result<usize, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE anilist_id IS NULL OR anilist_id = 0")?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?)))?;
    let mut updated = 0usize;
    for row in rows {
        let (manga_id, title, alts) = row?;
        if let Some(aid) = resolve_id(client, &title, &alts).await? {
            conn.execute("UPDATE manga SET anilist_id = ?1 WHERE id = ?2", rusqlite::params![aid, manga_id])?;
            conn.execute("INSERT OR REPLACE INTO provider_ids (manga_id, provider, provider_id) VALUES (?1,'anilist',?2)", rusqlite::params![manga_id, aid.to_string()])?;
            updated += 1;
        }
    }
    Ok(updated)
}

pub async fn fetch_details(client: &Client, anilist_id: i64) -> Result<(Option<String>, Vec<String>, Option<bool>), Box<dyn Error>> {
    // returns (description, genres, isAdult)
    let query = serde_json::json!({
        "query": "query ($id: Int) { Media(id: $id, type: MANGA) { id description(asHtml: false) genres isAdult } }",
        "variables": {"id": anilist_id}
    });
    let res = client.post("https://graphql.anilist.co").json(&query).send().await?;
    if !res.status().is_success() { return Ok((None, vec![], None)); }
    let txt = res.text().await?;
    let json: Value = serde_json::from_str(&txt)?;
    let media = json.get("data").and_then(|d| d.get("Media")).cloned().unwrap_or(Value::Null);
    let desc = media.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let genres = media.get("genres").and_then(|v| v.as_array()).map(|arr| arr.iter().filter_map(|g| g.as_str().map(|s| s.to_string())).collect()).unwrap_or_else(Vec::new);
    let is_adult = media.get("isAdult").and_then(|v| v.as_bool());
    Ok((desc, genres, Some(is_adult.unwrap_or(false))))
}
