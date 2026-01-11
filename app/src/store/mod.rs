#[cfg(feature = "ssr")]
pub mod series;

#[cfg(feature = "ssr")]
pub mod episode;

#[cfg(feature = "ssr")]
pub use series::SeriesStore;

#[cfg(feature = "ssr")]
pub use episode::EpisodeStore;