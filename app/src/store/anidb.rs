use ::entity::anidb_title;
use sea_orm::prelude::Expr;
use sea_orm::*;

/// Encapsulates all database queries and mutations for the AniDB tables.
#[derive(Debug, Clone)]
pub struct AniDBStore {
    db: DatabaseConnection,
}

/// A simple struct representing a title entry for fuzzy matching.
/// Contains only the fields needed for matching, avoiding ORM overhead.
#[derive(Debug, Clone)]
pub struct TitleEntry {
    pub anime_id: i32,
    pub title: String,
    pub title_type: anidb_title::TitleType,
    pub language: String,
}

impl AniDBStore {
    /// Create a new AniDBStore with the given database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Fetch all titles from the anidb_titles table.
    pub async fn get_all_titles(&self) -> Result<Vec<TitleEntry>, DbErr> {
        let titles = anidb_title::Entity::find()
            .all(&self.db)
            .await?;

        Ok(titles
            .into_iter()
            .map(|t| TitleEntry {
                anime_id: t.anime_id,
                title: t.title,
                title_type: t.title_type,
                language: t.language,
            })
            .collect())
    }

    /// Fetch English and romanized Japanese titles (lang: "en", "x-jat").
    pub async fn get_english_titles(&self) -> Result<Vec<TitleEntry>, DbErr> {
        let titles = anidb_title::Entity::find()
            .filter(
                Condition::any()
                    .add(anidb_title::Column::Language.eq("en"))
                    .add(anidb_title::Column::Language.eq("x-jat"))
            )
            .all(&self.db)
            .await?;

        Ok(titles
            .into_iter()
            .map(|t| TitleEntry {
                anime_id: t.anime_id,
                title: t.title,
                title_type: t.title_type,
                language: t.language,
            })
            .collect())
    }

    /// Find an anime_id by exact title match (case-insensitive).
    /// Returns the first matching anime_id, or None if not found.
    pub async fn find_anime_id_by_exact_title(&self, title: &str) -> Result<Option<i32>, DbErr> {
        let normalized_title = title.to_lowercase();

        // Use raw SQL with LOWER() for SQLite compatibility
        let result = anidb_title::Entity::find()
            .filter(
                Expr::cust("LOWER(title)").eq(&normalized_title)
            )
            .one(&self.db)
            .await?;

        Ok(result.map(|t| t.anime_id))
    }
}
