extern crate log;
use crate::models::{Chapter, Manga, MangaSourceData};
use log::error;
use rusqlite::params;
use rusqlite::{Connection, Result, Transaction};

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("manga.db")?;
    Ok(conn)
}

pub fn seed_sources(conn: &Connection) -> Result<()> {
    log::info!("Seeding sources table...");
    // Existing
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (1, 'MangaDex', 'https://mangadex.org')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (2, 'FireScans', 'https://firescans.xyz')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (3, 'RizzComic', 'https://rizzcomic.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (4, 'MyAnimeList', 'https://myanimelist.net')", [])?;
    conn.execute(
        "INSERT OR IGNORE INTO sources (id, name, url) VALUES (5, 'AniList', 'https://anilist.co')",
        [],
    )?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (6, 'DrakeComic', 'https://drakecomic.org')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (7, 'KDTNovels', 'https://kdtnovels.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (8, 'Asmotoon', 'https://asmotoon.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (9, 'ResetScans', 'https://reset-scans.org')", [])?;
    conn.execute(
        "INSERT OR IGNORE INTO sources (id, name, url) VALUES (10, 'Kagane', 'https://kagane.org')",
        [],
    )?;

    // New seeds requested
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (11, 'Asura Scans', 'https://asurascans.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (12, 'BookLive', 'https://booklive.jp')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (13, 'Comikey', 'https://comikey.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (14, 'DENPA BOOKS', 'https://denpa.pub')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (15, 'Dark Horse Comics', 'https://www.darkhorse.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (16, 'Day Comics', 'https://daycomics.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (17, 'FAKKU', 'https://www.fakku.net')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (18, 'Flame Comics', 'https://flamecomics.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (19, 'Grim Scans', 'https://grimscans.team')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (20, 'Hive Toons', 'https://hivetoons.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (21, 'INKR Comics', 'https://inkr.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (22, 'Irodori Comics', 'https://irodoricomics.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (23, 'J-Novel Club', 'https://j-novel.club')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (24, 'Kana', 'https://www.kana.fr')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (25, 'Kenscans', 'https://kenscans.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (26, 'Kodansha Comics', 'https://kodansha.us')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (27, 'Kodoku Studio', 'https://kodokustudio.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (28, 'Lezhin', 'https://www.lezhinus.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (29, 'Luna Toons', 'https://lunatoons.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (30, 'Madarascans', 'https://madarascans.com')", [])?;
    // 1 is MangaDex already
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (31, 'Manhuaus', 'https://manhuaus.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (32, 'Manta', 'https://manta.net')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (33, 'MediBang', 'https://medibang.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (34, 'Nyx Scans', 'https://nyxscans.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (35, 'One Peace Books', 'https://onepeacebooks.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (36, 'Others', 'https://example.com/others')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (37, 'Pocket Comics', 'https://www.pocketcomics.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (38, 'Qi Scans', 'https://qiscans.com')", [])?;
    // 9 is Reset Scans already
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (39, 'Rizz Fables', 'https://rizzfables.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (40, 'Rokari Comics', 'https://rokaricomics.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (41, 'Seven Seas Entertainment', 'https://sevenseasentertainment.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (42, 'Shusuisha', 'https://www.shueisha.co.jp')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (43, 'Siren Scans', 'https://sirenscans.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (44, 'Square Enix Manga', 'https://square-enix-books.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (45, 'StoneScape', 'https://stonescape.xyz')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (46, 'TOKYOPOP', 'https://tokyopop.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (47, 'Tapas', 'https://tapas.io')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (48, 'Tappytoon', 'https://www.tappytoon.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (49, 'Temple Scan', 'https://templescan.net')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (50, 'Thunderscans', 'https://thunderscans.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (51, 'Titan Manga', 'https://titan-comics.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (52, 'Toomics', 'https://toomics.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (53, 'UDON Entertainment', 'https://udonentertainment.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (54, 'VAST Visual', 'https://vastvisual.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (55, 'VIZ Media', 'https://www.viz.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (56, 'Vortex Scans', 'https://vortexscans.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (57, 'Webcomics', 'https://www.webcomicsapp.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (58, 'Webtoon', 'https://www.webtoons.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (59, 'Witch Scans', 'https://witchscans.com')", [])?;
    conn.execute("INSERT OR IGNORE INTO sources (id, name, url) VALUES (60, 'Yen Press', 'https://yenpress.com')", [])?;

    log::info!("Sources seeded successfully.");
    Ok(())
}

pub fn create_tables(conn: &Connection) -> Result<()> {
    log::info!("Creating tables if not exists...");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sources (\n            id INTEGER PRIMARY KEY AUTOINCREMENT,\n            name TEXT NOT NULL UNIQUE,\n            url TEXT NOT NULL\n        );",
        [],
    )?;

conn.execute(
        "CREATE TABLE IF NOT EXISTS manga (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            alt_titles TEXT,
            cover_url TEXT,
            description TEXT,
            tags TEXT,
            rating TEXT,
            monitored INTEGER,
            check_interval_secs INTEGER,
            discover_interval_secs INTEGER,
            last_chapter_check INTEGER,
            last_discover_check INTEGER,
            mal_id INTEGER,
            anilist_id INTEGER,
            mangabaka_id TEXT
        );",
        [],
    )?;

    // Migrations for existing DBs: add missing columns
    ensure_column(conn, "manga", "rating", "TEXT")?;
    ensure_column(conn, "manga", "monitored", "INTEGER")?;
    ensure_column(conn, "manga", "check_interval_secs", "INTEGER")?;
    ensure_column(conn, "manga", "discover_interval_secs", "INTEGER")?;
    ensure_column(conn, "manga", "last_chapter_check", "INTEGER")?;
    ensure_column(conn, "manga", "last_discover_check", "INTEGER")?;
    ensure_column(conn, "manga", "mangabaka_id", "TEXT")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS manga_source_data (\n            id INTEGER PRIMARY KEY AUTOINCREMENT,\n            manga_id TEXT NOT NULL,\n            source_id INTEGER NOT NULL,\n            source_manga_id TEXT NOT NULL,\n            source_manga_url TEXT NOT NULL,\n            FOREIGN KEY (manga_id) REFERENCES manga (id),\n            FOREIGN KEY (source_id) REFERENCES sources (id),\n            UNIQUE(manga_id, source_id)\n        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS chapters (\n            id INTEGER PRIMARY KEY AUTOINCREMENT,\n            manga_source_data_id INTEGER NOT NULL,\n            chapter_number TEXT NOT NULL,\n            url TEXT NOT NULL,\n            scraped BOOLEAN NOT NULL DEFAULT 0,\n            FOREIGN KEY (manga_source_data_id) REFERENCES manga_source_data (id),\n            UNIQUE(manga_source_data_id, url)\n        );",
        [],
    )?;

    // Helpful indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_msd_manga ON manga_source_data(manga_id);",
        [],
    )?;

    // Provider IDs mapping
    conn.execute(
        "CREATE TABLE IF NOT EXISTS provider_ids (\n            id INTEGER PRIMARY KEY AUTOINCREMENT,\n            manga_id TEXT NOT NULL,\n            provider TEXT NOT NULL,\n            provider_id TEXT NOT NULL,\n            UNIQUE(manga_id, provider),\n            FOREIGN KEY (manga_id) REFERENCES manga(id)\n        );",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_provider_manga ON provider_ids(manga_id);",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_ch_msd ON chapters(manga_source_data_id);",
        [],
    )?;

    log::info!("Tables ensured.");
    seed_sources(conn)?;
    Ok(())
}

fn ensure_column(conn: &Connection, table: &str, column: &str, column_type: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let mut exists = false;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(1)?; // 2nd column is name
        Ok(name)
    })?;
    for r in rows {
        if r? == column {
            exists = true;
            break;
        }
    }
    if !exists {
        let sql = format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            table, column, column_type
        );
        let _ = conn.execute(&sql, params![]);
    }
    Ok(())
}

