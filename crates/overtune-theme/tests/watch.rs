use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use async_channel::Receiver;
use overtune_theme::watch;

fn atomic_write(path: &Path, text: &str) {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text).unwrap();
    fs::rename(&tmp, path).unwrap();
}

fn collect(rx: &Receiver<Option<String>>, wait: Duration) -> Vec<Option<String>> {
    let end = Instant::now() + wait;
    let mut out = Vec::new();
    while Instant::now() < end {
        while let Ok(v) = rx.try_recv() {
            out.push(v);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    out
}

fn start(path: &Path, recheck: Duration) -> Receiver<Option<String>> {
    let rx = watch(path.to_path_buf(), Duration::from_millis(100), recheck);
    std::thread::sleep(Duration::from_millis(150));
    rx
}

#[test]
fn atomic_replace_emits_new_contents_once() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("palette.json");
    let rx = start(&path, Duration::from_secs(3));
    atomic_write(&path, "first");
    std::thread::sleep(Duration::from_millis(20));
    atomic_write(&path, "second");
    assert_eq!(
        collect(&rx, Duration::from_millis(1000)),
        vec![Some("second".to_string())]
    );
}

#[test]
fn delete_emits_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("palette.json");
    fs::write(&path, "x").unwrap();
    let rx = start(&path, Duration::from_secs(3));
    fs::remove_file(&path).unwrap();
    assert_eq!(collect(&rx, Duration::from_millis(800)), vec![None]);
}

#[test]
fn directory_created_later_is_picked_up() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("rice");
    let path = sub.join("palette.json");
    let rx = start(&path, Duration::from_millis(200));
    fs::create_dir(&sub).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    atomic_write(&path, "late");
    let got = collect(&rx, Duration::from_millis(2000));
    assert_eq!(got.last(), Some(&Some("late".to_string())));
}

#[test]
fn other_files_in_dir_are_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("palette.json");
    let rx = start(&path, Duration::from_secs(3));
    fs::write(dir.path().join("sibling.json"), "nope").unwrap();
    assert!(collect(&rx, Duration::from_millis(500)).is_empty());
}

#[test]
fn directory_removed_and_recreated_keeps_working() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("rice");
    fs::create_dir(&sub).unwrap();
    let path = sub.join("palette.json");
    fs::write(&path, "a").unwrap();
    let rx = start(&path, Duration::from_millis(200));
    fs::remove_dir_all(&sub).unwrap();
    std::thread::sleep(Duration::from_millis(400));
    fs::create_dir(&sub).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    atomic_write(&path, "b");
    let got = collect(&rx, Duration::from_millis(2000));
    assert!(got.contains(&None));
    assert_eq!(got.last(), Some(&Some("b".to_string())));
}
