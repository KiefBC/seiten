use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Represents an episode of an anime series.
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

/// Represents a type of episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EpisodeType {
    Canon, // Default, assume it's Canon
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

/// Represents an anime series from the AniDB XML scrape
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AniDBSeriesData {
    pub id: i32,
    pub restricted: bool,
    pub anime_type: String,
    pub episode_count: Option<i32>,
    pub episodes: Vec<AniDBEpisodeData>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub title_main: String,
    pub title_ja: Option<String>,
    pub title_en: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
}

impl AniDBSeriesData {
    pub fn new(
        id: i32,
        restricted: bool,
        anime_type: &str,
        episode_count: Option<i32>,
        episodes: &[AniDBEpisodeData],
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        title_main: &str,
        title_ja: Option<&str>,
        title_en: Option<&str>,
        description: Option<&str>,
        url: Option<&str>,
    ) -> Self {
        AniDBSeriesData {
            id,
            restricted,
            anime_type: anime_type.to_string(),
            episode_count,
            episodes: episodes.to_vec(),
            start_date,
            end_date,
            title_main: title_main.to_string(),
            title_ja: title_ja.map(|s| s.to_string()),
            title_en: title_en.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            url: url.map(|s| s.to_string())
        }
    }
}

/// Represents a single episode from the AniDB XML scrape
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AniDBEpisodeData {
    pub id: i32,
    pub update_date: NaiveDate,
    pub epno: i32,
    pub epno_type: i32,
    pub length: Option<i32>,
    pub airdate: Option<NaiveDate>,
    pub rating: Option<f64>,
    pub votes: Option<i32>,
    pub title_ja: Option<String>,
    pub title_en: Option<String>,
    pub summary: Option<String>,
    pub crunchyroll_id: Option<String>,
}

impl AniDBEpisodeData {
    pub fn new(
        id: i32,
        update_date: NaiveDate,
        epno: i32,
        epno_type: i32,
        length: Option<i32>,
        airdate: Option<NaiveDate>,
        rating: Option<f64>,
        votes: Option<i32>,
        title_ja: Option<&str>,
        title_en: Option<&str>,
        summary: Option<&str>,
        crunchyroll_id: Option<&str>,
    ) -> Self {
        AniDBEpisodeData {
            id,
            update_date,
            epno,
            epno_type,
            length,
            airdate,
            rating,
            votes,
            title_ja: title_ja.map(|s| s.to_string()),
            title_en: title_en.map(|s| s.to_string()),
            summary: summary.map(|s| s.to_string()),
            crunchyroll_id: crunchyroll_id.map(|s| s.to_string()),
        }
    }
}

/// Represents an entry from the AniDB Dump File
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AniDBDump {
    pub id: i32,
    pub series_type: i32,
    pub lang: String,
    pub title: String,
    pub hash: Option<String>,
}

impl AniDBDump {
    pub fn new(id: i32, series_type: i32, lang: &str, title: &str, hash: &str) -> Self {
        AniDBDump {
            id,
            series_type,
            lang: lang.to_string(),
            title: title.to_string(),
            hash: Some(hash.to_string()),
        }
    }
}