use crate::models::{Chapter, Manga, MangaSourceData};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use log::{error, info};
use tokio_postgres::{NoTls, Error as PgError};

/// Creates and returns a PostgreSQL connection pool
pub fn create_pool() -> Pool {
    info!("Creating PostgreSQL connection pool...");

    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.dbname = Some("manga_scraper".to_string());
    cfg.user = Some("manga_admin".to_string());
    cfg.password = Some("manga_password".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create connection pool");

    info!("PostgreSQL connection pool created successfully");
    pool
}

/// Get paginated manga list with optional filtering
pub async fn get_manga_paginated(
    pool: &Pool,
    limit: Option<i32>,
    offset: Option<i32>,
    sort_by: &str,
    rating_filter: Option<&str>,
) -> Result<Vec<Manga>, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let sort_expr = match sort_by {
        "rating" => "CASE lower(coalesce(rating,'')) WHEN 'safe' THEN 1 WHEN 'suggestive' THEN 2 WHEN 'erotica' THEN 3 WHEN 'pornographic' THEN 4 ELSE 5 END, title",
        _ => "title",
    };

    let mut query = format!(
        "SELECT id, title, alt_titles, cover_url, description, tags, rating, monitored, check_interval_secs, discover_interval_secs, last_chapter_check, last_discover_check FROM manga"
    );

    let limit_val = limit.unwrap_or(100);
    let offset_val = offset.unwrap_or(0);

    if let Some(r) = rating_filter {
        query.push_str(&format!(" WHERE lower(rating) = lower('{}') ORDER BY {} LIMIT $1 OFFSET $2", r, sort_expr));
        let rows = client.query(&query, &[&limit_val, &offset_val]).await?;

        let manga_list: Vec<Manga> = rows.iter().map(|row| Manga {
            id: row.get(0),
            title: row.get(1),
            alt_titles: row.get(2),
            cover_url: row.get(3),
            description: row.get(4),
            tags: row.get(5),
            rating: row.get(6),
            monitored: row.get(7),
            check_interval_secs: row.get(8),
            discover_interval_secs: row.get(9),
            last_chapter_check: row.get(10),
            last_discover_check: row.get(11),
        }).collect();

        return Ok(manga_list);
    } else {
        query.push_str(&format!(" ORDER BY {} LIMIT $1 OFFSET $2", sort_expr));
        let rows = client.query(&query, &[&limit_val, &offset_val]).await?;

        let manga_list: Vec<Manga> = rows.iter().map(|row| Manga {
            id: row.get(0),
            title: row.get(1),
            alt_titles: row.get(2),
            cover_url: row.get(3),
            description: row.get(4),
            tags: row.get(5),
            rating: row.get(6),
            monitored: row.get(7),
            check_interval_secs: row.get(8),
            discover_interval_secs: row.get(9),
            last_chapter_check: row.get(10),
            last_discover_check: row.get(11),
        }).collect();

        Ok(manga_list)
    }
}

/// Get total manga count
pub async fn get_manga_count(pool: &Pool) -> Result<i64, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let row = client.query_one("SELECT COUNT(*) FROM manga", &[]).await?;
    let count: i64 = row.get(0);
    Ok(count)
}

/// Get total chapter count
pub async fn get_chapter_count(pool: &Pool) -> Result<i64, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let row = client.query_one("SELECT COUNT(*) FROM chapters", &[]).await?;
    let count: i64 = row.get(0);
    Ok(count)
}

/// Get total source count
pub async fn get_source_count(pool: &Pool) -> Result<i64, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let row = client.query_one("SELECT COUNT(*) FROM sources", &[]).await?;
    let count: i64 = row.get(0);
    Ok(count)
}

#[derive(Debug, serde::Serialize)]
pub struct PerSourceCounts {
    pub source_id: i32,
    pub source_name: String,
    pub manga: i64,
    pub chapters: i64,
}

/// Get per-source manga and chapter counts
pub async fn get_per_source_counts(pool: &Pool) -> Result<Vec<PerSourceCounts>, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let sql = r#"
        SELECT s.id, s.name,
            (SELECT COUNT(DISTINCT msd.manga_id) FROM manga_source_data msd WHERE msd.source_id = s.id) AS manga_cnt,
            (SELECT COUNT(*) FROM chapters c JOIN manga_source_data msd ON c.manga_source_data_id = msd.id WHERE msd.source_id = s.id) AS ch_cnt
        FROM sources s ORDER BY s.id
    "#;

    let rows = client.query(sql, &[]).await?;

    let counts: Vec<PerSourceCounts> = rows.iter().map(|row| PerSourceCounts {
        source_id: row.get(0),
        source_name: row.get(1),
        manga: row.get(2),
        chapters: row.get(3),
    }).collect();

    Ok(counts)
}

/// Get source name by ID
pub async fn get_source_name(pool: &Pool, source_id: i32) -> Result<String, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let row = client.query_one("SELECT name FROM sources WHERE id = $1", &[&source_id]).await?;
    let name: String = row.get(0);
    Ok(name)
}

