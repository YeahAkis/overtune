use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use url::Url;

use crate::error::{Error, Result};
use crate::parse::{array, at, str_at};

const PREFERRED_ITAGS: [u32; 2] = [251, 140];

#[derive(Debug, Clone, PartialEq)]
pub struct AudioFormat {
    pub itag: u32,
    pub url: String,
    pub mime: String,
    pub bitrate: u32,
    pub sample_rate: Option<u32>,
    pub content_length: Option<u64>,
    pub loudness_db: Option<f32>,
}

impl AudioFormat {
    pub fn is_audio(&self) -> bool {
        self.mime.starts_with("audio/")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerInfo {
    pub video_id: String,
    pub title: String,
    pub duration_ms: u64,
    pub formats: Vec<AudioFormat>,
    pub expires_at: Option<SystemTime>,
}

fn number(value: &Value, key: &str) -> Option<u64> {
    let v = value.get(key)?;
    v.as_u64().or_else(|| v.as_str()?.parse().ok())
}

fn format(value: &Value, fallback_loudness: Option<f32>) -> Option<AudioFormat> {
    let mime = value.get("mimeType")?.as_str()?;
    if !mime.starts_with("audio/") {
        return None;
    }
    Some(AudioFormat {
        itag: number(value, "itag")? as u32,
        url: value.get("url")?.as_str()?.to_owned(),
        mime: mime.to_owned(),
        bitrate: number(value, "bitrate").unwrap_or(0) as u32,
        sample_rate: number(value, "audioSampleRate").map(|v| v as u32),
        content_length: number(value, "contentLength"),
        loudness_db: value
            .get("loudnessDb")
            .and_then(Value::as_f64)
            .map(|v| v as f32)
            .or(fallback_loudness),
    })
}

pub fn player_info(response: &Value) -> Result<PlayerInfo> {
    let status = str_at(response, &["playabilityStatus", "status"]).unwrap_or("UNKNOWN");
    if status != "OK" {
        let reason = str_at(response, &["playabilityStatus", "reason"]).unwrap_or(status);
        return Err(Error::Unplayable(reason.to_owned()));
    }
    let loudness = at(response, &["playerConfig", "audioConfig", "loudnessDb"])
        .and_then(Value::as_f64)
        .map(|v| v as f32);
    let formats: Vec<AudioFormat> = array(response, &["streamingData", "adaptiveFormats"])
        .iter()
        .filter_map(|f| format(f, loudness))
        .collect();
    if formats.is_empty() {
        return Err(Error::Unplayable("no direct audio urls".into()));
    }
    let details = response
        .get("videoDetails")
        .ok_or(Error::Parse("videoDetails missing"))?;
    let expires_at = formats.iter().find_map(|f| url_expiry(&f.url));
    Ok(PlayerInfo {
        video_id: str_at(details, &["videoId"])
            .ok_or(Error::Parse("videoId missing"))?
            .to_owned(),
        title: str_at(details, &["title"]).unwrap_or_default().to_owned(),
        duration_ms: number(details, "lengthSeconds").unwrap_or(0) * 1000,
        formats,
        expires_at,
    })
}

pub fn pick_format(formats: &[AudioFormat]) -> Option<&AudioFormat> {
    let audio = || formats.iter().filter(|f| f.is_audio());
    PREFERRED_ITAGS
        .iter()
        .find_map(|itag| audio().find(|f| f.itag == *itag))
        .or_else(|| audio().max_by_key(|f| f.bitrate))
}

pub fn url_expiry(url: &str) -> Option<SystemTime> {
    let parsed = Url::parse(url).ok()?;
    let secs: u64 = parsed
        .query_pairs()
        .find(|(k, _)| k == "expire")?
        .1
        .parse()
        .ok()?;
    Some(UNIX_EPOCH + Duration::from_secs(secs))
}
