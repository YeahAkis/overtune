use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use async_channel::{Receiver, Sender};
use notify::{Event, RecursiveMode, Watcher, recommended_watcher};

const TICK: Duration = Duration::from_millis(50);

pub fn watch(path: PathBuf, debounce: Duration, recheck: Duration) -> Receiver<Option<String>> {
    let (tx, rx) = async_channel::unbounded();
    thread::Builder::new()
        .name("palette-watch".into())
        .spawn(move || run(&path, debounce, recheck, &tx))
        .expect("spawn palette watcher");
    rx
}

fn read(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(err) if err.kind() == ErrorKind::NotFound => None,
        Err(err) => {
            tracing::debug!("cannot read {}: {err}", path.display());
            None
        }
    }
}

fn emit(tx: &Sender<Option<String>>, last: &mut Option<String>, next: Option<String>) -> bool {
    if *last == next {
        return true;
    }
    last.clone_from(&next);
    tx.send_blocking(next).is_ok()
}

fn run(path: &Path, debounce: Duration, recheck: Duration, tx: &Sender<Option<String>>) {
    let Some(parent) = path.parent().map(Path::to_path_buf) else {
        return;
    };
    let Some(name) = path.file_name().map(|n| n.to_os_string()) else {
        return;
    };
    let mut last = read(path);
    loop {
        while !parent.is_dir() {
            if tx.is_closed() {
                return;
            }
            thread::sleep(recheck);
        }
        if !emit(tx, &mut last, read(path)) {
            return;
        }
        let (etx, erx) = mpsc::channel::<notify::Result<Event>>();
        let Ok(mut watcher) = recommended_watcher(etx) else {
            thread::sleep(recheck);
            continue;
        };
        if watcher.watch(&parent, RecursiveMode::NonRecursive).is_err() {
            thread::sleep(recheck);
            continue;
        }
        let mut deadline: Option<Instant> = None;
        loop {
            match erx.recv_timeout(TICK) {
                Ok(Ok(event)) => {
                    if event
                        .paths
                        .iter()
                        .any(|p| p.file_name() == Some(name.as_os_str()))
                    {
                        deadline = Some(Instant::now() + debounce);
                    }
                }
                Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if tx.is_closed() {
                return;
            }
            if !parent.is_dir() {
                if !emit(tx, &mut last, None) {
                    return;
                }
                break;
            }
            if deadline.is_some_and(|d| Instant::now() >= d) {
                deadline = None;
                if !emit(tx, &mut last, read(path)) {
                    return;
                }
            }
        }
    }
}