/// Search manga with pagination and filtering
pub async fn search_manga_paginated(
    pool: &Pool,
    query: &str,
    tags: Option<&str>,
    rating: Option<&str>,
    limit: Option<i32>,
    offset: Option<i32>,
    sort_by: &str,
) -> Result<Vec<Manga>, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let search_pattern = format!("%{}%", query);
    let sort_expr = match sort_by {
        "rating" => "CASE lower(coalesce(rating,'')) WHEN 'safe' THEN 1 WHEN 'suggestive' THEN 2 WHEN 'erotica' THEN 3 WHEN 'pornographic' THEN 4 ELSE 5 END, title",
        _ => "title",
    };

    let mut where_clauses: Vec<String> = vec!["(title ILIKE $1 OR alt_titles ILIKE $2)".to_string()];
    let mut param_count = 2;

    if tags.is_some() {
        param_count += 1;
        where_clauses.push(format!("tags ILIKE ${}", param_count));
    }

    if rating.is_some() {
        param_count += 1;
        where_clauses.push(format!("lower(rating) = lower(${})", param_count));
    }

    let sql = format!(
        "SELECT id, title, alt_titles, cover_url, description, tags, rating, monitored, check_interval_secs, discover_interval_secs, last_chapter_check, last_discover_check FROM manga WHERE {} ORDER BY {} LIMIT ${} OFFSET ${}",
        where_clauses.join(" AND "),
        sort_expr,
        param_count + 1,
        param_count + 2
    );

    let limit_val = limit.unwrap_or(100);
    let offset_val = offset.unwrap_or(0);

    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![&search_pattern, &search_pattern];

    let tag_pattern;
    if let Some(tag_filter) = tags {
        tag_pattern = format!("%{}%", tag_filter);
        params.push(&tag_pattern);
    }

    if let Some(r) = &rating {
        params.push(r);
    }

    params.push(&limit_val);
    params.push(&offset_val);

    let rows = client.query(&sql, &params[..]).await?;

    let manga_list: Vec<Manga> = rows.iter().map(|row| Manga {
        id: row.get(0),
        title: row.get(1),
        alt_titles: row.get(2),
        cover_url: row.get(3),
        description: row.get(4),
        tags: row.get(5),
        rating: row.get(6),
        monitored: row.get(7),
        check_interval_secs: row.get(8),
        discover_interval_secs: row.get(9),
        last_chapter_check: row.get(10),
        last_discover_check: row.get(11),
    }).collect();

    Ok(manga_list)
}

/// Get manga by ID
pub async fn get_manga_by_id(pool: &Pool, manga_id: &str) -> Result<Option<Manga>, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let rows = client.query(
        "SELECT id, title, alt_titles, cover_url, description, tags, rating, monitored, check_interval_secs, discover_interval_secs, last_chapter_check, last_discover_check FROM manga WHERE id = $1",
        &[&manga_id]
    ).await?;

    if let Some(row) = rows.get(0) {
        Ok(Some(Manga {
            id: row.get(0),
            title: row.get(1),
            alt_titles: row.get(2),
            cover_url: row.get(3),
            description: row.get(4),
            tags: row.get(5),
            rating: row.get(6),
            monitored: row.get(7),
            check_interval_secs: row.get(8),
            discover_interval_secs: row.get(9),
            last_chapter_check: row.get(10),
            last_discover_check: row.get(11),
        }))
    } else {
        Ok(None)
    }
}

/// Get chapters by manga ID and source ID
pub async fn get_chapters_by_manga_source_data_id(
    pool: &Pool,
    manga_id: &str,
    source_id: i32,
) -> Result<Vec<Chapter>, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    // First get manga_source_data_id
    let msd_row = client.query_one(
        "SELECT id FROM manga_source_data WHERE manga_id = $1 AND source_id = $2",
        &[&manga_id, &source_id]
    ).await?;

    let manga_source_data_id: i32 = msd_row.get(0);

    // Then get chapters
    let rows = client.query(
        "SELECT id, manga_source_data_id, chapter_number, url, scraped FROM chapters WHERE manga_source_data_id = $1",
        &[&manga_source_data_id]
    ).await?;

    let chapters: Vec<Chapter> = rows.iter().map(|row| Chapter {
        id: row.get(0),
        manga_source_data_id: row.get(1),
        chapter_number: row.get(2),
        url: row.get(3),
        scraped: row.get(4),
    }).collect();

    Ok(chapters)
}

/// Insert manga (upsert)
pub async fn insert_manga(pool: &Pool, manga: &Manga) -> Result<(), PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    match client.execute(
        "INSERT INTO manga (id, title, alt_titles, cover_url, description, tags, rating)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT(id) DO UPDATE SET
            title = EXCLUDED.title,
            alt_titles = EXCLUDED.alt_titles,
            cover_url = EXCLUDED.cover_url,
            description = EXCLUDED.description,
            tags = EXCLUDED.tags,
            rating = EXCLUDED.rating",
        &[
            &manga.id,
            &manga.title,
            &manga.alt_titles.as_deref().unwrap_or(""),
            &manga.cover_url.as_deref().unwrap_or(""),
            &manga.description.as_deref().unwrap_or(""),
            &manga.tags.as_deref().unwrap_or(""),
            &manga.rating.as_deref().unwrap_or(""),
        ],
    ).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to insert manga: {}", e);
            Err(e)
        }
    }
}

