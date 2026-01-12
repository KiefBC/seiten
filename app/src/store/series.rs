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
    /// Otherwise, returns the series model.
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
    /// Returns None if no series with the given ID exists.
    /// Otherwise, returns the series model.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<series::Model>, DbErr> {
        if id.is_nil() {
            return Err(DbErr::Custom("id cannot be nil UUID".to_string()));
        }

        Series::find_by_id(id).one(&self.db).await
    }

    /// Check if a series with the given slug exists.
    /// Returns true if it exists, false otherwise.
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
    /// Returns a vector of series models.
    pub async fn list_all(&self) -> Result<Vec<series::Model>, DbErr> {
        Series::find()
            .order_by_asc(series::Column::Title)
            .all(&self.db)
            .await
    }

    /// Insert a new series into the database.
    /// Returns the created series model.
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
    /// Returns the updated series model.
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
    /// Returns the updated series model.
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
    /// Returns the result of the delete operation.
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        if id.is_nil() {
            return Err(DbErr::Custom("id cannot be nil UUID".to_string()));
        }
        
        Series::delete_by_id(id).exec(&self.db).await
    }
}