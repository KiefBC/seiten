#![allow(unused_imports)]

#[cfg(feature = "ssr")]
use entity::prelude::AnidbDumpMeta;
use leptos::prelude::*;
use leptos::logging::log;

use quick_xml::Reader;
#[cfg(feature = "ssr")]
use serde::Deserialize;

#[cfg(feature = "ssr")]
use crate::{AppState, SeriesData, AniDBSeriesData, AniDBEpisodeData};

use crate::api::episodes::parse_episodes_from_html;

#[cfg(feature = "ssr")]
use crate::api::fuzzy_match::{smart_fuzzy_match, FuzzyMatchConfig};

use uuid::Uuid;

#[cfg(feature = "ssr")]
use ::entity::episode;

#[cfg(feature = "ssr")]
use sea_orm::ActiveValue::Set;

/// Root anime element from AniDB HTTP API response
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct AnimeXML {
    #[serde(rename = "@id")]
    id: i32,
    #[serde(rename = "@restricted")]
    restricted: Option<String>,
    #[serde(rename = "type")]
    anime_type: Option<String>,
    episodecount: Option<i32>,
    startdate: Option<String>,
    enddate: Option<String>,
    titles: Option<TitlesXML>,
    url: Option<String>,
    description: Option<String>,
    episodes: Option<EpisodesXML>,
}

/// Container for title elements
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct TitlesXML {
    #[serde(rename = "title", default)]
    titles: Vec<TitleXML>,
}

/// Single title element with language and type attributes
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct TitleXML {
    #[serde(rename = "@xml:lang")]
    lang: String,
    #[serde(rename = "@type")]
    title_type: String,
    #[serde(rename = "$text")]
    value: String,
}

/// Container for episode elements
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct EpisodesXML {
    #[serde(rename = "episode", default)]
    episodes: Vec<EpisodeXML>,
}

/// Single episode element from AniDB
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct EpisodeXML {
    #[serde(rename = "@id")]
    id: i32,
    #[serde(rename = "@update")]
    update: String,
    epno: EpisodeNumXML,
    length: Option<i32>,
    airdate: Option<String>,
    #[serde(rename = "title", default)]
    titles: Vec<EpisodeTitleXML>,
    summary: Option<String>,
    resources: Option<ResourcesXML>,
}

/// Episode number with type (1=regular, 2=special, etc.)
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct EpisodeNumXML {
    #[serde(rename = "@type")]
    epno_type: i32,
    #[serde(rename = "$text")]
    value: String,
}

/// Episode title with language attribute
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct EpisodeTitleXML {
    #[serde(rename = "@xml:lang")]
    lang: String,
    #[serde(rename = "$text")]
    value: String,
}

/// Container for resource elements (external links)
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct ResourcesXML {
    #[serde(rename = "resource", default)]
    resources: Vec<ResourceXML>,
}

/// External resource (Crunchyroll, etc.)
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct ResourceXML {
    #[serde(rename = "@type")]
    resource_type: i32,
    externalentity: Option<ExternalEntityXML>, // Do not rename/fix, serde will handle it
}

/// External entity containing identifier
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct ExternalEntityXML {
    identifier: Option<String>,
}

/// AniDB error response
#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct AniDBErrorXML {
    #[serde(rename = "$text")]
    message: String,
}

/// Parses AniDB XML response into AniDBSeriesData
#[cfg(feature = "ssr")]
pub fn parse_anidb_xml(xml: &str) -> Result<AniDBSeriesData, ServerFnError> {
    use chrono::NaiveDate;

    if xml.contains("<error>") {
        if let Ok(e) = quick_xml::de::from_str::<AniDBErrorXML>(xml) {
            return Err(ServerFnError::ServerError(format!(
                "AniDB API error: {}",
                e.message
            )));
        }
    }

    let anime: AnimeXML = match quick_xml::de::from_str(xml) {
        Ok(anime) => anime,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Failed to parse AniDB XML: {}",
                e
            )));
        }
    };

    let (title_main, title_ja, title_en) = extract_titles(&anime.titles);

    let start_date = anime
        .startdate
        .as_ref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let end_date = anime
        .enddate
        .as_ref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    // Filter to regular episodes only (type=1)
    let episodes: Vec<AniDBEpisodeData> = anime
        .episodes
        .map(|eps| {
            eps.episodes
                .into_iter()
                .filter(|ep| ep.epno.epno_type == 1) // Only regular episodes
                .filter_map(|ep| parse_episode_xml(ep).ok())
                .collect()
        })
        .unwrap_or_default();

    Ok(AniDBSeriesData::new(
        anime.id,
        anime.restricted.as_deref() == Some("true"),
        anime.anime_type.as_deref().unwrap_or("Unknown"),
        anime.episodecount,
        &episodes,
        start_date,
        end_date,
        &title_main,
        title_ja.as_deref(),
        title_en.as_deref(),
        anime.description.as_deref(),
        anime.url.as_deref(),
    ))
}