pub fn get_manga_paginated(
    conn: &Connection,
    limit: Option<i32>,
    offset: Option<i32>,
    sort_by: &str,
    rating_filter: Option<&str>,
) -> Result<Vec<Manga>> {
    let sort_expr = match sort_by {
        "rating" => "CASE lower(coalesce(rating,'')) WHEN 'safe' THEN 1 WHEN 'suggestive' THEN 2 WHEN 'erotica' THEN 3 WHEN 'pornographic' THEN 4 ELSE 5 END, title",
        _ => "title",
    };

    let mut base = format!(
        "SELECT id, title, alt_titles, cover_url, description, tags, rating, monitored, check_interval_secs, discover_interval_secs, last_chapter_check, last_discover_check FROM manga"
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(r) = rating_filter {
        base.push_str(" WHERE lower(rating) = lower(?)");
        params.push(Box::new(r.to_string()));
    }
    base.push_str(&format!(" ORDER BY {} LIMIT ? OFFSET ?", sort_expr));

    params.push(Box::new(limit.unwrap_or(100)));
    params.push(Box::new(offset.unwrap_or(0)));
    let mut stmt = conn.prepare(&base)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(Manga {
            id: row.get(0)?,
            title: row.get(1)?,
            alt_titles: row.get(2)?,
            cover_url: row.get(3)?,
            description: row.get(4)?,
            tags: row.get(5)?,
            rating: row.get(6)?,
            monitored: row.get(7).ok(),
            check_interval_secs: row.get(8).ok(),
            discover_interval_secs: row.get(9).ok(),
            last_chapter_check: row.get(10).ok(),
            last_discover_check: row.get(11).ok(),
        })
    })?;

    let mut manga_list = Vec::new();
    for row in rows {
        manga_list.push(row?);
    }

    Ok(manga_list)
}

