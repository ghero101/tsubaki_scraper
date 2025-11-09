use reqwest::Client;
use rusqlite::Connection;
use std::error::Error;

// Combine metadata from providers into manga.description, manga.tags, manga.rating
#[allow(dead_code)]
pub async fn sync_all(conn: &Connection, client: &Client) -> Result<usize, Box<dyn Error>> {
    // Ensure provider IDs exist first
    let _ = super::mangabaka::sync_all(conn, client).await;
    let _ = super::mal::sync_all(conn, client).await;
    let _ = super::anilist::sync_all(conn, client).await;
    merge_only(conn, client).await
}

// Merge using existing provider IDs without running provider syncs
pub async fn merge_only(conn: &Connection, client: &Client) -> Result<usize, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT id, COALESCE(mangabaka_id,''), COALESCE(mal_id,0), COALESCE(anilist_id,0), title FROM manga")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;
    let mut updated = 0usize;
    for row in rows {
        let (manga_id, _mangabaka_id, mal_id, anilist_id, _title) = row?;
        let mut descr: Option<String> = None;
        let mut tags: Vec<String> = Vec::new();
        let mut rating: Option<String> = None;
        // MangaBaka details
        if !_mangabaka_id.is_empty() {
            if let Ok((d, g)) = super::mangabaka::fetch_details(client, &_mangabaka_id).await {
                if descr.is_none() {
                    descr = d;
                }
                if !g.is_empty() {
                    tags.extend(g);
                }
            }
        }
        if mal_id > 0 {
            if let Ok((d, g, r)) = super::mal::fetch_details(client, mal_id).await {
                if descr.is_none() {
                    descr = d;
                }
                if !g.is_empty() {
                    tags.extend(g);
                }
                if rating.is_none() {
                    rating = r;
                }
            }
        }
        if anilist_id > 0 {
            if let Ok((d, g, adult)) = super::anilist::fetch_details(client, anilist_id).await {
                if descr.is_none() {
                    descr = d;
                }
                if !g.is_empty() {
                    tags.extend(g);
                }
                if rating.is_none() {
                    if let Some(is_adult) = adult {
                        if is_adult {
                            rating = Some("erotica".to_string());
                        }
                    }
                }
            }
        }
        tags.sort();
        tags.dedup();
        let tags_str = if tags.is_empty() {
            None
        } else {
            Some(tags.join(", "))
        };
        let desc_str = descr.as_deref().filter(|s| !s.trim().is_empty());
        let rating_str = rating.as_deref();
        if desc_str.is_some() || tags_str.is_some() || rating_str.is_some() {
            conn.execute(
                "UPDATE manga SET description = COALESCE(?1, description), tags = COALESCE(?2, tags), rating = COALESCE(?3, rating) WHERE id = ?4",
                rusqlite::params![desc_str, tags_str, rating_str, manga_id],
            )?;
            updated += 1;
        }
    }
    Ok(updated)
}