/// Extracts prioritized titles from AniDB title list.
#[cfg(feature = "ssr")]
fn extract_titles(titles_opt: &Option<TitlesXML>) -> (String, Option<String>, Option<String>) {
    let titles = match titles_opt {
        Some(t) => &t.titles,
        None => return ("Unknown".to_string(), None, None),
    };

    let main_title = titles
        .iter()
        .find(|t| t.title_type == "main")
        .map(|t| t.value.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let title_ja = titles
        .iter()
        .find(|t| t.lang == "ja")
        .or_else(|| titles.iter().find(|t| t.lang == "x-jat"))
        .map(|t| t.value.clone());

    let title_en = titles
        .iter()
        .find(|t| t.lang == "en")
        .map(|t| t.value.clone());

    (main_title, title_ja, title_en)
}

/// Parses a single episode from XML into AniDBEpisodeData
#[cfg(feature = "ssr")]
fn parse_episode_xml(ep: EpisodeXML) -> Result<AniDBEpisodeData, ServerFnError> {
    use chrono::NaiveDate;

    let update_date = match NaiveDate::parse_from_str(&ep.update, "%Y-%m-%d") {
        Ok(d) => d,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Failed to parse episode update date: {}",
                e
            )));
        }
    };

    let episode_num: i32 = ep.epno.value.trim().parse().unwrap_or(0);

    let airdate = ep
        .airdate
        .as_ref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let title_ja = ep
        .titles
        .iter()
        .find(|t| t.lang == "ja" || t.lang == "x-jat")
        .map(|t| t.value.clone());
    let title_en = ep.titles.iter().find(|t| t.lang == "en").map(|t| t.value.clone());

    let crunchyroll_id = ep
        .resources
        .and_then(|res| {
            res.resources
                .into_iter()
                .find(|r| r.resource_type == 28)
                .and_then(|r| r.externalentity)
                .and_then(|e| e.identifier)
        });

    Ok(AniDBEpisodeData::new(
        ep.id,
        update_date,
        episode_num,
        ep.epno.epno_type,
        ep.length,
        airdate,
        None, // rating
        None, // votes
        title_ja.as_deref(),
        title_en.as_deref(),
        ep.summary.as_deref(),
        crunchyroll_id.as_deref(),
    ))
}

/// Fetches anime data from AniDB HTTP API.
#[cfg(feature = "ssr")]
pub async fn parse_anidb_series(aid: &str) -> Result<String, ServerFnError> {
    let client_id = match std::env::var("ANIDB_CLIENT_ID") {
        Ok(val) => val,
        Err(_) => {
            log!("ANIDB_CLIENT_ID environment variable not set");
            return Err(ServerFnError::ServerError(
                "ANIDB_CLIENT_ID not set".to_string(),
            ));
        }
    };

    let client_version = match std::env::var("ANIDB_CLIENT_VERSION") {
        Ok(val) => val,
        Err(_) => {
            log!("ANIDB_CLIENT_VERSION environment variable not set");
            return Err(ServerFnError::ServerError(
                "ANIDB_CLIENT_VERSION not set".to_string(),
            ));
        }
    };

    let url = format!(
        "http://api.anidb.net:9001/httpapi?client={}&clientver={}&protover=1&request=anime&aid={}",
        client_id, client_version, aid
    );
    log!("Fetching AniDB data from: {}...", url);

    let state = expect_context::<AppState>();
    let response = match state.http.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            log!("HTTP request to AniDB API failed: {}", e);
            return Err(ServerFnError::ServerError(format!(
                "HTTP request failed: {}",
                e
            )));
        }
    };

    let xml = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            log!("Failed to read AniDB response: {}", e);
            return Err(ServerFnError::ServerError(format!(
                "Failed to read response: {}",
                e
            )));
        }
    };

    log!("Received AniDB XML: {} bytes", xml.len());
    Ok(xml)
}

