use app::*;
use axum::Router;
// use entity::prelude::*;
// use entity::{episode, series};
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
// use sea_orm::entity::prelude::Uuid;
// use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
use sea_orm::Database;
mod plex;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    log!("Connecting to database: {}", db_url);
    let db = &Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    log!("Database connected successfully");

    log!("Starting schema sync...");
    db.get_schema_registry("entity::*")
        .sync(db)
        .await
        .expect("Failed to sync schema");
    log!("Schema sync completed");

    // log!("Creating dummy data...");
    // // Check if One Piece already exists by slug
    // let existing_series = Series::find()
    //     .filter(series::Column::Slug.eq("one-piece"))
    //     .one(db)
    //     .await
    //     .unwrap();
    //
    // let _series_id = if let Some(series) = existing_series {
    //     log!("Series 'One Piece' already exists, skipping...");
    //     series.id
    // } else {
    //     // Create new series
    //     let series_id = Uuid::new_v4();
    //     let one_piece = series::ActiveModel {
    //         id: Set(series_id),
    //         slug: Set("one-piece".to_string()),
    //         title: Set("One Piece".to_string()),
    //         last_fetched: Set(None),
    //         ..Default::default()
    //     };
    //     one_piece.insert(db).await.unwrap();
    //     log!("Created series: One Piece");
    //
    //     // Create 3 episodes
    //     let episodes_data = [
    //         ("Romance Dawn", 1, episode::EpisodeType::Canon),
    //         ("Enter the Great Swordsman", 2, episode::EpisodeType::Canon),
    //         ("Morgan vs. Luffy", 3, episode::EpisodeType::MixedCanon),
    //     ];
    //
    //     for (title, num, ep_type) in episodes_data {
    //         let ep = episode::ActiveModel {
    //             id: Set(Uuid::new_v4()),
    //             show_id: Set(series_id),
    //             episode_num: Set(num),
    //             episode_type: Set(ep_type),
    //             title: Set(Some(title.to_string())),
    //             ..Default::default()
    //         };
    //         ep.insert(db).await.unwrap();
    //         log!("Created episode {}: {}", num, title);
    //     }
    //
    //     series_id
    // };

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Create AppState with leptos_options, db, and http client
    let app_state = AppState::new(leptos_options.clone(), db.clone());

    // Generate the list of routes in your Leptos App
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