/// Insert manga source data
pub async fn insert_manga_source_data(
    pool: &Pool,
    manga_source_data: &MangaSourceData,
) -> Result<i32, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let row = client.query_one(
        "INSERT INTO manga_source_data (manga_id, source_id, source_manga_id, source_manga_url)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (manga_id, source_id) DO UPDATE SET
            source_manga_id = EXCLUDED.source_manga_id,
            source_manga_url = EXCLUDED.source_manga_url
         RETURNING id",
        &[
            &manga_source_data.manga_id,
            &manga_source_data.source_id,
            &manga_source_data.source_manga_id,
            &manga_source_data.source_manga_url,
        ],
    ).await?;

    Ok(row.get(0))
}

/// Insert chapters in bulk
pub async fn insert_chapters(
    pool: &Pool,
    manga_source_data_id: i32,
    chapters: &[Chapter],
) -> Result<(), PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let stmt = client.prepare(
        "INSERT INTO chapters (manga_source_data_id, chapter_number, url)
         VALUES ($1, $2, $3)
         ON CONFLICT (manga_source_data_id, url) DO NOTHING"
    ).await?;

    for chapter in chapters {
        client.execute(
            &stmt,
            &[
                &manga_source_data_id,
                &chapter.chapter_number,
                &chapter.url,
            ],
        ).await?;
    }

    Ok(())
}

/// Set manga monitoring status
pub async fn set_manga_monitoring(
    pool: &Pool,
    manga_id: &str,
    monitored: bool,
    check_interval_secs: Option<i64>,
    discover_interval_secs: Option<i64>,
) -> Result<(), PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let monitored_val = if monitored { 1 } else { 0 };

    client.execute(
        "UPDATE manga SET monitored = $1,
         check_interval_secs = COALESCE($2, check_interval_secs),
         discover_interval_secs = COALESCE($3, discover_interval_secs)
         WHERE id = $4",
        &[&monitored_val, &check_interval_secs, &discover_interval_secs, &manga_id],
    ).await?;

    Ok(())
}

/// Mark chapter check timestamp
pub async fn mark_chapter_check(pool: &Pool, manga_id: &str, ts: i64) -> Result<(), PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    client.execute(
        "UPDATE manga SET last_chapter_check = $1 WHERE id = $2",
        &[&ts, &manga_id],
    ).await?;

    Ok(())
}

/// Get manga IDs due for chapter check
pub async fn due_for_chapter_check(pool: &Pool, now_ts: i64) -> Result<Vec<String>, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let rows = client.query(
        "SELECT id FROM manga WHERE monitored = 1 AND check_interval_secs IS NOT NULL AND (
            last_chapter_check IS NULL OR ($1 - last_chapter_check) >= check_interval_secs
        )",
        &[&now_ts],
    ).await?;

    let ids: Vec<String> = rows.iter().map(|row| row.get(0)).collect();
    Ok(ids)
}

/// Get manga source data by manga ID
pub async fn get_manga_source_data_by_manga_id(
    pool: &Pool,
    manga_id: &str,
) -> Result<Vec<MangaSourceData>, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let rows = client.query(
        "SELECT id, manga_id, source_id, source_manga_id, source_manga_url FROM manga_source_data WHERE manga_id = $1",
        &[&manga_id]
    ).await?;

    let manga_source_data: Vec<MangaSourceData> = rows.iter().map(|row| MangaSourceData {
        manga_id: row.get(1),
        source_id: row.get(2),
        source_manga_id: row.get(3),
        source_manga_url: row.get(4),
    }).collect();

    Ok(manga_source_data)
}

/// Get all manga from a specific source
pub async fn get_manga_by_source(pool: &Pool, source_id: i32) -> Result<Vec<Manga>, PgError> {
    let client = pool.get().await.expect("Failed to get connection from pool");

    let rows = client.query(
        "SELECT DISTINCT m.id, m.title, m.alt_titles, m.cover_url, m.description, m.tags, m.rating,
         m.monitored, m.check_interval_secs, m.discover_interval_secs, m.last_chapter_check, m.last_discover_check
         FROM manga m
         JOIN manga_source_data msd ON m.id = msd.manga_id
         WHERE msd.source_id = $1",
        &[&source_id]
    ).await?;

    let manga_list: Vec<Manga> = rows.iter().map(|row| Manga {
        id: row.get(0),
        title: row.get(1),
        alt_titles: row.get(2),
        cover_url: row.get(3),
        description: row.get(4),
        tags: row.get(5),
        rating: row.get(6),
        monitored: row.get(7),
        check_interval_secs: row.get(8),
        discover_interval_secs: row.get(9),
        last_chapter_check: row.get(10),
        last_discover_check: row.get(11),
    }).collect();

    Ok(manga_list)
}