/// Sends an HTTP GET request to the given URL and returns the response body as a string.
#[cfg(feature = "ssr")]
pub async fn parse_response(url: &str) -> Result<String, ServerFnError> {
    if url.trim().is_empty() {
        log!("Cannot send request: URL is empty");
        return Err(ServerFnError::ServerError(
            "URL cannot be empty".to_string(),
        ));
    }

    let app_state = expect_context::<AppState>();

    let response = match app_state.http.get(*&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!("HTTP request failed: {}", e)));
        }
    };

    if !response.status().is_success() {
        log!("HTTP request failed with status: {}", response.status());
        return Err(ServerFnError::ServerError(format!(
            "HTTP request returned status: {}",
            response.status()
        )));
    }

    log!("HTTP request successful, reading response body...");
    let body = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            return Err(ServerFnError::ServerError(e.to_string()));
        }
    };

    Ok(body)
}

/// Parses the given URL to extract the series slug.
#[cfg(feature = "ssr")]
pub async fn parse_url(url: &String) -> Result<String, ServerFnError> {
    use url::Url;

    let parsed_url = match Url::parse(&url) {
        Ok(u) => u,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!("Invalid URL: {}", e)));
        }
    };

    let slug = parsed_url
        .path_segments()
        .and_then(|mut segments| segments.next_back()) // Get the last segment of the path e.g. "naruto-shippuden"
        .unwrap_or("unknown-series")// This should help avoid panics, but ideally we should handle this case better.
        .to_string();

    Ok(slug)
}

/// This is the main orchestration function that composes all scraping operations.
#[cfg(feature = "ssr")]
pub async fn orchestrate_scrape(url: &str) -> Result<SeriesData, ServerFnError> {
    if url.trim().is_empty() {
        log!("Cannot scrape: URL is empty");
        return Err(ServerFnError::ServerError(
            "URL cannot be empty".to_string(),
        ));
    }

    let state = expect_context::<AppState>();

    log!("Parsing URL: {}...", url);
    let slug = parse_url(&url.to_string()).await?;
    let series_title = slug.replace("-", " ").to_uppercase();
    log!("Parsed URL successfully: {:?}", slug);

    log!("Sending a GET request to fetch the page content...");
    let body = parse_response(&url.to_string()).await?;
    log!("Response received, body length: {} bytes", body.len());

    let ep_series_data = parse_episodes_from_html(&body);
    log!("Total episodes found: {}", ep_series_data.len());
    let series_data = SeriesData::new(&slug, &series_title, &ep_series_data);

    log!("Attempting to fuzzy match series title against AniDB titles...");
    let anidb_id = {
        let config = FuzzyMatchConfig::default();
        match smart_fuzzy_match(&series_title, &state.anidb_store, &config).await {
            Some(result) => {
                log!(
                    "Found fuzzy match: '{}' -> '{}' (anime_id={}, score={:.3})",
                    series_title,
                    result.matched_title,
                    result.anime_id,
                    result.score
                );
                Some(result.anime_id)
            }
            None => {
                log!("No fuzzy match found for '{}', proceeding without AniDB ID", series_title);
                None
            }
        }
    };

    log!("Finding or creating series '{}' with anidb_id={:?}...", slug, anidb_id);
    let series_uuid = match state.series_store.find_or_create(&slug, &series_title, anidb_id).await {
        Ok(model) => {
            log!("Series ready: {} (id={})", slug, model.id);
            model.id
        }
        Err(e) => {
            log!("Failed to find or create series: {:?}", e);
            return Err(ServerFnError::ServerError(format!("Failed to find or create series: {}", e)));
        }
    };

    match state.episode_store.create_many(
        ep_series_data
            .into_iter()
            .map(|ep| episode::ActiveModel {
                id: Set(Uuid::new_v4()),
                show_id: Set(series_uuid),
                episode_num: Set(ep.ep_number),
                episode_type: Set(ep.episode_type.into()),
                title: Set(Some(ep.eng_title)),
                anidb_id: Set(None),
                title_ja: Set(None),
                airdate: Set(None),
                length: Set(None),
                summary: Set(None),
                crunchyroll_id: Set(None),
            })
            .collect(),
    ).await {
        Ok((created, skipped)) => {
            log!("Episode bulk creation complete for series '{}': {} created, {} skipped (duplicates)", slug, created, skipped);
        }
        Err(e) => {
            log!("Failed to create episodes: {:?}", e);
            return Err(ServerFnError::ServerError(format!("Failed to create episodes: {}", e)));
        }
    }

    if anidb_id.is_some() {
        log!("Auto-enriching series '{}' with AniDB metadata...", slug);
        match enrich_series_with_anidb(series_uuid).await {
            Ok((updated, unmatched)) => {
                log!(
                    "Auto-enrichment complete for '{}': {} episodes updated, {} unmatched",
                    slug,
                    updated,
                    unmatched
                );
            }
            Err(e) => {
                // Log the error but don't fail the whole scrape - we still have the basic data
                log!(
                    "WARNING: Auto-enrichment failed for '{}': {:?}. Basic data was saved.",
                    slug,
                    e
                );
            }
        }
    } else {
        log!(
            "No AniDB ID found for '{}' - skipping auto-enrichment. Manual enrichment may be needed.",
            slug
        );
    }

    Ok(series_data)
}

