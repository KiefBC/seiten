use chrono::NaiveDate;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum EpisodeType {
    #[sea_orm(string_value = "canon")]
    Canon,
    #[sea_orm(string_value = "mixed")]
    MixedCanon,
    #[sea_orm(string_value = "filler")]
    Filler,
    #[sea_orm(string_value = "anime_canon")]
    AnimeCanon,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "episodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub show_id: Uuid,
    #[sea_orm(belongs_to, from = "show_id", to = "id")]
    pub series: HasOne<super::series::Entity>,
    pub episode_num: i32,
    pub episode_type: EpisodeType,
    pub title: Option<String>,
    // AniDB fields
    pub anidb_id: Option<i32>, // Episode ID from AniDB
    pub title_ja: Option<String>,
    pub airdate: Option<NaiveDate>,
    pub length: Option<i32>, // Episode length in minutes
    pub summary: Option<String>,
    pub crunchyroll_id: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}