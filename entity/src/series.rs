use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "series")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub slug: String,
    pub title: String,
    pub last_fetched: Option<DateTimeLocal>,
    #[sea_orm(has_many)]
    pub episodes: HasMany<super::episode::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