/// This is the main orchestration function that composes all AniDB scraping operations.
#[cfg(feature = "ssr")]
pub async fn orchestrate_anidb_scrape(anidb_id: &str) -> Result<AniDBSeriesData, ServerFnError> {
    if anidb_id.trim().is_empty() {
        log!("Cannot scrape AniDB: ID is empty");
        return Err(ServerFnError::ServerError(
            "AniDB ID cannot be empty".to_string(),
        ));
    }

    log!("Sending a GET request to fetch AniDB series data for ID: {}", anidb_id);
    let body = parse_anidb_series(anidb_id).await?;
    log!("AniDB response received, body length: {} bytes", body.len());

    let series_data = parse_anidb_xml(&body)?;
    log!(
        "Parsed AniDB series: '{}' with {} episodes",
        series_data.title_main,
        series_data.episodes.len()
    );

    Ok(series_data)
}

/// Enriches a series and its episodes with metadata from AniDB.
/// Fetches data from AniDB API and updates the database records.
#[cfg(feature = "ssr")]
pub async fn enrich_series_with_anidb(series_id: Uuid) -> Result<(u64, u64), ServerFnError> {
    if series_id.is_nil() {
        return Err(ServerFnError::ServerError(
            "Series ID cannot be nil UUID".to_string(),
        ));
    }

    let state = expect_context::<AppState>();

    let series = match state.series_store.find_by_id(series_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Err(ServerFnError::ServerError(format!(
                "Series not found: {}",
                series_id
            )));
        }
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Failed to fetch series: {}",
                e
            )));
        }
    };

    let anidb_id = match series.anidb_id {
        Some(id) => id,
        None => {
            return Err(ServerFnError::ServerError(format!(
                "Series '{}' has no AniDB ID - cannot enrich",
                series.slug
            )));
        }
    };

    log!(
        "Enriching series '{}' (id={}) with AniDB data (anidb_id={})",
        series.slug,
        series_id,
        anidb_id
    );

    let anidb_data = orchestrate_anidb_scrape(&anidb_id.to_string()).await?;

    match state.series_store.enrich_with_anidb(series_id, &anidb_data).await {
        Ok(updated_series) => {
            log!(
                "Series '{}' enriched: type={:?}, episode_count={:?}",
                updated_series.slug,
                updated_series.anime_type,
                updated_series.episode_count
            );
        }
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Failed to enrich series: {}",
                e
            )));
        }
    }

    let (updated, unmatched) = match state
        .episode_store
        .enrich_with_anidb(series_id, &anidb_data.episodes)
        .await
    {
        Ok(counts) => counts,
        Err(e) => {
            return Err(ServerFnError::ServerError(format!(
                "Failed to enrich episodes: {}",
                e
            )));
        }
    };

    log!(
        "Episode enrichment complete for '{}': {} updated, {} unmatched",
        series.slug,
        updated,
        unmatched
    );

    Ok((updated, unmatched))
}