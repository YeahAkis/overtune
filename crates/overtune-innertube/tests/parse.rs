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
