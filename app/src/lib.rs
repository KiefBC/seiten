use leptos::{prelude::*, reactive::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

mod types;
pub use types::*;

#[cfg(feature = "ssr")]
use axum::extract::FromRef;
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
    pub db: DatabaseConnection,
    pub http: Client,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub fn new(leptos_options: LeptosOptions, db: DatabaseConnection) -> Self {
        let http = Client::builder()
            .user_agent("Seiten/1.0")
            .build()
            .expect("Failed to build HTTP client");

        Self {
            leptos_options,
            db,
            http,
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

#[server]
pub async fn scrape_anime_series(url: String) -> Result<SeriesData, ServerFnError> {
    use chrono::NaiveDate;

    let app_state = expect_context::<AppState>();

    leptos::logging::log!("Fetching URL: {}", url);

    let response = match app_state.http.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return Err(ServerFnError::ServerError(e.to_string()));
        }
    };

    if !response.status().is_success() {
        return Err(ServerFnError::ServerError(format!(
            "HTTP request returned status: {}",
            response.status()
        )));
    }

    let body = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            return Err(ServerFnError::ServerError(e.to_string()));
        }
    };

    leptos::logging::log!("Response received, body length: {} bytes", body.len());

    // For now, return a placeholder SeriesData with valid structure
    // TODO: Parse the HTML body and extract episode data
    Ok(SeriesData {
        title: "Test Series".to_string(),
        slug: "test-series".to_string(),
        episodes: vec![EpisodeData {
            ep_number: 1,
            jap_release: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            eng_release: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            episode_type: EpisodeType::Canon,
            eng_title: "Fetched from URL".to_string(),
            jap_title: None,
            duration: Some(24),
            manga_chapters: None,
        }],
    })
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let input_value = RwSignal::new(String::new());
    let scraped_data = RwSignal::new(Option::<SeriesData>::None);

    // only used for our hydration test
    let count = RwSignal::new(0);

    let on_scrape = move |_| {
        leptos::logging::log!("Scrape clicked with value: {}", input_value.get());

        let url = input_value.get();
        spawn_local(async move {
            leptos::logging::log!("Scraping: {}", url);

            match scrape_anime_series(url).await {
                Ok(data) => {
                    leptos::logging::log!(
                        "Scraped successfully: {} Episodes!",
                        data.episodes.len()
                    );
                    scraped_data.set(Some(data));
                }
                Err(e) => {
                    leptos::logging::log!("Scrape failed: {:?}", e)
                }
            }
        });
    };

    let on_sync = move |_| {
        leptos::logging::log!("Sync clicked");
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
                                <button class="btn btn-accent" on:click=on_sync>
                                    "Sync"
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
        }
}
