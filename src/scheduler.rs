use crate::{app_state::AppState, db, models::Source};
use actix_web::web;
use chrono::Utc;

pub fn spawn(data: web::Data<AppState>) {
    let data_clone = data.clone();
    actix_web::rt::spawn(async move {
        loop {
            // sleep between cycles
            actix_web::rt::time::sleep(std::time::Duration::from_secs(60)).await;
            let now_ts = Utc::now().timestamp();

            // Fetch due manga and process
            if let Ok(conn) = data_clone.db.lock() {
                match db::due_for_chapter_check(&conn, now_ts) {
                    Ok(ids) => {
                        drop(conn);
                        for id in ids {
                            // For each manga, fetch sources and refresh chapters
                            if let Ok(conn2) = data_clone.db.lock() {
                                if let Ok(msd_list) =
                                    db::get_manga_source_data_by_manga_id(&conn2, &id)
                                {
                                    drop(conn2);
                                    for msd in msd_list {
                                        let chapters = match msd.source_id {
                                            x if x == Source::MangaDex as i32 => {
                                                crate::sources::mangadex::get_chapters(
                                                    &data_clone.client,
                                                    &msd.source_manga_id,
                                                )
                                                .await
                                                .unwrap_or_default()
                                            }
                                            x if x == Source::FireScans as i32 => {
                                                crate::sources::firescans::get_chapters(
                                                    &data_clone.client,
                                                    &msd.source_manga_url,
                                                )
                                                .await
                                                .unwrap_or_default()
                                            }
                                            x if x == Source::RizzComic as i32 => {
                                                crate::sources::rizzcomic::get_chapters(
                                                    &data_clone.client,
                                                    &msd.source_manga_url,
                                                )
                                                .await
                                                .unwrap_or_default()
                                            }
                                            x if x == Source::DrakeComic as i32 => {
                                                crate::sources::drakecomic::get_chapters(
                                                    &data_clone.client,
                                                    &msd.source_manga_url,
                                                )
                                                .await
                                                .unwrap_or_default()
                                            }
                                            x if x == Source::Asmotoon as i32 => {
                                                crate::sources::asmotoon::get_chapters(
                                                    &data_clone.client,
                                                    &msd.source_manga_url,
                                                )
                                                .await
                                                .unwrap_or_default()
                                            }
                                            x if x == Source::ResetScans as i32 => {
                                                crate::sources::reset_scans::get_chapters(
                                                    &data_clone.client,
                                                    &msd.source_manga_url,
                                                )
                                                .await
                                                .unwrap_or_default()
                                            }
                                            x if x == Source::Kagane as i32 => {
                                                crate::sources::kagane::get_chapters(
                                                    &data_clone.client,
                                                    &msd.source_manga_url,
                                                )
                                                .await
                                                .unwrap_or_default()
                                            }
                                            _ => Vec::new(),
                                        };
                                        if let Ok(mut conn3) = data_clone.db.lock() {
                                            if let Ok(msd_id) = conn3.prepare("SELECT id FROM manga_source_data WHERE manga_id = ?1 AND source_id = ?2").and_then(|mut s| s.query_row(rusqlite::params![&id, msd.source_id], |row| row.get::<_, i64>(0))) {
                                                if let Ok(tx) = conn3.transaction() {
                                                    let _ = db::insert_chapters(&tx, msd_id, &chapters);
                                                    let _ = tx.commit();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if let Ok(conn4) = data_clone.db.lock() {
                                let _ = db::mark_chapter_check(&conn4, &id, now_ts);
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }
    });
}
