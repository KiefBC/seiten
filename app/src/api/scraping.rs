use leptos::prelude::*;
use leptos::logging::log;

#[cfg(feature = "ssr")]
use crate::{AppState, SeriesData};

use crate::api::episodes::parse_episodes_from_html;
use crate::api::series::create_series;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use ::entity::episode;

#[cfg(feature = "ssr")]
use sea_orm::*;

/// Sends an HTTP GET request to the given URL and returns the response body as a string.
#[cfg(feature = "ssr")]
pub async fn parse_response(url: &String) -> Result<String, ServerFnError> {
    if url.trim().is_empty() {
        log!("Cannot send request: URL is empty");
        return Err(ServerFnError::ServerError(
            "URL cannot be empty".to_string(),
        ));
    }

    let app_state = expect_context::<AppState>();

    log!("Sending HTTP GET request to URL: {}", url);
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

    log!("Parsing URL: {}", url);
    let slug = parse_url(&url.to_string()).await?;
    let series_title = slug.replace("-", " ").to_uppercase();
    log!("Parsed URL successfully: {:?}", slug);

    log!("Sending a GET request to fetch the page content...");
    let body = parse_response(&url.to_string()).await?;
    log!("Response received, body length: {} bytes", body.len());

    let ep_series_data = parse_episodes_from_html(&body);
    log!("Total episodes found: {}", ep_series_data.len());
    let series_data = SeriesData::new(&slug, &series_title, &ep_series_data);

    log!("Checking if series exists in the database...");
    let exists = {
        match state.series_store.exists_by_slug(&slug).await {
            Ok(exists) => exists,
            Err(e) => {
                log!("Failed to check series existence: {:?}", e);
                return Err(ServerFnError::ServerError(format!("Failed to check series existence: {}", e)));
            }
        }
    };

    let series_uuid = if exists {
        log!("Series '{}' already exists in the database.", slug);
        match state.series_store.find_by_slug(&slug).await {
            Ok(Some(model)) => model.id,
            Ok(None) => {
                log!("Series exists but could not be found, this should not happen.");
                return Err(ServerFnError::ServerError(
                    "Series exists but could not be found".to_string(), // This should not happen
                ));
            }
            Err(e) => {
                log!("Failed to retrieve existing series: {:?}", e);
                return Err(ServerFnError::ServerError(format!("Failed to retrieve existing series: {}", e)));
            }
        }
    } else {
        log!("Series '{}' does not exist in the database. Proceeding to create it.", slug);
        match create_series(slug.clone(), series_title.clone()).await {
            Ok(id) => {
                log!("Successfully created series with ID: {}", id);
                id
            }
            Err(e) => {
                log!("Failed to create series: {:?}", e);
                return Err(ServerFnError::ServerError(format!("Failed to create series: {}", e)));
            }
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
            })
            .collect(),
    ).await {
        Ok(_) => {
            log!("Successfully created episodes for series '{}'", slug);
        }
        Err(e) => {
            log!("Failed to create episodes: {:?}", e);
            return Err(ServerFnError::ServerError(format!("Failed to create episodes: {}", e)));
        }
    }

    Ok(series_data)
}