use leptos::{prelude::*, reactive::spawn_local};
use leptos::prelude::set_timeout;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

mod types;
pub use types::*;

pub mod api;

#[cfg(feature = "ssr")]
pub mod store;
#[cfg(feature = "ssr")]
use store::{AniDBStore, EpisodeStore, SeriesStore};

#[cfg(feature = "ssr")]
use axum::extract::FromRef;
use leptos::logging::log;
#[cfg(feature = "ssr")]
use reqwest::Client;
#[cfg(feature = "ssr")]
use sea_orm::DatabaseConnection;

/// Application state that holds shared resources
/// This is available in server functions via `expect_context::<AppState>()`
#[cfg(feature = "ssr")]
#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    /// This is a db pool conn not a single conn
    pub db: DatabaseConnection,
    pub http: Client,
    pub series_store: SeriesStore,
    pub episode_store: EpisodeStore,
    pub anidb_store: AniDBStore,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub fn new(leptos_options: LeptosOptions, db: DatabaseConnection) -> Self {
        let http = Client::builder()
            .user_agent("Seiten/1.0")
            .build()
            .expect("Failed to build HTTP client");

        let series_store = SeriesStore::new(db.clone());
        let episode_store = EpisodeStore::new(db.clone());
        let anidb_store = AniDBStore::new(db.clone());

        Self {
            leptos_options,
            db,
            http,
            series_store,
            episode_store,
            anidb_store,
        }
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="mytheme">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/seiten.css"/>

        // sets the document title
        <Title text="Seiten - Anime Canon Manager"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Scrapes anime series data from the given URL.
/// Specifically designed to work with animefillerlist.com series pages.
#[server]
pub async fn scrape_anime_series(url: String) -> Result<SeriesData, ServerFnError> {
    use crate::api::scraping::orchestrate_scrape;
    orchestrate_scrape(&url).await
}

/// Scrapes AniDB series data given an AniDB anime ID.
#[server]
pub async fn scrape_anidb_series(anidb_id: String) -> Result<AniDBSeriesData, ServerFnError> {
    use crate::api::scraping::orchestrate_anidb_scrape;
    orchestrate_anidb_scrape(&anidb_id).await
}

/// Enriches a series and its episodes with metadata from AniDB.
#[server]
pub async fn enrich_with_anidb(series_id: String) -> Result<(u64, u64), ServerFnError> {
    use crate::api::scraping::enrich_series_with_anidb;
    use uuid::Uuid;

    let uuid = match Uuid::parse_str(&series_id) {
        Ok(u) => u,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!("Invalid UUID: {}", e)));
        }
    };

    enrich_series_with_anidb(uuid).await
}

/// Toast notification data
#[derive(Clone)]
struct Toast {
    message: String,
    is_error: bool,
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let input_value =
        RwSignal::new("https://www.animefillerlist.com/shows/naruto-shippuden".to_string());
    let scraped_data = RwSignal::new(Option::<SeriesData>::None);
    let toast = RwSignal::new(Option::<Toast>::None);

    // Only used for our hydration test
    let count = RwSignal::new(0);

    let show_toast = move |message: String, is_error: bool| {
        toast.set(Some(Toast { message, is_error }));

        set_timeout(
            move || toast.set(None),
            std::time::Duration::from_secs(3),
        );
    };

    let on_scrape = move |_| {
        log!("Scrape clicked with value: {}", input_value.get());

        let url = input_value.get();
        spawn_local(async move {
            log!("Scraping: {}", url);

            match scrape_anime_series(url).await {
                Ok(data) => {
                    log!(
                        "Scraped successfully: {} Episodes!",
                        data.episodes.len()
                    );
                    scraped_data.set(Some(data));
                    show_toast("Scrape successful!".to_string(), false);
                }
                Err(e) => {
                    log!("Scrape failed: {:?}", e);
                    show_toast(format!("Scrape failed: {:?}", e), true);
                }
            }
        });
    };

