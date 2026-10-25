use std::fs;

use overtune_innertube::{Thumbnail, best_thumbnail, parse, parse_duration, resize_thumbnail};
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn thumb(w: u32) -> Thumbnail {
    Thumbnail {
        url: format!("u{w}"),
        width: w,
        height: w,
    }
}

#[test]
fn suggestions_have_queries_and_songs() {
    let s = parse::suggestions(&fixture("suggestions_blinding.json"));
    assert!(s.queries.iter().any(|q| q.starts_with("blinding lights")));
    assert!(!s.songs.is_empty() && s.songs.len() <= 3);
    assert!(s.songs.iter().all(|x| x.video_id.len() == 11));
}

#[test]
fn suggestions_skip_music_videos() {
    let s = parse::suggestions(&fixture("suggestions_blinding.json"));
    assert!(s.songs.iter().all(|x| x.video_id != "4NRXx6U8ABQ"));
    assert_eq!(s.songs[0].title, "Blinding Lights");
}

#[test]
fn suggestion_song_has_artist_and_album() {
    let s = parse::suggestions(&fixture("suggestions_blinding.json"));
    let song = &s.songs[0];
    assert_eq!(song.artists.len(), 1);
    assert_eq!(song.artists[0].name, "The Weeknd");
    assert!(
        song.artists[0]
            .browse_id
            .as_deref()
            .unwrap()
            .starts_with("UC")
    );
    assert_eq!(song.album.as_ref().unwrap().title, "Blinding Lights");
    assert!(!song.thumbnails.is_empty());
}

#[test]
fn empty_response_gives_empty_suggestions() {
    let s = parse::suggestions(&serde_json::json!({}));
    assert!(s.queries.is_empty() && s.songs.is_empty());
}

#[test]
fn durations() {
    assert_eq!(parse_duration("3:20"), Some(200_000));
    assert_eq!(parse_duration("1:02:03"), Some(3_723_000));
    assert_eq!(parse_duration("0:07"), Some(7_000));
    assert_eq!(parse_duration("abc"), None);
    assert_eq!(parse_duration("3:2x"), None);
    assert_eq!(parse_duration(""), None);
}

#[test]
fn best_thumbnail_prefers_smallest_sufficient() {
    let list = vec![thumb(60), thumb(120), thumb(544)];
    assert_eq!(best_thumbnail(&list, 100).unwrap().width, 120);
    assert_eq!(best_thumbnail(&list, 1000).unwrap().width, 544);
    assert!(best_thumbnail(&[], 10).is_none());
}

#[test]
fn resize_googleusercontent_thumbnail() {
    assert_eq!(
        resize_thumbnail("https://lh3.googleusercontent.com/abc=w60-h60-l90-rj", 544),
        "https://lh3.googleusercontent.com/abc=w544-h544-l90-rj"
    );
    assert_eq!(
        resize_thumbnail("https://i.ytimg.com/vi/x/sddefault.jpg", 544),
        "https://i.ytimg.com/vi/x/sddefault.jpg"
    );
}

mod search {
    use overtune_innertube::parse::{list_item, search::page};
    use overtune_innertube::{SearchKind, SearchResults};
    use serde_json::json;

    use super::fixture;

    #[test]
    fn songs_page_parses() {
        let p = page(&fixture("search_songs_blinding.json"), SearchKind::Songs);
        let SearchResults::Songs(songs) = p.results else {
            panic!("wrong kind")
        };
        let first = &songs[0];
        assert_eq!(first.title, "Blinding Lights");
        assert_eq!(first.artists[0].name, "The Weeknd");
        assert!(first.duration_ms.unwrap().abs_diff(200_000) <= 2_000);
        assert!(first.album.is_some());
        assert_eq!(first.video_id.len(), 11);
        assert!(songs.len() >= 10);
        assert!(p.continuation.is_some());
    }

    #[test]
    fn continuation_parses() {
        let p = page(
            &fixture("search_songs_blinding_continuation.json"),
            SearchKind::Songs,
        );
        let SearchResults::Songs(songs) = p.results else {
            panic!("wrong kind")
        };
        assert!(!songs.is_empty());
        assert!(p.continuation.is_some());
    }

