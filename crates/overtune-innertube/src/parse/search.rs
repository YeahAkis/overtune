use serde_json::Value;

use crate::models::{AlbumItem, ArtistItem, PlaylistItem, SongItem};
use crate::parse::{array, at, list_item, str_at};
use crate::search::SearchKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchResults {
    Songs(Vec<SongItem>),
    Artists(Vec<ArtistItem>),
    Albums(Vec<AlbumItem>),
    Playlists(Vec<PlaylistItem>),
}

impl SearchResults {
    pub fn len(&self) -> usize {
        match self {
            SearchResults::Songs(v) => v.len(),
            SearchResults::Artists(v) => v.len(),
            SearchResults::Albums(v) => v.len(),
            SearchResults::Playlists(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPage {
    pub results: SearchResults,
    pub continuation: Option<String>,
}

fn shelf(response: &Value) -> Option<&Value> {
    if let Some(s) = at(
        response,
        &["continuationContents", "musicShelfContinuation"],
    ) {
        return Some(s);
    }
    array(
        response,
        &[
            "contents",
            "tabbedSearchResultsRenderer",
            "tabs",
            "0",
            "tabRenderer",
            "content",
            "sectionListRenderer",
            "contents",
        ],
    )
    .iter()
    .rev()
    .find_map(|s| s.get("musicShelfRenderer"))
}

fn collect<T>(items: &[Value], f: fn(&Value) -> Option<T>) -> Vec<T> {
    items
        .iter()
        .filter_map(|i| i.get("musicResponsiveListItemRenderer"))
        .filter_map(f)
        .collect()
}

pub fn page(response: &Value, kind: SearchKind) -> SearchPage {
    let shelf = shelf(response);
    let items = shelf.map(|s| array(s, &["contents"])).unwrap_or_default();
    let results = match kind {
        SearchKind::Songs => SearchResults::Songs(collect(items, list_item::song)),
        SearchKind::Artists => SearchResults::Artists(collect(items, list_item::artist)),
        SearchKind::Albums => SearchResults::Albums(collect(items, list_item::album)),
        SearchKind::Playlists => SearchResults::Playlists(collect(items, list_item::playlist)),
    };
    let continuation = shelf
        .and_then(|s| {
            str_at(
                s,
                &["continuations", "0", "nextContinuationData", "continuation"],
            )
        })
        .map(str::to_owned);
    SearchPage {
        results,
        continuation,
    }
}
