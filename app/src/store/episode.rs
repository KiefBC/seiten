use ::entity::{prelude::*, episode};
use sea_orm::*;
use std::collections::HashSet;
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
        if id.is_nil() {
            return Err(DbErr::Custom("id cannot be nil UUID".to_string()));
        }

        Episode::find_by_id(id).one(&self.db).await
    }

    /// Find all episodes for a given series, ordered by episode number.
    /// Returns an empty vector if no episodes exist for the given show_id.
    /// Otherwise, returns a vector of episode models.
    pub async fn find_by_series(&self, show_id: Uuid) -> Result<Vec<episode::Model>, DbErr> {
        if show_id.is_nil() {
            return Err(DbErr::Custom("show_id cannot be nil UUID".to_string()));
        }

        Episode::find()
            .filter(episode::Column::ShowId.eq(show_id))
            .order_by_asc(episode::Column::EpisodeNum)
            .all(&self.db)
            .await
    }

    /// Find an episode by show_id and episode_num.
    /// Returns None if no episode with the given show_id and episode_num exists.
    /// Otherwise, returns the episode model.
    pub async fn find_by_show_and_num(
        &self,
        show_id: Uuid,
        episode_num: i32,
    ) -> Result<Option<episode::Model>, DbErr> {
        if show_id.is_nil() {
            return Err(DbErr::Custom("show_id cannot be nil UUID".to_string()));
        }

        if episode_num <= 0 {
            return Err(DbErr::Custom(format!(
                "episode_num must be positive, got {}",
                episode_num
            )));
        }

        Episode::find()
            .filter(episode::Column::ShowId.eq(show_id))
            .filter(episode::Column::EpisodeNum.eq(episode_num))
            .one(&self.db)
            .await
    }

    /// Insert a new episode into the database.
    /// Checks for duplicates and returns existing episode if found (idempotent).
    /// Returns the created or existing episode model.
    /// TODO: Eventually add a last_fetched field to episodes and update that on duplicates. Use it to determine whether to skip or update.
    pub async fn create(&self, model: episode::ActiveModel) -> Result<episode::Model, DbErr> {
        let show_id = match &model.show_id {
            Set(id) => *id,
            _ => return Err(DbErr::Custom("show_id must be Set".to_string())),
        };

        let episode_num = match &model.episode_num {
            Set(num) => *num,
            _ => return Err(DbErr::Custom("episode_num must be Set".to_string())),
        };

        if let Some(existing) = self.find_by_show_and_num(show_id, episode_num).await? {
            return Ok(existing);
        }

        model.insert(&self.db).await
    }

    /// Insert multiple episodes in a single transaction.
    /// Checks for duplicates and only inserts new episodes.
    /// Returns a tuple of (created_count, skipped_count).
    /// TODO: Eventually add a last_fetched field to episodes and update that on duplicates. Use it to determine whether to skip or update.
    pub async fn create_many(&self, models: Vec<episode::ActiveModel>) -> Result<(u64, u64), DbErr> {
        if models.is_empty() {
            return Ok((0, 0));
        }

        for (index, model) in models.iter().enumerate() {
            let show_id = match &model.show_id {
                Set(id) => {
                    if id.is_nil() {
                        return Err(DbErr::Custom(format!(
                            "Model at index {}: show_id cannot be nil UUID",
                            index
                        )));
                    }
                    *id
                }
                _ => {
                    return Err(DbErr::Custom(format!(
                        "Model at index {}: show_id must be Set",
                        index
                    )))
                }
            };

            match &model.episode_num {
                Set(num) => {
                    if *num <= 0 {
                        return Err(DbErr::Custom(format!(
                            "Model at index {} (show_id={}): episode_num must be positive, got {}",
                            index, show_id, num
                        )));
                    }
                }
                _ => {
                    return Err(DbErr::Custom(format!(
                        "Model at index {} (show_id={}): episode_num must be Set",
                        index, show_id
                    )))
                }
            }
        }

        let show_ids: HashSet<Uuid> = models
            .iter()
            .filter_map(|model| match &model.show_id {
                Set(id) => Some(*id),
                _ => None,
            })
            .collect();

        let existing_episodes: Vec<episode::Model> = Episode::find()
            .filter(episode::Column::ShowId.is_in(show_ids))
            .all(&self.db)
            .await?;

        let existing_keys: HashSet<(Uuid, i32)> = existing_episodes
            .into_iter()
            .map(|ep| (ep.show_id, ep.episode_num))
            .collect();

        let total_count = models.len();
        let new_episodes: Vec<episode::ActiveModel> = models
            .into_iter()
            .filter(|model| {
                let show_id = match &model.show_id {
                    Set(id) => *id,
                    _ => return false, // Skip if show_id is not Set
                };
                
                let episode_num = match &model.episode_num {
                    Set(num) => *num,
                    _ => return false, // Skip if episode_num is not Set
                };

                // Keep only if this (show_id, episode_num) doesn't exist
                !existing_keys.contains(&(show_id, episode_num))
            })
            .collect();

        let created_count = new_episodes.len() as u64;
        let skipped_count = (total_count - new_episodes.len()) as u64;

        if !new_episodes.is_empty() {
            Episode::insert_many(new_episodes).exec(&self.db).await?;
        }

        Ok((created_count, skipped_count))
    }

    /// Update an existing episode in the database.
    /// Returns the updated episode model.
    pub async fn update(&self, model: episode::ActiveModel) -> Result<episode::Model, DbErr> {
        let id = match &model.id {
            Set(id) => *id,
            _ => return Err(DbErr::Custom("id must be Set for update".to_string())),
        };

        if id.is_nil() {
            return Err(DbErr::Custom("id cannot be nil UUID".to_string()));
        }

        if let Set(show_id) = &model.show_id {
            if show_id.is_nil() {
                return Err(DbErr::Custom("show_id cannot be nil UUID".to_string()));
            }
        }

        if let Set(episode_num) = &model.episode_num {
            if *episode_num <= 0 {
                return Err(DbErr::Custom(format!(
                    "episode_num must be positive, got {}",
                    episode_num
                )));
            }
        }

        model.update(&self.db).await
    }

    /// Delete an episode by its UUID.
    /// Returns the number of deleted episodes (0 or 1).
    pub async fn delete(&self, id: Uuid) -> Result<DeleteResult, DbErr> {
        if id.is_nil() {
            return Err(DbErr::Custom("id cannot be nil UUID".to_string()));
        }

        Episode::delete_by_id(id).exec(&self.db).await
    }

    /// Delete all episodes for a given series.
    /// Returns the number of deleted episodes.
    pub async fn delete_by_series(&self, show_id: Uuid) -> Result<DeleteResult, DbErr> {
        if show_id.is_nil() {
            return Err(DbErr::Custom("show_id cannot be nil UUID".to_string()));
        }

        Episode::delete_many()
            .filter(episode::Column::ShowId.eq(show_id))
            .exec(&self.db)
            .await
    }

    /// Count episodes for a given series.
    /// Returns the count of episodes.
    pub async fn count_by_series(&self, show_id: Uuid) -> Result<u64, DbErr> {
        if show_id.is_nil() {
            return Err(DbErr::Custom("show_id cannot be nil UUID".to_string()));
        }

        Episode::find()
            .filter(episode::Column::ShowId.eq(show_id))
            .count(&self.db)
            .await
    }
}
