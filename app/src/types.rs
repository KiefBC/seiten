use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Represents an episode of an anime series.
/// TODO: Grab data from AniDB to fill out the Japanese info
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EpisodeData {
    pub ep_number: i32,
    pub absolute_episode_number: i32,
    pub jap_release: Option<NaiveDate>,
    pub eng_release: NaiveDate,
    pub episode_type: EpisodeType,
    pub eng_title: String,
    pub jap_title: Option<String>,
    pub duration: Option<i32>,
    pub manga_chapters: Option<Vec<i32>>,
}

impl EpisodeData {
    pub fn new(
        ep_number: i32,
        absolute_episode_number: i32,
        jap_release: Option<NaiveDate>,
        eng_release: NaiveDate,
        episode_type: EpisodeType,
        eng_title: &str,
        jap_title: Option<&str>,
        duration: Option<i32>,
        manga_chapters: Option<&[i32]>,
    ) -> Self {
        EpisodeData {
            ep_number,
            absolute_episode_number,
            jap_release,
            eng_release,
            episode_type,
            eng_title: eng_title.to_string(),
            jap_title: jap_title.map(|title| title.to_string()),
            duration,
            manga_chapters: manga_chapters.map(|chapters| chapters.to_vec()),
        }
    }
}

/// Represents a series of anime.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SeriesData {
    pub slug: String,
    pub title: String,
    pub episodes: Vec<EpisodeData>,
}

impl SeriesData {
    pub fn new(slug: &str, title: &str, episodes: &[EpisodeData]) -> Self {
        SeriesData {
            slug: slug.to_string(),
            title: title.to_string(),
            episodes: episodes.to_vec(),
        }
    }
}

/// Represents a type of episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EpisodeType {
    Canon, // Default, assume its Canon
    Filler,
    Mixed,
    AnimeCanon,
}

#[cfg(feature = "ssr")]
impl From<EpisodeType> for ::entity::episode::EpisodeType {
    fn from(ep_type: EpisodeType) -> Self {
        match ep_type {
            EpisodeType::Canon => ::entity::episode::EpisodeType::Canon,
            EpisodeType::Filler => ::entity::episode::EpisodeType::Filler,
            EpisodeType::Mixed => ::entity::episode::EpisodeType::MixedCanon,
            EpisodeType::AnimeCanon => ::entity::episode::EpisodeType::AnimeCanon,
        }
    }
}
