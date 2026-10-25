use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use overtune_innertube::AudioFormat;
use overtune_player::resolve::{Extractor, ExtractorKind, ResolvedStream, Resolver};
use overtune_player::ytdlp::parse_formats;

struct Fake {
    kind: ExtractorKind,
    fail: bool,
    calls: AtomicUsize,
    expires_in: Option<Duration>,
}

impl Fake {
    fn new(kind: ExtractorKind, fail: bool, expires_in: Option<Duration>) -> Arc<Fake> {
        Arc::new(Fake {
            kind,
            fail,
            calls: AtomicUsize::new(0),
            expires_in,
        })
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl Extractor for Fake {
    fn kind(&self) -> ExtractorKind {
        self.kind
    }

    async fn resolve(&self, video_id: &str) -> Result<ResolvedStream> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail {
            return Err(anyhow!("{:?} failed for {video_id}", self.kind));
        }
        let format = AudioFormat {
            itag: 251,
            url: format!("https://x/{video_id}"),
            mime: "audio/webm".into(),
            bitrate: 1,
            sample_rate: None,
            content_length: None,
            loudness_db: None,
        };
        Ok(ResolvedStream {
            url: format.url.clone(),
            format,
            source: self.kind,
            expires_at: self.expires_in.map(|d| SystemTime::now() + d),
        })
    }
}

const HOUR: Option<Duration> = Some(Duration::from_secs(3600));

#[tokio::test]
async fn falls_through_on_error() {
    let a = Fake::new(ExtractorKind::Innertube, true, HOUR);
    let b = Fake::new(ExtractorKind::YtDlp, false, HOUR);
    let r = Resolver::new(vec![a.clone(), b.clone()]);
    let s = r.resolve("abc").await.unwrap();
    assert_eq!(s.source, ExtractorKind::YtDlp);
    assert_eq!((a.calls(), b.calls()), (1, 1));
}

#[tokio::test]
async fn all_fail_returns_last_error() {
    let a = Fake::new(ExtractorKind::Innertube, true, HOUR);
    let b = Fake::new(ExtractorKind::YtDlp, true, HOUR);
    let r = Resolver::new(vec![a, b]);
    let err = r.resolve("abc").await.unwrap_err().to_string();
    assert!(err.contains("YtDlp"), "{err}");
}

#[tokio::test]
async fn empty_chain_is_error() {
    assert!(Resolver::new(vec![]).resolve("abc").await.is_err());
}

#[tokio::test]
async fn cached_until_near_expiry() {
    let a = Fake::new(ExtractorKind::Innertube, false, HOUR);
    let r = Resolver::new(vec![a.clone()]);
    r.resolve("abc").await.unwrap();
    r.resolve("abc").await.unwrap();
    assert_eq!(a.calls(), 1);
    let soon = Fake::new(
        ExtractorKind::Innertube,
        false,
        Some(Duration::from_secs(30)),
    );
    let r = Resolver::new(vec![soon.clone()]);
    r.resolve("abc").await.unwrap();
    r.resolve("abc").await.unwrap();
    assert_eq!(soon.calls(), 2);
}

#[tokio::test]
async fn cache_is_per_video() {
    let a = Fake::new(ExtractorKind::Innertube, false, HOUR);
    let r = Resolver::new(vec![a.clone()]);
    r.resolve("abc").await.unwrap();
    r.resolve("def").await.unwrap();
    assert_eq!(a.calls(), 2);
}

#[tokio::test]
async fn invalidate_forces_refetch() {
    let a = Fake::new(ExtractorKind::Innertube, false, HOUR);
    let r = Resolver::new(vec![a.clone()]);
    r.resolve("abc").await.unwrap();
    r.invalidate("abc");
    r.resolve("abc").await.unwrap();
    assert_eq!(a.calls(), 2);
}

#[test]
fn ytdlp_json_parses() {
    let path = format!(
        "{}/tests/fixtures/ytdlp_blinding.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    let formats = parse_formats(&json);
    let itags: Vec<u32> = formats.iter().map(|f| f.itag).collect();
    assert_eq!(itags, vec![139, 249, 250, 140, 251]);
    let opus = formats.iter().find(|f| f.itag == 251).unwrap();
    assert_eq!(opus.mime, "audio/webm");
    assert_eq!(opus.bitrate, 132_978);
    assert_eq!(opus.sample_rate, Some(48_000));
    assert_eq!(opus.content_length, Some(3_350_747));
    assert_eq!(
        formats.iter().find(|f| f.itag == 140).unwrap().mime,
        "audio/mp4"
    );
}
