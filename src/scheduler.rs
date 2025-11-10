use crate::{app_state::AppState, pg_db, models::Source};
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
            match pg_db::due_for_chapter_check(&data_clone.pool, now_ts).await {
                Ok(ids) => {
                    for id in ids {
                        // For each manga, fetch sources and refresh chapters
                        if let Ok(msd_list) =
                            pg_db::get_manga_source_data_by_manga_id(&data_clone.pool, &id).await
                        {
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

                                // Get manga_source_data_id and insert chapters
                                if let Ok(client) = data_clone.pool.get().await {
                                    if let Ok(row) = client.query_one(
                                        "SELECT id FROM manga_source_data WHERE manga_id = $1 AND source_id = $2",
                                        &[&id, &msd.source_id]
                                    ).await {
                                        let msd_id: i32 = row.get(0);
                                        let _ = pg_db::insert_chapters(&data_clone.pool, msd_id, &chapters).await;
                                    }
                                }
                            }
                        }
                        let _ = pg_db::mark_chapter_check(&data_clone.pool, &id, now_ts).await;
                    }
                }
                Err(_) => {}
            }
        }
    });
}
