use std::collections::BTreeMap;
use std::fs;

use overtune_library::{Library, Repeat, RightPanel, Session, SongRecord, play_threshold_ms};

fn song(id: &str) -> SongRecord {
    SongRecord {
        id: id.into(),
        title: "Blinding Lights".into(),
        artists_text: Some("The Weeknd".into()),
        duration_text: Some("3:20".into()),
        thumbnail_url: Some("https://x/t.jpg".into()),
        liked_at: None,
        total_play_time_ms: 0,
        blacklisted: false,
        explicit: false,
    }
}

#[test]
fn threshold() {
    assert_eq!(play_threshold_ms(Some(200_000)), 30_000);
    assert_eq!(play_threshold_ms(Some(60_000)), 30_000);
    assert_eq!(play_threshold_ms(Some(40_000)), 20_000);
    assert_eq!(play_threshold_ms(Some(12_000)), 6_000);
    assert_eq!(play_threshold_ms(Some(0)), 0);
    assert_eq!(play_threshold_ms(None), 30_000);
}

#[test]
fn listen_below_threshold_adds_time_but_no_event() {
    let lib = Library::in_memory().unwrap();
    lib.upsert_song(&song("a")).unwrap();
    assert!(!lib.record_listen("a", 10_000, Some(200_000), 1).unwrap());
    assert_eq!(lib.song("a").unwrap().unwrap().total_play_time_ms, 10_000);
    assert_eq!(lib.event_count("a").unwrap(), 0);
}

#[test]
fn listen_at_threshold_creates_event() {
    let lib = Library::in_memory().unwrap();
    lib.upsert_song(&song("a")).unwrap();
    assert!(lib.record_listen("a", 30_000, Some(200_000), 5).unwrap());
    assert_eq!(lib.event_count("a").unwrap(), 1);
    assert_eq!(lib.events("a").unwrap(), vec![(5, 30_000)]);
}

#[test]
fn short_song_counts_at_half() {
    let lib = Library::in_memory().unwrap();
    lib.upsert_song(&song("a")).unwrap();
    assert!(lib.record_listen("a", 20_000, Some(40_000), 1).unwrap());
    assert!(!lib.record_listen("a", 5_000, Some(12_000), 2).unwrap());
    assert!(lib.record_listen("a", 6_000, Some(12_000), 3).unwrap());
}

#[test]
fn total_play_time_accumulates() {
    let lib = Library::in_memory().unwrap();
    lib.upsert_song(&song("a")).unwrap();
    lib.record_listen("a", 10_000, None, 1).unwrap();
    lib.record_listen("a", 45_000, None, 2).unwrap();
    assert_eq!(lib.song("a").unwrap().unwrap().total_play_time_ms, 55_000);
}

#[test]
fn listen_for_unknown_song_is_error() {
    let lib = Library::in_memory().unwrap();
    assert!(lib.record_listen("missing", 40_000, None, 1).is_err());
}

#[test]
fn upsert_keeps_stats_and_like() {
    let lib = Library::in_memory().unwrap();
    let mut s = song("a");
    s.liked_at = Some(42);
    s.blacklisted = true;
    lib.upsert_song(&s).unwrap();
    lib.record_listen("a", 31_000, None, 1).unwrap();
    let mut update = song("a");
    update.title = "Blinding Lights (Remastered)".into();
    lib.upsert_song(&update).unwrap();
    let got = lib.song("a").unwrap().unwrap();
    assert_eq!(got.title, "Blinding Lights (Remastered)");
    assert_eq!(got.liked_at, Some(42));
    assert!(got.blacklisted);
    assert_eq!(got.total_play_time_ms, 31_000);
}

#[test]
fn missing_song_is_none() {
    assert!(Library::in_memory().unwrap().song("x").unwrap().is_none());
}

#[test]
fn session_roundtrip() {
    let lib = Library::in_memory().unwrap();
    assert!(lib.load_session().unwrap().is_none());
    let s = Session {
        queue_json: r#"[{"video_id":"a"}]"#.into(),
        current_index: Some(0),
        position_ms: 65_000,
        shuffle: true,
        repeat: Repeat::All,
        volume: 0.7,
        right_panel: RightPanel::Queue,
    };
    lib.save_session(&s).unwrap();
    assert_eq!(lib.load_session().unwrap(), Some(s.clone()));
    let mut t = s;
    t.position_ms = 1;
    t.current_index = None;
    t.repeat = Repeat::One;
    lib.save_session(&t).unwrap();
    assert_eq!(lib.load_session().unwrap(), Some(t));
}

#[test]
fn reopen_file_keeps_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/library.db");
    {
        let lib = Library::open(&path).unwrap();
        lib.upsert_song(&song("a")).unwrap();
    }
    let lib = Library::open(&path).unwrap();
    assert!(lib.song("a").unwrap().is_some());
}

#[test]
fn schema_matches_vitune_for_shared_tables() {
    let path = format!(
        "{}/tests/fixtures/vitune_31_columns.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let expected: BTreeMap<String, Vec<(String, String, bool)>> =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    let lib = Library::in_memory().unwrap();
    for (table, columns) in expected {
        assert_eq!(lib.table_columns(&table).unwrap(), columns, "{table}");
    }
}