    #[test]
    fn artists_page_parses() {
        let p = page(&fixture("search_artists_weeknd.json"), SearchKind::Artists);
        let SearchResults::Artists(artists) = p.results else {
            panic!("wrong kind")
        };
        assert!(artists[0].name.contains("Weeknd"));
        assert!(artists[0].browse_id.starts_with("UC"));
        assert_eq!(
            artists[0].subscribers.as_deref(),
            Some("238M monthly audience")
        );
    }

    #[test]
    fn albums_page_parses() {
        let p = page(
            &fixture("search_albums_after_hours.json"),
            SearchKind::Albums,
        );
        let SearchResults::Albums(albums) = p.results else {
            panic!("wrong kind")
        };
        assert!(albums.iter().all(|a| a.browse_id.starts_with("MPREb_")));
        assert_eq!(albums[0].year.as_deref(), Some("2026"));
        assert_eq!(albums[0].artists[0].name, "weirdsoftly");
    }

    #[test]
    fn playlists_page_parses() {
        let p = page(
            &fixture("search_playlists_synthwave.json"),
            SearchKind::Playlists,
        );
        let SearchResults::Playlists(lists) = p.results else {
            panic!("wrong kind")
        };
        assert!(
            lists
                .iter()
                .all(|l| l.browse_id.starts_with("VL") || l.browse_id.starts_with("PL"))
        );
        assert_eq!(lists[0].title, "Top 50 Synthwave Tracks");
        assert_eq!(lists[0].author.as_deref(), Some("NeonAether"));
    }

    #[test]
    fn garbage_gives_empty_page() {
        let p = page(&json!({"contents": 5}), SearchKind::Songs);
        assert_eq!(p.results, SearchResults::Songs(vec![]));
        assert!(p.continuation.is_none());
    }

    fn artist_run(name: &str, id: &str) -> serde_json::Value {
        json!({"text": name, "navigationEndpoint": {"browseEndpoint": {"browseId": id,
            "browseEndpointContextSupportedConfigs": {"browseEndpointContextMusicConfig": {"pageType": "MUSIC_PAGE_TYPE_ARTIST"}}}}})
    }

    #[test]
    fn multi_artist_song_keeps_order() {
        let row = json!({
            "playlistItemData": {"videoId": "abcdefghijk"},
            "flexColumns": [
                {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [{"text": "this is what falling in love feels like"}]}}},
                {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [
                    artist_run("JVKE", "UC1"), {"text": ", "}, artist_run("CG5", "UC2"),
                    {"text": " • "}, {"text": "2:00"}
                ]}}}
            ]
        });
        let song = list_item::song(&row).unwrap();
        assert_eq!(song.artists[0].name, "JVKE");
        assert_eq!(song.artists[1].name, "CG5");
        assert_eq!(song.duration_ms, Some(120_000));
        assert_eq!(song.artists_text(), "JVKE, CG5");
    }

    #[test]
    fn unlinked_artists_are_split() {
        let row = json!({
            "playlistItemData": {"videoId": "abcdefghijk"},
            "flexColumns": [
                {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [{"text": "x"}]}}},
                {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [
                    {"text": "Song"}, {"text": " • "}, {"text": "A & B"}, {"text": " • "}, {"text": "3:00"}
                ]}}}
            ]
        });
        let names: Vec<String> = list_item::song(&row)
            .unwrap()
            .artists
            .into_iter()
            .map(|a| a.name)
            .collect();
        assert_eq!(names, vec!["A", "B"]);
    }

    #[test]
    fn explicit_badge_detected() {
        let row = json!({
            "playlistItemData": {"videoId": "abcdefghijk"},
            "badges": [{"musicInlineBadgeRenderer": {"icon": {"iconType": "MUSIC_EXPLICIT_BADGE"}}}],
            "flexColumns": [
                {"musicResponsiveListItemFlexColumnRenderer": {"text": {"runs": [{"text": "x"}]}}}
            ]
        });
        assert!(list_item::song(&row).unwrap().explicit);
    }
}