pub fn get_manga_count(conn: &Connection) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM manga")?;
    let count: i32 = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

pub fn get_chapter_count(conn: &Connection) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM chapters")?;
    let count: i32 = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

pub fn get_source_count(conn: &Connection) -> Result<i32> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM sources")?;
    let count: i32 = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

#[derive(Debug, serde::Serialize)]
pub struct PerSourceCounts {
    pub source_id: i32,
    pub source_name: String,
    pub manga: i32,
    pub chapters: i32,
}

pub fn get_per_source_counts(conn: &Connection) -> Result<Vec<PerSourceCounts>> {
    let sql = r#"
        SELECT s.id, s.name,
            (SELECT COUNT(DISTINCT msd.manga_id) FROM manga_source_data msd WHERE msd.source_id = s.id) AS manga_cnt,
            (SELECT COUNT(*) FROM chapters c JOIN manga_source_data msd ON c.manga_source_data_id = msd.id WHERE msd.source_id = s.id) AS ch_cnt
        FROM sources s ORDER BY s.id
    "#;
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(PerSourceCounts {
            source_id: row.get(0)?,
            source_name: row.get(1)?,
            manga: row.get(2)?,
            chapters: row.get(3)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn get_source_name(conn: &Connection, source_id: i32) -> Result<String> {
    let mut stmt = conn.prepare("SELECT name FROM sources WHERE id = ?1")?;
    let name: String = stmt.query_row([source_id], |row| row.get(0))?;
    Ok(name)
}

pub fn search_manga_paginated(
    conn: &Connection,
    query: &str,
    tags: Option<&str>,
    rating: Option<&str>,
    limit: Option<i32>,
    offset: Option<i32>,
    sort_by: &str,
) -> Result<Vec<Manga>> {
    let search_pattern = format!("%{}%", query);
    let sort_expr = match sort_by {
        "rating" => "CASE lower(coalesce(rating,'')) WHEN 'safe' THEN 1 WHEN 'suggestive' THEN 2 WHEN 'erotica' THEN 3 WHEN 'pornographic' THEN 4 ELSE 5 END, title",
        _ => "title",
    };

    let mut where_clauses: Vec<&str> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // title OR alt_titles
    where_clauses.push("(title LIKE ? OR alt_titles LIKE ?)");
    params.push(Box::new(search_pattern.clone()));
    params.push(Box::new(search_pattern.clone()));

    if let Some(tag_filter) = tags {
        where_clauses.push("tags LIKE ?");
        let tag_pattern = format!("%{}%", tag_filter);
        params.push(Box::new(tag_pattern));
    }
    if let Some(r) = rating {
        where_clauses.push("lower(rating) = lower(?)");
        params.push(Box::new(r.to_string()));
    }

    let where_clause = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };
    let query_str = format!(
        "SELECT id, title, alt_titles, cover_url, description, tags, rating, monitored, check_interval_secs, discover_interval_secs, last_chapter_check, last_discover_check FROM manga{} ORDER BY {} LIMIT ? OFFSET ?",
        where_clause, sort_expr
    );

    let limit_val = limit.unwrap_or(100);
    let offset_val = offset.unwrap_or(0);
    params.push(Box::new(limit_val));
    params.push(Box::new(offset_val));

    let mut stmt = conn.prepare(&query_str)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(Manga {
            id: row.get(0)?,
            title: row.get(1)?,
            alt_titles: row.get(2)?,
            cover_url: row.get(3)?,
            description: row.get(4)?,
            tags: row.get(5)?,
            rating: row.get(6)?,
            monitored: row.get(7).ok(),
            check_interval_secs: row.get(8).ok(),
            discover_interval_secs: row.get(9).ok(),
            last_chapter_check: row.get(10).ok(),
            last_discover_check: row.get(11).ok(),
        })
    })?;

    let mut manga_list = Vec::new();
    for row in rows {
        manga_list.push(row?);
    }

    Ok(manga_list)
}

