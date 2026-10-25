use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use async_channel::{Receiver, Sender};
use gst::prelude::*;
use gst::{ClockTime, Element, ElementFactory, MessageView, SeekFlags, State};
use gst_audio::prelude::*;
use gst_audio::{StreamVolume, StreamVolumeFormat};

const TICK: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Stopped,
    Loading,
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    State(PlayState),
    Position {
        position_ms: u64,
        duration_ms: Option<u64>,
    },
    EndOfStream,
    StreamError {
        position_ms: u64,
        message: String,
    },
}

#[derive(Debug, Default)]
struct Pending {
    seek_ms: Option<u64>,
    autoplay: bool,
    active: bool,
}

struct Shared {
    playbin: Element,
    tx: Sender<EngineEvent>,
    pending: Mutex<Pending>,
    last_position: AtomicU64,
    stop: AtomicBool,
}

pub struct Engine {
    shared: Arc<Shared>,
}

impl Engine {
    pub fn new() -> (Engine, Receiver<EngineEvent>) {
        Engine::build(None)
    }

    pub fn with_audio_sink(factory: &str) -> (Engine, Receiver<EngineEvent>) {
        Engine::build(Some(factory))
    }

    fn build(sink: Option<&str>) -> (Engine, Receiver<EngineEvent>) {
        gst::init().expect("gstreamer init");
        let playbin = ElementFactory::make("playbin3")
            .build()
            .or_else(|_| ElementFactory::make("playbin").build())
            .expect("playbin");
        playbin.set_property_from_str("flags", "audio+soft-volume");
        if let Some(factory) = sink {
            let element = ElementFactory::make(factory)
                .property("sync", true)
                .build()
                .expect("audio sink");
            playbin.set_property("audio-sink", &element);
        }
        let (tx, rx) = async_channel::unbounded();
        let shared = Arc::new(Shared {
            playbin,
            tx,
            pending: Mutex::new(Pending::default()),
            last_position: AtomicU64::new(0),
            stop: AtomicBool::new(false),
        });
        let worker = shared.clone();
        thread::Builder::new()
            .name("engine-bus".into())
            .spawn(move || run_bus(&worker))
            .expect("spawn engine bus thread");
        (Engine { shared }, rx)
    }

    pub fn load(&self, uri: &str, start_ms: u64, autoplay: bool) {
        let s = &self.shared;
        let _ = s.playbin.set_state(State::Null);
        s.last_position.store(start_ms, Ordering::SeqCst);
        *s.pending.lock().expect("pending") = Pending {
            seek_ms: (start_ms > 0).then_some(start_ms),
            autoplay,
            active: true,
        };
        s.playbin.set_property("uri", uri);
        let _ = s.tx.send_blocking(EngineEvent::State(PlayState::Loading));
        if s.playbin.set_state(State::Paused).is_err() {
            let _ = s.tx.send_blocking(EngineEvent::StreamError {
                position_ms: start_ms,
                message: format!("cannot open {uri}"),
            });
        }
    }

    pub fn play(&self) {
        let s = &self.shared;
        let mut pending = s.pending.lock().expect("pending");
        if pending.active {
            pending.autoplay = true;
            return;
        }
        drop(pending);
        let _ = s.playbin.set_state(State::Playing);
    }

    pub fn pause(&self) {
        let s = &self.shared;
        let mut pending = s.pending.lock().expect("pending");
        if pending.active {
            pending.autoplay = false;
            return;
        }
        drop(pending);
        let _ = s.playbin.set_state(State::Paused);
    }

    pub fn seek(&self, ms: u64) {
        let s = &self.shared;
        let mut pending = s.pending.lock().expect("pending");
        if pending.active {
            pending.seek_ms = Some(ms);
            return;
        }
        drop(pending);
        s.last_position.store(ms, Ordering::SeqCst);
        let _ = s.playbin.seek_simple(
            SeekFlags::FLUSH | SeekFlags::ACCURATE,
            ClockTime::from_mseconds(ms),
        );
    }

