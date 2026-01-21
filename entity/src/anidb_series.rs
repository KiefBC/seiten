use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "anidb_series")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub anime_id: i32, // AniDB anime ID
    pub restricted: bool,
    pub anime_type: String, // TV Series, Movie, OVA, etc.
    pub episode_count: Option<i32>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub title_main: String, // Main title (romanized)
    pub title_ja: Option<String>,
    pub title_en: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>, // Official website
    pub picture: Option<String>,
    pub last_updated: Option<ChronoDateTime>, // When this dump was last updated
    #[sea_orm(has_many)]
    pub episodes: HasMany<super::anidb_episode::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}