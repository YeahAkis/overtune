mod client;
mod context;
mod error;
mod models;
pub mod parse;
mod player;
mod search;
mod suggestions;

pub use client::Innertube;
pub use context::ClientContext;
pub use error::{Error, Result};
pub use models::{
    AlbumItem, AlbumRef, ArtistItem, ArtistRef, PlaylistItem, SongItem, Suggestions, Thumbnail,
};
pub use parse::parse_duration;
pub use parse::search::{SearchPage, SearchResults};
pub use parse::thumbnail::{best_thumbnail, resize_thumbnail};
pub use player::{AudioFormat, PlayerInfo, pick_format};
pub use search::SearchKind;