pub fn get_manga_by_id(conn: &Connection, manga_id: &str) -> Result<Option<Manga>> {
    let mut stmt = conn.prepare("SELECT id, title, alt_titles, cover_url, description, tags, rating, monitored, check_interval_secs, discover_interval_secs, last_chapter_check, last_discover_check FROM manga WHERE id = ?1")?;
    let mut rows = stmt.query_map(&[manga_id], |row| {
        Ok(Manga {
            id: row.get(0)?,
            title: row.get(1)?,
            alt_titles: row.get(2)?,
            cover_url: row.get(3)?,
            description: row.get(4)?,
            tags: row.get(5)?,
            rating: row.get(6)?,
            monitored: row.get(7).ok(),
            check_interval_secs: row.get(8).ok(),
            discover_interval_secs: row.get(9).ok(),
            last_chapter_check: row.get(10).ok(),
            last_discover_check: row.get(11).ok(),
        })
    })?;

    if let Some(row) = rows.next() {
        Ok(Some(row?))
    } else {
        Ok(None)
    }
}

pub fn get_chapters_by_manga_source_data_id(
    conn: &Connection,
    manga_id: &str,
    source_id: i32,
) -> Result<Vec<Chapter>> {
    let mut stmt =
        conn.prepare("SELECT id FROM manga_source_data WHERE manga_id = ?1 AND source_id = ?2")?;
    let manga_source_data_id: i32 =
        stmt.query_row(&[manga_id, &source_id.to_string()], |row| row.get(0))?;

    let mut stmt = conn.prepare("SELECT id, manga_source_data_id, chapter_number, url, scraped FROM chapters WHERE manga_source_data_id = ?1")?;
    let rows = stmt.query_map(&[&manga_source_data_id], |row| {
        Ok(Chapter {
            id: row.get(0)?,
            manga_source_data_id: row.get(1)?,
            chapter_number: row.get(2)?,
            url: row.get(3)?,
            scraped: row.get(4)?,
        })
    })?;

    let mut chapters = Vec::new();
    for row in rows {
        chapters.push(row?);
    }

    Ok(chapters)
}

