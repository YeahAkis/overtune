mod client;
mod context;
mod error;
mod models;
pub mod parse;
mod suggestions;

pub use client::Innertube;
pub use context::ClientContext;
pub use error::{Error, Result};
pub use models::{
    AlbumItem, AlbumRef, ArtistItem, ArtistRef, PlaylistItem, SongItem, Suggestions, Thumbnail,
};
pub use parse::parse_duration;
pub use parse::thumbnail::{best_thumbnail, resize_thumbnail};
