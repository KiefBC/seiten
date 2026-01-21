use leptos::logging::log;
use rust_fuzzy_search::fuzzy_search_best_n;

use crate::store::{AniDBStore, TitleEntry};
use ::entity::anidb_title::TitleType;

/// Result of a fuzzy match operation.
#[derive(Debug, Clone)]
pub struct FuzzyMatchResult {
    pub anime_id: i32,
    pub matched_title: String,
    /// The fuzzy match score (0.0 to 1.0, higher is better).
    pub score: f32,
    pub title_type: TitleType,
    pub language: String,
}

/// Configuration for fuzzy matching.
#[derive(Debug, Clone)]
pub struct FuzzyMatchConfig {
    /// Minimum score threshold for a match to be considered valid.
    /// Default: 0.75 (75% similarity)
    pub threshold: f32,
    /// Number of top matches to consider.
    /// Default: 5
    pub top_n: usize,
    /// Whether to prioritize official titles over synonyms/short titles.
    /// Default: true
    pub prioritize_official: bool,
}

impl Default for FuzzyMatchConfig {
    fn default() -> Self {
        Self {
            threshold: 0.75,
            top_n: 5,
            prioritize_official: true,
        }
    }
}

/// Normalize a title for fuzzy matching.
pub fn normalize_title(title: &str) -> String {
    let mut normalized = title.to_lowercase();

    // Remove common patterns that don't affect core title based on AniDB conventions
    let patterns_to_remove = [
        " (tv)",
        " (ova)",
        " (movie)",
        " (special)",
        " (ona)", // I only found one title with this, but just in case
        " season 1",
        " season 2", // A lot of titles use "season X" format up to season 2, otherwise they use "part" or "nth season"
        " season 3",
        " season 4",
        " season 5", // We shouldn't expect more than 5 seasons in AniDB titles, but just in case
        " 1st season",
        " 2nd season",
        " 3rd season",
        " 4th season",
        " 5th season",
        " first season",
        " second season",
        " third season",
        " fourth season",
        " part 1",
        " part 2",
        " part i",
        " part ii", // This is unlikely but some titles might use roman numerals
        " the animation",
        " the movie", // Why do they do this? but it exists
    ];

    for pattern in patterns_to_remove {
        normalized = normalized.replace(pattern, "");
    }

    // Remove trailing year patterns like "(2020)"
    if let Some(idx) = normalized.rfind(" (") {
        if normalized[idx..].chars().filter(|c| c.is_numeric()).count() == 4 {
            normalized = normalized[..idx].to_string();
        }
    }

    normalized.trim().to_string()
}

/// Calculate a priority boost based on title type.
/// Official and Primary titles get a slight boost to prefer them over synonyms.
fn title_type_boost(title_type: &TitleType) -> f32 {
    match title_type {
        TitleType::Official => 0.05,
        TitleType::Primary => 0.03,
        TitleType::Synonym => 0.01,
        TitleType::Short => 0.0,
    }
}

/// Perform fuzzy matching against a list of titles.
/// Returns the best match above the threshold, or None if no match found.
pub fn fuzzy_match_title(
    query: &str,
    titles: &[TitleEntry],
    config: &FuzzyMatchConfig,
) -> Option<FuzzyMatchResult> {
    if titles.is_empty() {
        return None;
    }

    let normalized_query = normalize_title(query);
    log!("Normalized query: '{}' -> '{}'", query, normalized_query);

    let title_strings: Vec<&str> = titles.iter().map(|t| t.title.as_str()).collect();
    let matches = fuzzy_search_best_n(&normalized_query, &title_strings, config.top_n);

    let mut best_match: Option<FuzzyMatchResult> = None;
    let mut best_adjusted_score: f32 = 0.0;

    for (title_str, score) in matches {
        if let Some(entry) = titles.iter().find(|t| t.title.as_str() == title_str) {
            let boost = if config.prioritize_official {
                title_type_boost(&entry.title_type)
            } else {
                0.0
            };

            let adjusted_score = (score as f32 + boost).min(1.0);

            log!(
                "  Match: '{}' (anime_id={}, score={:.3}, adjusted={:.3}, type={:?})",
                title_str,
                entry.anime_id,
                score,
                adjusted_score,
                entry.title_type
            );

            if adjusted_score >= config.threshold && adjusted_score > best_adjusted_score {
                best_adjusted_score = adjusted_score;
                best_match = Some(FuzzyMatchResult {
                    anime_id: entry.anime_id,
                    matched_title: entry.title.clone(),
                    score: adjusted_score,
                    title_type: entry.title_type.clone(),
                    language: entry.language.clone(),
                });
            }
        }
    }

    best_match
}

