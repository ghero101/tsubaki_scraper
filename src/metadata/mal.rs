use reqwest::Client;
use deadpool_postgres::Pool;
use serde_json::Value;
use std::error::Error;

fn normalize(s: &str) -> String {
    s.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "")
}

pub async fn resolve_id(
    client: &Client,
    title: &str,
    alts: &str,
) -> Result<Option<i64>, Box<dyn Error>> {
    let mut candidates: Vec<String> = vec![title.to_string()];
    for t in alts.split(", ") {
        if !t.trim().is_empty() {
            candidates.push(t.trim().to_string());
        }
    }
    for q in candidates {
        let url = format!(
            "https://api.jikan.moe/v4/manga?q={}&limit=5",
            urlencoding::encode(&q)
        );
        let res = client.get(&url).send().await?;
        if !res.status().is_success() {
            continue;
        }
        let txt = res.text().await?;
        if let Ok(json) = serde_json::from_str::<Value>(&txt) {
            let target_norm = normalize(&q);
            if let Some(arr) = json.get("data").and_then(|v| v.as_array()) {
                for item in arr {
                    let title_field = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
                    let id = item.get("mal_id").and_then(|v| v.as_i64()).unwrap_or(0);
                    if id > 0 && !title_field.is_empty() && normalize(title_field) == target_norm {
                        return Ok(Some(id));
                    }
                }
            }
        }
    }
    Ok(None)
}

#[allow(dead_code)]
pub async fn sync_all(pool: &Pool, client: &Client) -> Result<usize, Box<dyn Error>> {
    let db_client = pool.get().await.expect("Failed to get connection from pool");

    // Find manga missing mal_id
    let rows = db_client.query(
        "SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE mal_id IS NULL OR mal_id = 0",
        &[],
    ).await?;

    let mut updated = 0usize;
    for row in rows {
        let manga_id: String = row.get(0);
        let title: String = row.get(1);
        let alts: String = row.get(2);

        if let Some(mal_id) = resolve_id(client, &title, &alts).await? {
            db_client.execute(
                "UPDATE manga SET mal_id = $1 WHERE id = $2",
                &[&mal_id, &manga_id],
            ).await?;
            db_client.execute(
                "INSERT INTO provider_ids (manga_id, provider, provider_id) VALUES ($1, 'mal', $2) ON CONFLICT (manga_id, provider) DO UPDATE SET provider_id = $2",
                &[&manga_id, &mal_id.to_string()],
            ).await?;
            updated += 1;
        }
    }
    Ok(updated)
}

pub async fn fetch_details(
    client: &Client,
    mal_id: i64,
) -> Result<(Option<String>, Vec<String>, Option<String>), Box<dyn Error>> {
    // returns (description, genres, rating)
    let url = format!("https://api.jikan.moe/v4/manga/{}", mal_id);
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Ok((None, vec![], None));
    }
    let txt = res.text().await?;
    let json: Value = serde_json::from_str(&txt)?;
    let data = json.get("data").cloned().unwrap_or(Value::Null);
    let desc = data
        .get("synopsis")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let mut genres = Vec::new();
    if let Some(arr) = data.get("genres").and_then(|v| v.as_array()) {
        for g in arr {
            if let Some(n) = g.get("name").and_then(|v| v.as_str()) {
                genres.push(n.to_string());
            }
        }
    }
    let rating = data
        .get("rating")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok((desc, genres, rating))
}
