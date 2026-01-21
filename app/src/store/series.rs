use ::entity::{prelude::*, series};
use sea_orm::*;
use uuid::Uuid;

use crate::AniDBSeriesData;

/// Encapsulates all database queries and mutations for the Series table.
#[derive(Debug, Clone)]
pub struct SeriesStore {
    db: DatabaseConnection,
}

impl SeriesStore {
    /// Create a new SeriesStore with the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Find a series by its slug.
    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<series::Model>, DbErr> {
        if slug.trim().is_empty() {
            return Err(DbErr::Custom("slug cannot be empty".to_string()));
        }

        Series::find()
            .filter(series::Column::Slug.eq(slug))
            .one(&self.db)
            .await
    }

    /// Find a series by its UUID.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<series::Model>, DbErr> {
        if id.is_nil() {
            return Err(DbErr::Custom("id cannot be nil UUID".to_string()));
        }

        Series::find_by_id(id).one(&self.db).await
    }

    /// Check if a series with the given slug exists.
    pub async fn exists_by_slug(&self, slug: &str) -> Result<bool, DbErr> {
        if slug.trim().is_empty() {
            return Err(DbErr::Custom("slug cannot be empty".to_string()));
        }

        let count = Series::find()
            .filter(series::Column::Slug.eq(slug))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    /// List all series, ordered by title.
    pub async fn list_all(&self) -> Result<Vec<series::Model>, DbErr> {
        Series::find()
            .order_by_asc(series::Column::Title)
            .all(&self.db)
            .await
    }

    /// Insert a new series into the database.
    pub async fn create(&self, model: series::ActiveModel) -> Result<series::Model, DbErr> {
        let slug = match &model.slug {
            Set(s) => {
                if s.trim().is_empty() {
                    return Err(DbErr::Custom("slug cannot be empty".to_string()));
                }
                s
            }
            _ => return Err(DbErr::Custom("slug must be Set".to_string())),
        };

        match &model.title {
            Set(t) => {
                if t.trim().is_empty() {
                    return Err(DbErr::Custom(format!(
                        "title cannot be empty for series with slug '{}'",
                        slug
                    )));
                }
            }
            _ => {
                return Err(DbErr::Custom(format!(
                    "title must be Set for series with slug '{}'",
                    slug
                )))
            }
        }

        model.insert(&self.db).await
    }

    /// Update an existing series in the database.
    pub async fn update(&self, model: series::ActiveModel) -> Result<series::Model, DbErr> {
        let id = match &model.id {
            Set(id) => *id,
            _ => return Err(DbErr::Custom("id must be Set for update".to_string())),
        };

        if id.is_nil() {
            return Err(DbErr::Custom("id cannot be nil UUID".to_string()));
        }

        if let Set(slug) = &model.slug {
            if slug.trim().is_empty() {
                return Err(DbErr::Custom("slug cannot be empty".to_string()));
            }
        }

        if let Set(title) = &model.title {
            if title.trim().is_empty() {
                return Err(DbErr::Custom("title cannot be empty".to_string()));
            }
        }

        model.update(&self.db).await
    }

    /// Update the last_fetched timestamp for a series.
    pub async fn touch_last_fetched(&self, id: Uuid) -> Result<series::Model, DbErr> {
        if id.is_nil() {
            return Err(DbErr::Custom("id cannot be nil UUID".to_string()));
        }

        let model = Series::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("Series not found: {}", id)))?;

        let mut active_model: series::ActiveModel = model.into();
        active_model.last_fetched = Set(Some(chrono::Local::now()));

        active_model.update(&self.db).await
    }

    /// Delete a series by its UUID.
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        if id.is_nil() {
            return Err(DbErr::Custom("id cannot be nil UUID".to_string()));
        }

        Series::delete_by_id(id).exec(&self.db).await
    }

    /// Find an existing series by slug, or create a new one if it doesn't exist.
    pub async fn find_or_create(
        &self,
        slug: &str,
        title: &str,
        anidb_id: Option<i32>,
    ) -> Result<series::Model, DbErr> {
        if slug.trim().is_empty() {
            return Err(DbErr::Custom("slug cannot be empty".to_string()));
        }
        if title.trim().is_empty() {
            return Err(DbErr::Custom("title cannot be empty".to_string()));
        }

        if let Some(existing) = self.find_by_slug(slug).await? {
            return Ok(existing);
        }

        let new_series = series::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(title.to_string()),
            slug: Set(slug.to_string()),
            last_fetched: Set(Some(chrono::Local::now())),
            anidb_id: Set(anidb_id),
            anime_type: Set(None),
            episode_count: Set(None),
            start_date: Set(None),
            end_date: Set(None),
            title_ja: Set(None),
            description: Set(None),
            official_url: Set(None),
        };

        self.create(new_series).await
    }

    /// Enrich a series with metadata from AniDB.
    /// Updates anime_type, episode_count, dates, Japanese title, description, and URL.
    pub async fn enrich_with_anidb(
        &self,
        id: Uuid,
        data: &AniDBSeriesData,
    ) -> Result<series::Model, DbErr> {
        if id.is_nil() {
            return Err(DbErr::Custom("id cannot be nil UUID".to_string()));
        }

        let existing = Series::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("Series not found: {}", id)))?;

        let mut active_model: series::ActiveModel = existing.into();

        active_model.anidb_id = Set(Some(data.id));
        active_model.anime_type = Set(Some(data.anime_type.clone()));
        active_model.episode_count = Set(data.episode_count);
        active_model.start_date = Set(data.start_date);
        active_model.end_date = Set(data.end_date);
        active_model.title_ja = Set(data.title_ja.clone());
        active_model.description = Set(data.description.clone());
        active_model.official_url = Set(data.url.clone());
        active_model.last_fetched = Set(Some(chrono::Local::now()));

        active_model.update(&self.db).await
    }
}