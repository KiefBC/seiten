use axum::Router;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use app::*;
use leptos::logging::log;
use sea_orm::Database;
use entity::prelude::*;

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

    // You can now use your entities like this:
    // use entity::prelude::*;
    // use sea_orm::EntityTrait;
    // let users = User::find().all(&db).await.unwrap();

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
