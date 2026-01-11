use ::entity::{prelude::*, episode};
use sea_orm::*;
use uuid::Uuid;

/// Encapsulates all database queries and mutations for the Episodes table.
#[derive(Debug, Clone)]
pub struct EpisodeStore {
    db: DatabaseConnection,
}

impl EpisodeStore {
    /// Create a new EpisodeStore with the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Find an episode by its UUID.
    /// Returns None if no episode with the given ID exists.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<episode::Model>, DbErr> {
        Episode::find_by_id(id).one(&self.db).await
    }

    /// Find all episodes for a given series, ordered by episode number.
    pub async fn find_by_series(&self, show_id: Uuid) -> Result<Vec<episode::Model>, DbErr> {
        Episode::find()
            .filter(episode::Column::ShowId.eq(show_id))
            .order_by_asc(episode::Column::EpisodeNum)
            .all(&self.db)
            .await
    }

    /// Insert a new episode into the database.
    /// Returns the created episode model.
    pub async fn create(&self, model: episode::ActiveModel) -> Result<episode::Model, DbErr> {
        model.insert(&self.db).await
    }

    /// Insert multiple episodes in a single transaction.
    /// More efficient than calling create() in a loop.
    pub async fn create_many(&self, models: Vec<episode::ActiveModel>) -> Result<(), DbErr> {
        if models.is_empty() {
            return Ok(());
        }

        Episode::insert_many(models).exec(&self.db).await?;

        Ok(())
    }

    /// Update an existing episode in the database.
    pub async fn update(&self, model: episode::ActiveModel) -> Result<episode::Model, DbErr> {
        model.update(&self.db).await
    }

    /// Delete an episode by its UUID.
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        Episode::delete_by_id(id).exec(&self.db).await
    }

    /// Delete all episodes for a given series.
    pub async fn delete_by_series(&self, show_id: Uuid) -> Result<DeleteResult, DbErr> {
        Episode::delete_many()
            .filter(episode::Column::ShowId.eq(show_id))
            .exec(&self.db)
            .await
    }

    /// Count episodes for a given series.
    pub async fn count_by_series(&self, show_id: Uuid) -> Result<u64, DbErr> {
        Episode::find()
            .filter(episode::Column::ShowId.eq(show_id))
            .count(&self.db)
            .await
    }
}
