use leptos::{prelude::*, logging::log};
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::AppState;
#[cfg(feature = "ssr")]
use ::entity::series;
#[cfg(feature = "ssr")]
use sea_orm::*;

/// Create a new series with the given slug and title.
#[server(endpoint = "series/create", prefix = "/api/v1")]
pub async fn create_series(slug: String, title: String) -> Result<Uuid, ServerFnError> {
    if slug.trim().is_empty() || title.trim().is_empty() {
        return Err(ServerFnError::ServerError(
            "Slug and title cannot be empty".to_string(),
        ));
    }

    let state = expect_context::<AppState>();

    log!("Checking if {} exists...", slug);
    let exists = match state.series_store.exists_by_slug(&slug).await {
        Ok(exists) => exists,
        Err(e) => {
            return Err(ServerFnError::ServerError(e.to_string()));
        }
    };

    if !exists {
        log!("Series does not exist, creating...");
        let new_series = series::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(title),
            slug: Set(slug),
            last_fetched: Set(Some(chrono::Local::now())),
        };

        match state.series_store.create(new_series).await {
            Ok(model) => Ok(model.id),
            Err(e) => Err(ServerFnError::ServerError(format!("Failed to create series: {}", e))),
        }
    } else {
        log!("Series already exists, skipping creation.");
        match state.series_store.find_by_slug(&slug).await {
            Ok(Some(model)) => Ok(model.id),
            Ok(None) => Err(ServerFnError::ServerError(
                "Series exists but could not be found".to_string(), // This should not happen
            )),
            Err(e) => Err(ServerFnError::ServerError(e.to_string())),
        }
    }
}

/// Get a series by its slug.
#[cfg(feature = "ssr")]
#[server(endpoint = "series/get", prefix = "/api/v1")]
pub async fn get_series_by_slug(slug: String) -> Result<Option<series::Model>, ServerFnError> {
    let state = expect_context::<AppState>();

    match state.series_store.find_by_slug(&slug).await {
        Ok(series) => Ok(series),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

/// List all series.
#[cfg(feature = "ssr")]
#[server(endpoint = "series/list", prefix = "/api/v1")]
pub async fn list_all_series() -> Result<Vec<series::Model>, ServerFnError> {
    let state = expect_context::<AppState>();

    match state.series_store.list_all().await {
        Ok(series) => Ok(series),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}
