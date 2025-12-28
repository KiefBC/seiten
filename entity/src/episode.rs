use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
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
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "episodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub show_id: i32,
    pub episode_num: i32,
    pub episode_type: EpisodeType,
    pub title: Option<String>,
}

impl ActiveModelBehavior for ActiveModel {}