use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Represents an episode of an anime series.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EpisodeData {
    pub ep_number: u32,
    pub jap_release: NaiveDate,
    pub eng_release: NaiveDate,
    pub episode_type: EpisodeType,
    pub eng_title: String,
    pub jap_title: Option<String>,
    pub duration: Option<u32>,
    pub manga_chapters: Option<Vec<u32>>,
}

/// Represents a series of anime.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SeriesData {
    pub slug: String,
    pub title: String,
    pub episodes: Vec<EpisodeData>,
}

/// Represents a type of episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EpisodeType {
    Canon,
    Filler,
    Mixed,
    AnimeCanon,
}
