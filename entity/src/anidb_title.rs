use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Title type from AniDB anime-titles.dat dump
/// See: https://wiki.anidb.net/API
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum TitleType {
    #[sea_orm(num_value = 1)]
    Primary,    // One per anime
    #[sea_orm(num_value = 2)]
    Synonym,    // Multiple per anime
    #[sea_orm(num_value = 3)]
    Short,      // Multiple per anime
    #[sea_orm(num_value = 4)]
    Official,   // One per language
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "anidb_titles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: Uuid,
    pub anime_id: i32,           // AniDB anime ID (aid)
    pub title_type: TitleType,   // 1=primary, 2=synonym, 3=short, 4=official
    pub language: String,        // Language code (e.g., "en", "ja", "x-jat")
    pub title: String,           // The actual title text
}

impl ActiveModelBehavior for ActiveModel {}
