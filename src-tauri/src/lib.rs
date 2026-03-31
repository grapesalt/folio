use tauri::{Emitter, Manager};

pub mod db;
pub mod errors;
pub mod index;
pub mod media;
pub mod search;
pub mod subtitles;
pub mod transcribe;

pub type FolioResult<T> = Result<T, errors::FolioError>;

#[inline]
pub fn get_folio_dir() -> FolioResult<std::path::PathBuf> {
    let dir = dirs::data_dir()
        .ok_or_else(|| {
            errors::FolioError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Data directory not found",
            ))
        })?
        .join("folio");

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

#[tauri::command]
async fn index(app: tauri::AppHandle) -> FolioResult<()> {
    let db = db::Database::open(&get_folio_dir()?.join("folio.db"))?;
    run_index(app, db).await
}

async fn run_index(app: tauri::AppHandle, db: db::Database) -> FolioResult<()> {
    app.emit("init:start", {}).ok();

    let dir = get_folio_dir()?;
    let settings = db.get_settings()?;
    let index_path = dir.join("index");

    let mut search_index = if index_path.exists() {
        search::SearchIndex::open(&index_path)?
    } else {
        search::SearchIndex::create(&index_path)?
    };

    let dirs: Vec<std::path::PathBuf> = settings
        .dirs
        .iter()
        .map(|d| std::path::PathBuf::from(d))
        .collect();
    let exts: Vec<&str> = settings.exts.iter().map(|s| s.as_str()).collect();

    let paths = index::get_files_par(&dirs, &exts)?;

    for (i, file) in paths.iter().enumerate() {
        if !db.is_upto_date(&file.media.to_string_lossy())? {
            app.emit(
                "init:progress",
                serde_json::json!({
                    "current": i + 1,
                    "total": paths.len(),
                    "file": file.media.to_string_lossy(),
                }),
            )
            .ok();

            let mut has_subtitles = file.subtitles.is_some();

            db.insert_file(&crate::db::IndexedFile {
                path: file.media.to_string_lossy().to_string(),
                modified_at: file
                    .media
                    .metadata()?
                    .modified()?
                    .elapsed()
                    .unwrap_or_default()
                    .as_millis() as i64,
                file_size: file.media.metadata()?.len() as i64,
                has_subtitles,
                transcription_model: None,
            })?;

            if has_subtitles {
                if let Some(srt) =
                    crate::media::extract_subtitles(&file.media).ok()
                {
                    has_subtitles = true;
                    db.store_segments(&file.media.to_string_lossy(), &srt)?;
                }
            } else {
                let Some(srt_file) = &file.subtitles else {
                    has_subtitles = false;
                    continue;
                };

                let srt = crate::subtitles::parse_srt_file(&srt_file)?;
                db.store_segments(&file.media.to_string_lossy(), &srt)?;
            }

            db.insert_file(&crate::db::IndexedFile {
                path: file.media.to_string_lossy().to_string(),
                modified_at: file
                    .media
                    .metadata()?
                    .modified()?
                    .elapsed()
                    .unwrap_or_default()
                    .as_millis() as i64,
                file_size: file.media.metadata()?.len() as i64,
                has_subtitles,
                transcription_model: None,
            })?;

            search_index.add_media_file(&file).ok();
        }
    }

    search_index.commit()?;
    app.emit("init:done", {}).ok();

    Ok(())
}

#[tauri::command]
fn get_settings() -> FolioResult<crate::db::Settings> {
    let db = db::Database::open(&get_folio_dir()?.join("folio.db"))?;
    db.get_settings()
}

#[tauri::command]
fn search(query: &str) -> FolioResult<Box<[search::SearchResult]>> {
    let index_path = get_folio_dir()?.join("index");
    let search_index = search::SearchIndex::open(&index_path)?;

    search_index.search(query, 50)
}

#[tauri::command]
async fn get_thumbnail(res: search::SearchResult) -> FolioResult<String> {
    media::generate_thumbnail(
        &res.file,
        (res.segment.start + res.segment.end) / 2000,
        6,
    )
}

#[tauri::command]
async fn store_settings(
    app: tauri::AppHandle,
    settings: crate::db::Settings,
) -> FolioResult<()> {
    let db = db::Database::open(&get_folio_dir()?.join("folio.db"))?;
    db.store_settings(&settings)?;

    run_index(app, db).await
}

#[tauri::command]
fn end() -> FolioResult<()> {
    let db = db::Database::open(&get_folio_dir()?.join("folio.db"))?;
    let paths = db.all_paths()?;
    let missing: Box<[_]> = paths
        .iter()
        .filter(|p| !std::path::Path::new(p).exists())
        .collect();

    let mut search_index =
        search::SearchIndex::open(&get_folio_dir()?.join("index"))?;

    for path in missing {
        db.remove_file(path)?;
        search_index.remove_media_file(std::path::Path::new(path));
    }

    search_index.commit()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .on_page_load(|window, _payload| {
            let handle = window.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = index(handle).await;
            });
        })
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            index,
            get_settings,
            search,
            store_settings,
            get_thumbnail,
            end
        ])
        .run(tauri::generate_context!())
        .expect("error while running folio.");
}