    pub fn set_volume(&self, volume: f64) {
        if let Some(sv) = self.shared.playbin.dynamic_cast_ref::<StreamVolume>() {
            sv.set_volume(StreamVolumeFormat::Cubic, volume.clamp(0.0, 1.0));
        }
    }

    pub fn position_ms(&self) -> Option<u64> {
        self.shared
            .playbin
            .query_position::<ClockTime>()
            .map(ClockTime::mseconds)
    }

    pub fn duration_ms(&self) -> Option<u64> {
        self.shared
            .playbin
            .query_duration::<ClockTime>()
            .map(ClockTime::mseconds)
    }

    pub fn stop(&self) {
        let s = &self.shared;
        *s.pending.lock().expect("pending") = Pending::default();
        let _ = s.playbin.set_state(State::Null);
        let _ = s.tx.send_blocking(EngineEvent::State(PlayState::Stopped));
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        let _ = self.shared.playbin.set_state(State::Null);
    }
}

fn run_bus(s: &Shared) {
    let Some(bus) = s.playbin.bus() else {
        return;
    };
    let mut last_tick = Instant::now();
    let mut playing = false;
    while !s.stop.load(Ordering::SeqCst) && !s.tx.is_closed() {
        if let Some(msg) = bus.timed_pop(ClockTime::from_mseconds(50)) {
            match msg.view() {
                MessageView::AsyncDone(..) => on_async_done(s),
                MessageView::StateChanged(change) if msg.src() == Some(s.playbin.upcast_ref()) => {
                    let pending = s.pending.lock().expect("pending").active;
                    match change.current() {
                        State::Playing if !pending => {
                            playing = true;
                            let _ = s.tx.send_blocking(EngineEvent::State(PlayState::Playing));
                        }
                        State::Paused if !pending && change.old() == State::Playing => {
                            playing = false;
                            let _ = s.tx.send_blocking(EngineEvent::State(PlayState::Paused));
                        }
                        State::Null | State::Ready => playing = false,
                        _ => {}
                    }
                }
                MessageView::Eos(..) => {
                    playing = false;
                    send_position(s);
                    let _ = s.tx.send_blocking(EngineEvent::EndOfStream);
                }
                MessageView::Error(err) => {
                    playing = false;
                    *s.pending.lock().expect("pending") = Pending::default();
                    let _ = s.playbin.set_state(State::Null);
                    let _ = s.tx.send_blocking(EngineEvent::StreamError {
                        position_ms: s.last_position.load(Ordering::SeqCst),
                        message: err.error().to_string(),
                    });
                }
                _ => {}
            }
        }
        if playing && last_tick.elapsed() >= TICK {
            last_tick = Instant::now();
            send_position(s);
        }
    }
}

fn on_async_done(s: &Shared) {
    let mut pending = s.pending.lock().expect("pending");
    if !pending.active {
        return;
    }
    if let Some(ms) = pending.seek_ms.take() {
        drop(pending);
        let _ = s.playbin.seek_simple(
            SeekFlags::FLUSH | SeekFlags::ACCURATE,
            ClockTime::from_mseconds(ms),
        );
        return;
    }
    let autoplay = pending.autoplay;
    *pending = Pending::default();
    drop(pending);
    send_position(s);
    if autoplay {
        let _ = s.playbin.set_state(State::Playing);
    } else {
        let _ = s.tx.send_blocking(EngineEvent::State(PlayState::Paused));
    }
}

fn send_position(s: &Shared) {
    let Some(position) = s.playbin.query_position::<ClockTime>() else {
        return;
    };
    let position_ms = position.mseconds();
    s.last_position.store(position_ms, Ordering::SeqCst);
    let duration_ms = s
        .playbin
        .query_duration::<ClockTime>()
        .map(ClockTime::mseconds);
    let _ = s.tx.send_blocking(EngineEvent::Position {
        position_ms,
        duration_ms,
    });
}