/// Smart fuzzy matching with multi-pass strategy.
///
/// This function implements a cascading search strategy:
/// 1. First, try exact match (fastest)
/// 2. Then, try fuzzy match against English titles only (~30-40% of data)
/// 3. Finally, try fuzzy match against all titles (fallback)
///
/// This approach optimizes performance by reducing the search space in common cases.
pub async fn smart_fuzzy_match(
    query: &str,
    store: &AniDBStore,
    config: &FuzzyMatchConfig,
) -> Option<FuzzyMatchResult> {
    log!("Fuzzy matching: '{}'", query);

    // Pass 1: Exact match (case-insensitive)
    log!("Pass 1: Trying exact match...");
    match store.find_anime_id_by_exact_title(query).await {
        Ok(Some(anime_id)) => {
            log!("Found exact match: anime_id={}", anime_id);
            return Some(FuzzyMatchResult {
                anime_id,
                matched_title: query.to_string(),
                score: 1.0,
                title_type: TitleType::Primary, // Assume primary for exact matches
                language: "en".to_string(),
            });
        }
        Ok(None) => {
            log!("No exact match found");
        }
        Err(e) => {
            log!("Error during exact match lookup: {:?}", e);
        }
    }

    // Pass 2: Fuzzy match against English titles only
    log!("Pass 2: Trying fuzzy match against English titles...");
    match store.get_english_titles().await {
        Ok(english_titles) => {
            log!("Loaded {} English titles for fuzzy matching", english_titles.len());
            if let Some(result) = fuzzy_match_title(query, &english_titles, config) {
                log!(
                    "Found fuzzy match in English titles: '{}' (anime_id={}, score={:.3})",
                    result.matched_title,
                    result.anime_id,
                    result.score
                );
                return Some(result);
            }
            log!("No fuzzy match found in English titles");
        }
        Err(e) => {
            log!("Error loading English titles: {:?}", e);
        }
    }

    // Pass 3: Fuzzy match against all titles
    log!("Pass 3: Trying fuzzy match against all titles...");
    match store.get_all_titles().await {
        Ok(all_titles) => {
            log!("Loaded {} total titles for fuzzy matching", all_titles.len());
            if let Some(result) = fuzzy_match_title(query, &all_titles, config) {
                log!(
                    "Found fuzzy match in all titles: '{}' (anime_id={}, score={:.3})",
                    result.matched_title,
                    result.anime_id,
                    result.score
                );
                return Some(result);
            }
            log!("No fuzzy match found in all titles");
        }
        Err(e) => {
            log!("Error loading all titles: {:?}", e);
        }
    }

    log!("No match found for '{}'", query);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title() {
        assert_eq!(normalize_title("Naruto Shippuden"), "naruto shippuden");
        assert_eq!(normalize_title("One Piece (TV)"), "one piece");
        assert_eq!(normalize_title("Attack on Titan Season 1"), "attack on titan");
        assert_eq!(normalize_title("My Hero Academia 2nd Season"), "my hero academia");
        assert_eq!(normalize_title("Demon Slayer: The Movie"), "demon slayer:");
    }

    #[test]
    fn test_title_type_boost() {
        assert!(title_type_boost(&TitleType::Official) > title_type_boost(&TitleType::Primary));
        assert!(title_type_boost(&TitleType::Primary) > title_type_boost(&TitleType::Synonym));
        assert!(title_type_boost(&TitleType::Synonym) > title_type_boost(&TitleType::Short));
    }
}
