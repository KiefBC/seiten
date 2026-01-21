use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "anidb_episode")]
pub struct Model {
    #[sea_orm(primary_key, auto_icrement = false, column_type = "Text")]
    pub id: Uuid,
    pub anime_id: i32, // Foreign key to anidb_dump
    #[sea_orm(belongs_to, from = "anime_id", to = "anime_id")]
    pub anime: HasOne<super::anidb_series::Entity>,
    pub episode_id: i32, // Episode ID from AniDB
    pub update_date: NaiveDate, // Last update in AniDB
    pub epno: i32,
    pub epno_type: i32, // 1=regular, 2=special, 3=credit, 4=trailer, 5=parody, 6=other
    pub length: Option<i32>, // Episode length in minutes
    pub airdate: Option<NaiveDate>, // Original air date
    pub title_ja: Option<String>,
    pub title_en: Option<String>,
    pub summary: Option<String>,
    pub crunchyroll_id: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}