use chrono::NaiveDate;
use leptos::{prelude::*, logging::log};
use scraper::{Html, Selector};
use uuid::Uuid;

use crate::{EpisodeData, EpisodeType};

#[cfg(feature = "ssr")]
use crate::AppState;
#[cfg(feature = "ssr")]
use ::entity::episode;
#[cfg(feature = "ssr")]
use sea_orm::*;

/// Parse episode data from HTML content of an anime episode list page.
pub fn parse_episodes_from_html(body: &str) -> Vec<EpisodeData> {
    let dom = Html::parse_document(&body);

    let table_episode_list_selector =
        Selector::parse("table.EpisodeList tbody tr").expect("Valid CSS selector");

    let ep_num_selector = Selector::parse("td.Number").expect("Valid CSS selector");
    let ep_title_selector = Selector::parse("td.Title").expect("Valid CSS selector");
    let ep_type_selector = Selector::parse("td.Type").expect("Valid CSS selector");
    let ep_date_selector = Selector::parse("td.Date").expect("Valid CSS selector");

    let mut ep_series_data: Vec<EpisodeData> = Vec::new();

    let mut absolute_count = 0;
    for row in dom.select(&table_episode_list_selector) {
        absolute_count += 1;

        let num = row
            .select(&ep_num_selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string());
        let title = row
            .select(&ep_title_selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string());
        let ep_type = row
            .select(&ep_type_selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string());
        // This is assuming the date is always in YYYY-MM-DD format on AnimeFillerList
        let date = row
            .select(&ep_date_selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string());

        let ep_data = EpisodeData::new(
            num.as_ref()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0),
            absolute_count,
            None, // No AniDB scrape yet
            date.as_deref()
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
            match ep_type.as_deref() {
                Some("Canon") => EpisodeType::Canon,
                Some("Filler") => EpisodeType::Filler,
                Some("Mixed") => EpisodeType::Mixed,
                Some("Anime Canon") => EpisodeType::AnimeCanon,
                _ => EpisodeType::Canon,
            },
            title.as_deref().unwrap_or("Untitled"),
            None,
            None,
            None,
        );

        ep_series_data.push(ep_data);
    }

    ep_series_data
}

/// Create a single episode for a series.
#[server(endpoint = "episodes/create", prefix = "/api/v1")]
pub async fn create_episode(
    show_id: Uuid,
    episode_num: i32,
    episode_type: EpisodeType,
    title: Option<String>,
) -> Result<Uuid, ServerFnError> {
    let state = expect_context::<AppState>();

    log!("Creating episode {} for series {}", episode_num, show_id);

    let new_episode = episode::ActiveModel {
        id: Set(Uuid::new_v4()),
        show_id: Set(show_id),
        episode_num: Set(episode_num),
        episode_type: Set(episode_type.into()),
        title: Set(title),
    };

    match state.episode_store.create(new_episode).await {
        Ok(model) => {
            log!("Episode created: {}", model.id);
            Ok(model.id)
        }
        Err(e) => Err(ServerFnError::ServerError(format!(
            "Failed to create episode: {}",
            e
        ))),
    }
}

/// Get an episode by its UUID.
#[cfg(feature = "ssr")]
#[server(endpoint = "episodes/get", prefix = "/api/v1")]
pub async fn get_episode_by_id(id: Uuid) -> Result<Option<episode::Model>, ServerFnError> {
    let state = expect_context::<AppState>();

    match state.episode_store.find_by_id(id).await {
        Ok(episode) => Ok(episode),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

/// List all episodes for a given series, ordered by episode number.
#[cfg(feature = "ssr")]
#[server(endpoint = "episodes/list", prefix = "/api/v1")]
pub async fn list_episodes_by_series(
    show_id: Uuid,
) -> Result<Vec<episode::Model>, ServerFnError> {
    let state = expect_context::<AppState>();

    match state.episode_store.find_by_series(show_id).await {
        Ok(episodes) => Ok(episodes),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

/// Delete all episodes for a given series.
#[cfg(feature = "ssr")]
#[server(endpoint = "episodes/delete_by_series", prefix = "/api/v1")]
pub async fn delete_episodes_by_series(show_id: Uuid) -> Result<u64, ServerFnError> {
    let state = expect_context::<AppState>();

    log!("Deleting all episodes for series {}", show_id);

    match state.episode_store.delete_by_series(show_id).await {
        Ok(result) => {
            log!("Deleted {} episodes", result.rows_affected);
            Ok(result.rows_affected)
        }
        Err(e) => Err(ServerFnError::ServerError(format!(
            "Failed to delete episodes: {}",
            e
        ))),
    }
}

/// Count episodes for a given series.
#[cfg(feature = "ssr")]
#[server(endpoint = "episodes/count", prefix = "/api/v1")]
pub async fn count_episodes_by_series(show_id: Uuid) -> Result<u64, ServerFnError> {
    let state = expect_context::<AppState>();

    match state.episode_store.count_by_series(show_id).await {
        Ok(count) => Ok(count),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}
