use ::entity::{prelude::*, series};
use sea_orm::*;
use uuid::Uuid;

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
    /// Returns None if no series with the given slug exists.
    pub async fn find_by_slug(&self, slug: &str) -> Result<Option<series::Model>, DbErr> {
        Series::find()
            .filter(series::Column::Slug.eq(slug))
            .one(&self.db)
            .await
    }

    /// Find a series by its UUID.
    /// Returns None if no series with the given ID exists.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<series::Model>, DbErr> {
        Series::find_by_id(id).one(&self.db).await
    }

    /// Check if a series with the given slug exists.
    /// More efficient than find_by_slug when you only need existence check.
    pub async fn exists_by_slug(&self, slug: &str) -> Result<bool, DbErr> {
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
    /// Returns the created series model.
    pub async fn create(&self, model: series::ActiveModel) -> Result<series::Model, DbErr> {
        model.insert(&self.db).await
    }

    /// Update an existing series in the database.
    pub async fn update(&self, model: series::ActiveModel) -> Result<series::Model, DbErr> {
        model.update(&self.db).await
    }

    /// Update the last_fetched timestamp for a series.
    pub async fn touch_last_fetched(&self, id: Uuid) -> Result<series::Model, DbErr> {
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
        Series::delete_by_id(id).exec(&self.db).await
    }
}