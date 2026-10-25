use serde_json::Value;

use crate::models::{AlbumRef, ArtistRef, SongItem};
use crate::parse::thumbnail::thumbnails;
use crate::parse::{array, parse_duration, runs, str_at, text};

const SEPARATOR: &str = " • ";
const TYPE_LABELS: [&str; 8] = [
    "Song", "Video", "Album", "Single", "EP", "Artist", "Playlist", "Episode",
];

pub fn page_type(run: &Value) -> Option<&str> {
    str_at(
        run,
        &[
            "navigationEndpoint",
            "browseEndpoint",
            "browseEndpointContextSupportedConfigs",
            "browseEndpointContextMusicConfig",
            "pageType",
        ],
    )
}

pub fn browse_id(run: &Value) -> Option<String> {
    str_at(run, &["navigationEndpoint", "browseEndpoint", "browseId"]).map(str::to_owned)
}

pub fn flex_column(renderer: &Value, index: usize) -> Option<&Value> {
    array(renderer, &["flexColumns"])
        .get(index)?
        .get("musicResponsiveListItemFlexColumnRenderer")?
        .get("text")
}

fn fixed_column(renderer: &Value, index: usize) -> Option<&Value> {
    array(renderer, &["fixedColumns"])
        .get(index)?
        .get("musicResponsiveListItemFixedColumnRenderer")?
        .get("text")
}

fn detail_runs(renderer: &Value) -> Vec<&Value> {
    array(renderer, &["flexColumns"])
        .iter()
        .skip(1)
        .filter_map(|c| c.pointer("/musicResponsiveListItemFlexColumnRenderer/text"))
        .flat_map(|t| runs(t).iter())
        .collect()
}

pub fn video_type(renderer: &Value) -> Option<&str> {
    let path = [
        "watchEndpoint",
        "watchEndpointMusicSupportedConfigs",
        "watchEndpointMusicConfig",
        "musicVideoType",
    ];
    let from_nav = renderer
        .get("navigationEndpoint")
        .and_then(|n| str_at(n, &path));
    from_nav.or_else(|| {
        let first = runs(flex_column(renderer, 0)?).first()?;
        str_at(first.get("navigationEndpoint")?, &path)
    })
}

pub fn type_label(renderer: &Value) -> Option<String> {
    let first = runs(flex_column(renderer, 1)?)
        .first()?
        .get("text")?
        .as_str()?;
    TYPE_LABELS.contains(&first).then(|| first.to_owned())
}

pub fn is_song(renderer: &Value) -> bool {
    match video_type(renderer) {
        Some(t) => t == "MUSIC_VIDEO_TYPE_ATV",
        None => type_label(renderer).as_deref() == Some("Song"),
    }
}

fn video_id(renderer: &Value) -> Option<String> {
    str_at(renderer, &["playlistItemData", "videoId"])
        .or_else(|| {
            str_at(
                renderer,
                &["navigationEndpoint", "watchEndpoint", "videoId"],
            )
        })
        .or_else(|| {
            let first = runs(flex_column(renderer, 0)?).first()?;
            str_at(first, &["navigationEndpoint", "watchEndpoint", "videoId"])
        })
        .map(str::to_owned)
}

pub fn segments(runs: &[&Value]) -> Vec<Vec<String>> {
    let mut out = vec![Vec::new()];
    for run in runs {
        let t = run.get("text").and_then(Value::as_str).unwrap_or_default();
        if t == SEPARATOR {
            out.push(Vec::new());
        } else if let Some(last) = out.last_mut() {
            last.push(t.to_owned());
        }
    }
    out.retain(|s| !s.is_empty());
    out
}

fn split_names(text: &str) -> Vec<ArtistRef> {
    text.split(", ")
        .flat_map(|p| p.split(" & "))
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|name| ArtistRef {
            name: name.to_owned(),
            browse_id: None,
        })
        .collect()
}

pub fn artists(runs: &[&Value]) -> Vec<ArtistRef> {
    let linked: Vec<ArtistRef> = runs
        .iter()
        .filter(|r| {
            matches!(
                page_type(r),
                Some("MUSIC_PAGE_TYPE_ARTIST" | "MUSIC_PAGE_TYPE_USER_CHANNEL")
            )
        })
        .filter_map(|r| {
            Some(ArtistRef {
                name: r.get("text")?.as_str()?.to_owned(),
                browse_id: browse_id(r),
            })
        })
        .collect();
    if !linked.is_empty() {
        return linked;
    }
    segments(runs)
        .into_iter()
        .map(|s| s.concat())
        .find(|s| !TYPE_LABELS.contains(&s.as_str()) && parse_duration(s).is_none())
        .map(|s| split_names(&s))
        .unwrap_or_default()
}

pub fn explicit(renderer: &Value) -> bool {
    array(renderer, &["badges"]).iter().any(|b| {
        str_at(b, &["musicInlineBadgeRenderer", "icon", "iconType"]) == Some("MUSIC_EXPLICIT_BADGE")
    })
}

pub fn song(renderer: &Value) -> Option<SongItem> {
    let video_id = video_id(renderer)?;
    let title = text(flex_column(renderer, 0)?)?;
    let details = detail_runs(renderer);
    let album = details
        .iter()
        .find(|r| page_type(r) == Some("MUSIC_PAGE_TYPE_ALBUM"))
        .and_then(|r| {
            Some(AlbumRef {
                title: r.get("text")?.as_str()?.to_owned(),
                browse_id: browse_id(r),
            })
        });
    let duration_ms = fixed_column(renderer, 0)
        .and_then(text)
        .and_then(|t| parse_duration(&t))
        .or_else(|| {
            details
                .iter()
                .filter_map(|r| r.get("text").and_then(Value::as_str))
                .find_map(parse_duration)
        });
    let artist_runs: Vec<&Value> = flex_column(renderer, 1)
        .map(|c| runs(c).iter().collect())
        .unwrap_or_default();
    Some(SongItem {
        video_id,
        title,
        artists: artists(&artist_runs),
        album,
        duration_ms,
        thumbnails: renderer
            .get("thumbnail")
            .map(thumbnails)
            .unwrap_or_default(),
        explicit: explicit(renderer),
    })
}
