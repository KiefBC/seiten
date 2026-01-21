use app::*;
use axum::Router;
use chrono::Local;
use entity::{
    anidb_dump_meta,
    anidb_title,
    prelude::{
        AnidbDumpMeta,
        AnidbTitle,
        TitleType
    }
};
use flate2::read::GzDecoder;
use leptos::{
    logging::log,
    prelude::*
};
use leptos_axum::{
    generate_route_list,
    LeptosRoutes};
use sea_orm::{
    prelude::*,
    ConnectOptions,
    Database,
    Set
};
use std::io::Read;
use std::time::Duration;

const ANIDB_DUMP_URL: &str = "https://anidb.net/api/anime-titles.dat.gz";
const DUMP_NAME: &str = "anime-titles";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    log!("Starting Seiten server...");
    let db_url = match std::env::var("DATABASE_URL") {
        Ok(val) => val,
        Err(e) => {
            panic!("DATABASE_URL must be set in .env file or environment: {}", e);
        }
    };

    log!("Connecting to database: {}", db_url);
    let mut db_opt = ConnectOptions::new(&db_url);
    db_opt.max_connections(10)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(300))
        .sqlx_logging(true);
    let db = match Database::connect(db_opt).await {
        Ok(conn) => conn,
        Err(e) => {
            panic!("Failed to connect to database: {}", e);
        }
    };
    log!("Database connected successfully");

    log!("Starting schema sync...");
    match db.get_schema_registry("entity::*").sync(&db).await {
        Ok(_) => log!("Schema sync successful"),
        Err(e) => {
            panic!("Schema sync failed: {}", e);
        }
    }
    log!("Schema sync completed");

    // Check if we need to refresh the AniDB titles dump
    match sync_anidb_titles(&db).await {
        Ok(_) => log!("AniDB titles are up to date"),
        Err(e) => {
            panic!("Failed to sync AniDB titles: {}", e);
        }
    }

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let app_state = AppState::new(leptos_options.clone(), db.clone());
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || provide_context(app_state.clone())
            },
            {
                let app_state = app_state.clone();
                move || shell(app_state.leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(|opts| {
            shell(opts)
        }))
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Check if we should fetch a fresh AniDB dump based on the last fetch time.
/// Returns true if we should fetch, false if we should skip.
fn should_fetch_dump(meta: Option<&anidb_dump_meta::Model>) -> bool {
    match meta {
        Some(m) => {
            if let Some(last_fetched) = m.last_fetched {
                let now = Local::now().naive_local();
                let duration_since = now - last_fetched;
                if duration_since.num_hours() >= 24 {
                    true
                } else {
                    false
                }
            } else {
                true
            }
        }
        None => true,
    }
}

/// Sync AniDB anime titles from the daily dump.
/// Downloads and parses the gzipped dump file if more than 24 hours have passed.
/// This will be called at server startup.
/// We are using Box<dyn Error> for simplicity in error handling since the function can fail in multiple ways.
///
/// TODO: Create a proper error enum for better error handling.
async fn sync_anidb_titles(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    log!("Checking AniDB titles dump...");
    let existing_meta = AnidbDumpMeta::find()
        .filter(anidb_dump_meta::Column::DumpName.eq(DUMP_NAME))
        .one(db)
        .await?;

    if !should_fetch_dump(existing_meta.as_ref()) {
        if let Some(ref meta) = existing_meta {
            log!(
                "AniDB dump is fresh (last fetched: {:?}), skipping download",
                meta.last_fetched
            );
        }
        return Ok(());
    }

    log!("Fetching AniDB titles dump from {}", ANIDB_DUMP_URL);
    // Using AppState won't work here because this function is called at startup
    let client = reqwest::Client::builder()
        .user_agent("Seiten/1.0")
        .build()?;

    let response = client.get(ANIDB_DUMP_URL).send().await?;
    let status = response.status();
    match status.is_success() {
        true => (),
        false => return Err(format!("Failed to download dump: HTTP {}", status).into()),
    }
    let compressed_bytes = response.bytes().await?;
    log!("Downloaded {} bytes", compressed_bytes.len());

    log!("Decompressing AniDB dump...");
    let mut decoder = GzDecoder::new(&compressed_bytes[..]);
    let mut content = String::new();
    decoder.read_to_string(&mut content)?;
    log!("Decompressed dump size: {} bytes", content.len());

    log!("Parsing dump...\nThis may take a few minutes...");
    let mut entry_count = 0;
    let mut dump_created: Option<String> = None;
    AnidbTitle::delete_many().exec(db).await?; // Clear existing titles, we'll re-import

    for line in content.lines() {
        // Skip comments, but capture the "created" timestamp
        if line.starts_with('#') {
            if line.starts_with("# created:") {
                dump_created = Some(line.trim_start_matches("# created:").trim().to_string());
            }
            continue;
        }

        // Parse: <aid>|<type>|<language>|<title>
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() != 4 {
            continue;
        }

        let anime_id: i32 = match parts[0].parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        let title_type: i32 = match parts[1].parse() {
            Ok(t) => t,
            Err(_) => continue,
        };

        let language = parts[2].to_string();
        let title = parts[3].to_string();

        let title_type_enum = match title_type {
            1 => TitleType::Primary,
            2 => TitleType::Synonym,
            3 => TitleType::Short,
            4 => TitleType::Official,
            _ => continue,
        };

        let new_title = anidb_title::ActiveModel {
            id: Set(Uuid::new_v4()),
            anime_id: Set(anime_id),
            title_type: Set(title_type_enum),
            language: Set(language),
            title: Set(title),
        };

        AnidbTitle::insert(new_title).exec(db).await?;
        entry_count += 1;
    }
    log!("Dump parsing complete. Imported {} titles from AniDB dump", entry_count);

    let now = Local::now().naive_local();
    match existing_meta {
        Some(existing) => {
            log!(
            "Updating AniDB dump meta: last_fetched={:?}, dump_created={:?}, entry_count={}",
            &now,
            &dump_created,
            entry_count
        );
            let mut updated: anidb_dump_meta::ActiveModel = existing.into();
            updated.last_fetched = Set(Some(now));
            updated.dump_created = Set(dump_created.clone());
            updated.entry_count = Set(Some(entry_count));
            updated.update(db).await?;
            log!("AniDB dump meta updated successfully");
        }
        None => {
            log!(
            "Inserting new AniDB dump meta: last_fetched={:?}, dump_created={:?}, entry_count={}",
            &now,
            &dump_created,
            entry_count
        );
            let new_meta = anidb_dump_meta::ActiveModel {
                id: Set(Uuid::new_v4()),
                dump_name: Set(DUMP_NAME.to_string()),
                last_fetched: Set(Some(now)),
                dump_created: Set(dump_created),
                entry_count: Set(Some(entry_count)),
            };
            AnidbDumpMeta::insert(new_meta).exec(db).await?;
            log!("Inserted new AniDB dump meta successfully");
        }
    }
    log!("AniDB titles sync completed successfully");

    Ok(())
}