    let on_anidb_scrape = move |_| {
        log!("Scrape clicked with value: {}", input_value.get());

        let aid = "239".to_string();
        spawn_local(async move {
            log!("Scraping: {}", aid);

            match scrape_anidb_series(aid).await {
                Ok(data) => {
                    log!(
                        "Scraped successfully: {} Episodes!",
                        data.episodes.len()
                    );
                    // scraped_data.set(Some(data));
                }
                Err(e) => {
                    log!("Scrape failed: {:?}", e)
                }
            }
        });
    };

    let on_count_click = move |_| *count.write() += 1;

    view! {
            <div class="min-h-screen flex items-center justify-center p-4">
                <div class="w-full max-w-2xl space-y-4">
                    <div class="card bg-base-100 shadow-xl">
                        <div class="card-body">
                            <h1 class="card-title text-5xl font-bold justify-center mb-8">"(正典) Seiten"</h1>

                            <div class="form-control w-full">
                                <label class="label">
                                    <span class="label-text">"Anime Series URL"</span>
                                </label>
                                <input
                                    type="text"
                                    placeholder="https://www.animefillerlist.com/shows/one-piece"
                                    class="input input-bordered input-primary w-full"
                                    on:input=move |ev| {
                                        input_value.set(event_target_value(&ev));
                                    }
                                    prop:value=move || input_value.get()
                                />
                            </div>

                            <div class="card-actions justify-end mt-6 gap-3">
                                <button class="btn btn-primary" on:click=on_scrape>
                                    "Scrape"
                                </button>
                            <button class="btn btn-info" on:click=on_anidb_scrape>
                                    "AniDB Scrape"
                                </button>
                            </div>
                        </div>
                    </div>

                    <div class="card bg-base-100 shadow-xl">
                        <div class="card-body">
                            <h2 class="card-title text-sm opacity-70">"Output"</h2>

                            <div role="tablist" class="tabs tabs-bordered">
                                <input type="radio" name="output_tabs" role="tab" class="tab" aria-label="JSON" checked=true/>
                                <div role="tabpanel" class="tab-content p-4 overflow-hidden">
                                    <pre class="bg-base-200 p-4 rounded-lg overflow-x-auto text-sm">
    {r#"{
  "series": {
    "title": "One Piece",
    "slug": "one-piece",
    "episodes": [
      {
        "number": 1,
        "type": "Canon",
        "title": "I'm Luffy! The Man Who's Gonna Be King of the Pirates!"
      },
      {
        "number": 2,
        "type": "Canon",
        "title": "Enter the Great Swordsman!"
      },
      {
        "number": 131,
        "type": "Filler",
        "title": "The First Patient! The Untold Story of the Rumble Ball!"
      }
    ]
  }
}"#}
                                    </pre>
                                </div>

                                <input type="radio" name="output_tabs" role="tab" class="tab" aria-label="RON"/>
                                <div role="tabpanel" class="tab-content p-4 overflow-hidden">
                                    <pre class="bg-base-200 p-4 rounded-lg overflow-x-auto text-sm">
    {r#"Series(
  title: "One Piece",
  slug: "one-piece",
  episodes: [
    Episode(
      number: 1,
      episode_type: Canon,
      title: Some("I'm Luffy! The Man Who's Gonna Be King of the Pirates!"),
    ),
    Episode(
      number: 2,
      episode_type: Canon,
      title: Some("Enter the Great Swordsman!"),
    ),
    Episode(
      number: 131,
      episode_type: Filler,
      title: Some("The First Patient! The Untold Story of the Rumble Ball!"),
    ),
  ],
)"#}
                                    </pre>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="card bg-base-200 shadow-xl">
                        <div class="card-body">
                            <h2 class="card-title text-sm opacity-70">"Hydration Test"</h2>
                            <button class="btn btn-secondary" on:click=on_count_click>
                                "Click Me: " {count}
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            {move || toast.get().map(|t| view! {
                <div class="toast toast-top toast-end">
                    <div class={if t.is_error { "alert alert-error" } else { "alert alert-success" }}>
                        <span>{t.message}</span>
                    </div>
                </div>
            })}
        }
}
