use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use async_channel::Receiver;
use gst::prelude::*;
use overtune_player::engine::{Engine, EngineEvent, PlayState};

fn make_opus(path: &Path, seconds: u32) {
    gst::init().unwrap();
    let buffers = seconds * 48_000 / 1024;
    let desc = format!(
        "audiotestsrc num-buffers={buffers} samplesperbuffer=1024 ! audio/x-raw,rate=48000 ! audioconvert ! opusenc ! oggmux ! filesink location={}",
        path.display()
    );
    let pipeline = gst::parse::launch(&desc).unwrap();
    pipeline.set_state(gst::State::Playing).unwrap();
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::from_seconds(20)) {
        match msg.view() {
            gst::MessageView::Eos(..) => break,
            gst::MessageView::Error(e) => panic!("{}", e.error()),
            _ => {}
        }
    }
    pipeline.set_state(gst::State::Null).unwrap();
}

fn wait_for(
    rx: &Receiver<EngineEvent>,
    within: Duration,
    mut f: impl FnMut(&EngineEvent) -> bool,
) -> Vec<EngineEvent> {
    let end = Instant::now() + within;
    let mut seen = Vec::new();
    while Instant::now() < end {
        match rx.try_recv() {
            Ok(ev) => {
                let done = f(&ev);
                seen.push(ev);
                if done {
                    return seen;
                }
            }
            Err(_) => thread::sleep(Duration::from_millis(10)),
        }
    }
    panic!("condition not met, saw {seen:?}");
}

fn uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

#[test]
fn plays_local_file_and_reports_position() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tone.opus");
    make_opus(&file, 3);
    let (engine, rx) = Engine::with_audio_sink("fakesink");
    engine.load(&uri(&file), 0, true);
    let seen = wait_for(&rx, Duration::from_secs(8), |e| {
        matches!(e, EngineEvent::EndOfStream)
    });
    assert!(
        seen.iter()
            .any(|e| matches!(e, EngineEvent::State(PlayState::Playing)))
    );
    let positions: Vec<u64> = seen
        .iter()
        .filter_map(|e| match e {
            EngineEvent::Position { position_ms, .. } => Some(*position_ms),
            _ => None,
        })
        .collect();
    assert!(positions.len() >= 3, "{positions:?}");
    assert!(positions.windows(2).all(|w| w[0] <= w[1]), "{positions:?}");
    assert!(seen.iter().any(|e| matches!(e, EngineEvent::Position { duration_ms: Some(d), .. } if d.abs_diff(3000) < 300)));
}

#[test]
fn load_paused_at_offset() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tone.opus");
    make_opus(&file, 4);
    let (engine, rx) = Engine::with_audio_sink("fakesink");
    engine.load(&uri(&file), 1500, false);
    wait_for(&rx, Duration::from_secs(5), |e| {
        matches!(e, EngineEvent::State(PlayState::Paused))
    });
    thread::sleep(Duration::from_millis(300));
    let pos = engine.position_ms().unwrap();
    assert!((1300..=1700).contains(&pos), "{pos}");
    thread::sleep(Duration::from_millis(500));
    assert!(
        engine.position_ms().unwrap().abs_diff(pos) < 50,
        "should stay paused"
    );
}

#[test]
fn pause_and_resume() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tone.opus");
    make_opus(&file, 4);
    let (engine, rx) = Engine::with_audio_sink("fakesink");
    engine.load(&uri(&file), 0, true);
    wait_for(&rx, Duration::from_secs(5), |e| {
        matches!(e, EngineEvent::State(PlayState::Playing))
    });
    engine.pause();
    wait_for(&rx, Duration::from_secs(5), |e| {
        matches!(e, EngineEvent::State(PlayState::Paused))
    });
    engine.play();
    wait_for(&rx, Duration::from_secs(5), |e| {
        matches!(e, EngineEvent::State(PlayState::Playing))
    });
}

#[test]
fn seek_moves_position() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tone.opus");
    make_opus(&file, 5);
    let (engine, rx) = Engine::with_audio_sink("fakesink");
    engine.load(&uri(&file), 0, false);
    wait_for(&rx, Duration::from_secs(5), |e| {
        matches!(e, EngineEvent::State(PlayState::Paused))
    });
    engine.seek(3000);
    thread::sleep(Duration::from_millis(400));
    assert!(engine.position_ms().unwrap().abs_diff(3000) < 200);
}

#[test]
fn bad_uri_gives_stream_error() {
    let (engine, rx) = Engine::with_audio_sink("fakesink");
    engine.load("file:///definitely/not/here.opus", 0, true);
    wait_for(&rx, Duration::from_secs(5), |e| {
        matches!(e, EngineEvent::StreamError { .. })
    });
}