pub fn insert_manga(tx: &Transaction, manga: &Manga) -> Result<()> {
    match tx.execute(
        "INSERT INTO manga (id, title, alt_titles, cover_url, description, tags, rating) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            title=excluded.title,
            alt_titles=excluded.alt_titles,
            cover_url=excluded.cover_url,
            description=excluded.description,
            tags=excluded.tags,
            rating=excluded.rating",
        &[&manga.id, &manga.title, manga.alt_titles.as_deref().unwrap_or(""), manga.cover_url.as_deref().unwrap_or(""), manga.description.as_deref().unwrap_or(""), manga.tags.as_deref().unwrap_or(""), manga.rating.as_deref().unwrap_or("")],
    ) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to insert manga: {}", e);
            Err(e)
        }
    }
}

pub fn insert_manga_source_data(
    tx: &Transaction,
    manga_source_data: &MangaSourceData,
) -> Result<i64> {
    tx.execute(
        "INSERT OR IGNORE INTO manga_source_data (manga_id, source_id, source_manga_id, source_manga_url) VALUES (?1, ?2, ?3, ?4)",
        &[&manga_source_data.manga_id, &manga_source_data.source_id.to_string(), &manga_source_data.source_manga_id, &manga_source_data.source_manga_url],
    )?;
    Ok(tx.last_insert_rowid())
}

pub fn insert_chapters(
    tx: &Transaction,
    manga_source_data_id: i64,
    chapters: &[Chapter],
) -> Result<()> {
    let mut stmt = tx.prepare("INSERT OR IGNORE INTO chapters (manga_source_data_id, chapter_number, url) VALUES (?1, ?2, ?3)")?;
    for chapter in chapters {
        stmt.execute(&[
            &manga_source_data_id.to_string(),
            &chapter.chapter_number,
            &chapter.url,
        ])?;
    }
    Ok(())
}

pub fn set_manga_monitoring(
    conn: &Connection,
    manga_id: &str,
    monitored: bool,
    check_interval_secs: Option<i64>,
    discover_interval_secs: Option<i64>,
) -> Result<()> {
    conn.execute(
        "UPDATE manga SET monitored = ?1, check_interval_secs = COALESCE(?2, check_interval_secs), discover_interval_secs = COALESCE(?3, discover_interval_secs) WHERE id = ?4",
        params![if monitored {1} else {0}, check_interval_secs, discover_interval_secs, manga_id],
    )?;
    Ok(())
}

pub fn mark_chapter_check(conn: &Connection, manga_id: &str, ts: i64) -> Result<()> {
    conn.execute(
        "UPDATE manga SET last_chapter_check = ?1 WHERE id = ?2",
        params![ts, manga_id],
    )?;
    Ok(())
}

pub fn due_for_chapter_check(conn: &Connection, now_ts: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM manga WHERE monitored = 1 AND (check_interval_secs IS NOT NULL) AND (
            last_chapter_check IS NULL OR (?1 - last_chapter_check) >= check_interval_secs
        )",
    )?;
    let rows = stmt.query_map(params![now_ts], |row| row.get(0))?;
    let mut ids = Vec::new();
    for r in rows {
        ids.push(r?);
    }
    Ok(ids)
}

pub fn get_manga_source_data_by_manga_id(
    conn: &Connection,
    manga_id: &str,
) -> Result<Vec<MangaSourceData>> {
    let mut stmt = conn.prepare("SELECT id, manga_id, source_id, source_manga_id, source_manga_url FROM manga_source_data WHERE manga_id = ?1")?;
    let rows = stmt.query_map(&[manga_id], |row| {
        Ok(MangaSourceData {
            manga_id: row.get(1)?,
            source_id: row.get(2)?,
            source_manga_id: row.get(3)?,
            source_manga_url: row.get(4)?,
        })
    })?;

    let mut manga_source_data = Vec::new();
    for row in rows {
        manga_source_data.push(row?);
    }

    Ok(manga_source_data)
}
