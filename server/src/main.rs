use axum::Router;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use app::*;
use leptos::logging::log;
use sea_orm::{Database, EntityTrait, Set, ActiveModelTrait};
use sea_orm::entity::prelude::Uuid;
use entity::prelude::*;
use entity::{series, episode};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    log!("Connecting to database: {}", db_url);
    let db = &Database::connect(&db_url)
        .await
        .expect("Failed to connect to database");
    log!("Database connected successfully");

    log!("Starting schema sync...");
    // db.get_schema_builder()
    //     .register(User)
    //     .register(Series)
    //     .register(Episode)
    //     .apply(db)
    //     .await
    //     .expect("Failed to apply schema");

    db.get_schema_registry("entity::*").sync(db)
        .await
        .expect("Failed to sync schema");
    log!("Schema sync completed");

    log!("Creating dummy data...");

    let series_id = Uuid::new_v4();
    let one_piece = series::ActiveModel {
        id: Set(series_id),
        slug: Set("one-piece".to_string()),
        title: Set("One Piece".to_string()),
        last_fetched: Set(None),
        ..Default::default()
    };

    if Series::find_by_id(series_id).one(db).await.unwrap().is_none() {
        one_piece.insert(db).await.unwrap();
        log!("Created series: One Piece");

        let episodes_data = [
            ("Romance Dawn", 1, episode::EpisodeType::Canon),
            ("Enter the Great Swordsman", 2, episode::EpisodeType::Canon),
            ("Morgan vs. Luffy", 3, episode::EpisodeType::MixedCanon),
        ];

        for (title, num, ep_type) in episodes_data {
            let ep = episode::ActiveModel {
                id: Set(Uuid::new_v4()),
                show_id: Set(series_id),
                episode_num: Set(num),
                episode_type: Set(ep_type),
                title: Set(Some(title.to_string())),
                ..Default::default()
            };
            ep.insert(db).await.unwrap();
            log!("Created episode {}: {}", num, title);
        }
    } else {
        log!("Dummy data already exists, skipping...");
    }

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
