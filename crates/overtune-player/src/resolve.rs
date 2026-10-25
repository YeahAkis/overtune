use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use overtune_innertube::{AudioFormat, Innertube, pick_format};

const EXPIRY_MARGIN: Duration = Duration::from_secs(60);
const UNKNOWN_EXPIRY_TTL: Duration = Duration::from_secs(3600);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtractorKind {
    Innertube,
    NewPipe,
    YtDlp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedStream {
    pub url: String,
    pub format: AudioFormat,
    pub source: ExtractorKind,
    pub expires_at: Option<SystemTime>,
}

#[async_trait]
pub trait Extractor: Send + Sync {
    fn kind(&self) -> ExtractorKind;
    async fn resolve(&self, video_id: &str) -> Result<ResolvedStream>;
}

struct Cached {
    stream: ResolvedStream,
    stored: Instant,
}

impl Cached {
    fn fresh(&self) -> bool {
        match self.stream.expires_at {
            Some(at) => at
                .duration_since(SystemTime::now())
                .is_ok_and(|left| left > EXPIRY_MARGIN),
            None => self.stored.elapsed() < UNKNOWN_EXPIRY_TTL,
        }
    }
}

pub struct Resolver {
    order: Vec<Arc<dyn Extractor>>,
    cache: Mutex<HashMap<String, Cached>>,
}

impl Resolver {
    pub fn new(order: Vec<Arc<dyn Extractor>>) -> Resolver {
        Resolver {
            order,
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub async fn resolve(&self, video_id: &str) -> Result<ResolvedStream> {
        if let Some(hit) = self
            .cache
            .lock()
            .expect("resolver cache")
            .get(video_id)
            .filter(|c| c.fresh())
        {
            return Ok(hit.stream.clone());
        }
        let mut last = anyhow!("no extractor available for {video_id}");
        for extractor in &self.order {
            match extractor.resolve(video_id).await {
                Ok(stream) => {
                    self.cache.lock().expect("resolver cache").insert(
                        video_id.to_owned(),
                        Cached {
                            stream: stream.clone(),
                            stored: Instant::now(),
                        },
                    );
                    return Ok(stream);
                }
                Err(err) => {
                    tracing::debug!(
                        "{:?} could not resolve {video_id}: {err:#}",
                        extractor.kind()
                    );
                    last = err;
                }
            }
        }
        Err(last)
    }

    pub fn invalidate(&self, video_id: &str) {
        self.cache.lock().expect("resolver cache").remove(video_id);
    }
}

pub struct InnertubeExtractor(pub Innertube);

#[async_trait]
impl Extractor for InnertubeExtractor {
    fn kind(&self) -> ExtractorKind {
        ExtractorKind::Innertube
    }

    async fn resolve(&self, video_id: &str) -> Result<ResolvedStream> {
        let info = self.0.player(video_id).await?;
        let format = pick_format(&info.formats)
            .ok_or_else(|| anyhow!("no audio format for {video_id}"))?
            .clone();
        Ok(ResolvedStream {
            url: format.url.clone(),
            format,
            source: ExtractorKind::Innertube,
            expires_at: info.expires_at,
        })
    }
}
