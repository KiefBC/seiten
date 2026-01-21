use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "series")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub slug: String,
    pub title: String,
    pub last_fetched: Option<DateTimeLocal>,
    #[sea_orm(has_many)]
    pub episodes: HasMany<super::episode::Entity>,
    // AniDB fields
    pub anidb_id: Option<i32>, // Anime ID from AniDB
    pub anime_type: Option<String>, // TV Series, Movie, OVA, etc.
    pub episode_count: Option<i32>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>, // Series end date (None if ongoing)
    pub title_ja: Option<String>,
    pub description: Option<String>,
    pub official_url: Option<String>, // Official website URL
}

impl ActiveModelBehavior for ActiveModel {}
