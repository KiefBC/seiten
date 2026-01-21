#![allow(unused_imports)]
use leptos::{prelude::*, logging::log};
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::AppState;
#[cfg(feature = "ssr")]
use ::entity::series;
#[cfg(feature = "ssr")]
use sea_orm::*;

/// Create a new series with the given slug and title.
/// This is idempotent - if the series already exists, returns its UUID.
#[server(endpoint = "series/create", prefix = "/api/v1")]
pub async fn create_series(slug: String, title: String) -> Result<Uuid, ServerFnError> {
    let state = expect_context::<AppState>();

    log!("Creating or finding series: {}", slug);
    match state.series_store.find_or_create(&slug, &title, None).await {
        Ok(model) => {
            log!("Series ready: {} (id={})", slug, model.id);
            Ok(model.id)
        }
        Err(e) => {
            Err(ServerFnError::ServerError(format!("Failed to create series: {}", e)))
        },
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