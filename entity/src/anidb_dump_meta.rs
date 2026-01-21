use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Metadata for tracking AniDB dump file downloads.
/// See: https://wiki.anidb.net/API — "YOU DO NOT REQUEST THIS FILE MORE THAN ONCE PER DAY"
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "anidb_dump_meta")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub id: Uuid,
    pub dump_name: String,                   // e.g., "anime-titles" to identify which dump
    pub last_fetched: Option<DateTime>,      // When we last downloaded the dump
    pub dump_created: Option<String>,        // The "# created:" timestamp from the dump header
    pub entry_count: Option<i32>,            // Number of entries imported from the dump
}

impl ActiveModelBehavior for ActiveModel {}