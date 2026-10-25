use std::env;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use overtune_innertube::parse::player::url_expiry;
use overtune_innertube::{AudioFormat, pick_format};
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;

use crate::resolve::{Extractor, ExtractorKind, ResolvedStream};

const TIMEOUT: Duration = Duration::from_secs(20);

pub struct YtDlpExtractor {
    binary: PathBuf,
}

impl YtDlpExtractor {
    pub fn find() -> Option<YtDlpExtractor> {
        let path = env::var_os("PATH")?;
        env::split_paths(&path)
            .map(|dir| dir.join("yt-dlp"))
            .find(|p| p.is_file())
            .map(|binary| YtDlpExtractor { binary })
    }
}

fn mime(ext: &str) -> String {
    match ext {
        "m4a" | "mp4" => "audio/mp4".into(),
        other => format!("audio/{other}"),
    }
}

pub fn parse_formats(json: &Value) -> Vec<AudioFormat> {
    json.get("formats")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(|f| {
            let acodec = f.get("acodec")?.as_str()?;
            if acodec == "none" || f.get("vcodec").and_then(Value::as_str) != Some("none") {
                return None;
            }
            if f.get("protocol").and_then(Value::as_str) != Some("https") {
                return None;
            }
            let id = f.get("format_id")?.as_str()?;
            let digits: String = id.chars().take_while(char::is_ascii_digit).collect();
            Some(AudioFormat {
                itag: digits.parse().ok()?,
                url: f.get("url")?.as_str()?.to_owned(),
                mime: mime(f.get("ext").and_then(Value::as_str).unwrap_or("webm")),
                bitrate: f
                    .get("abr")
                    .and_then(Value::as_f64)
                    .map_or(0, |kbps| (kbps * 1000.0).round() as u32),
                sample_rate: f.get("asr").and_then(Value::as_u64).map(|v| v as u32),
                content_length: f.get("filesize").and_then(Value::as_u64),
                loudness_db: None,
            })
        })
        .collect()
}

#[async_trait]
impl Extractor for YtDlpExtractor {
    fn kind(&self) -> ExtractorKind {
        ExtractorKind::YtDlp
    }

    async fn resolve(&self, video_id: &str) -> Result<ResolvedStream> {
        let url = format!("https://music.youtube.com/watch?v={video_id}");
        let output = timeout(
            TIMEOUT,
            Command::new(&self.binary)
                .args(["-J", "--no-warnings", "--no-playlist", &url])
                .kill_on_drop(true)
                .output(),
        )
        .await
        .context("yt-dlp timed out")??;
        if !output.status.success() {
            return Err(anyhow!(
                "yt-dlp failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let json: Value = serde_json::from_slice(&output.stdout)?;
        let formats = parse_formats(&json);
        let format = pick_format(&formats)
            .ok_or_else(|| anyhow!("yt-dlp returned no audio for {video_id}"))?
            .clone();
        Ok(ResolvedStream {
            url: format.url.clone(),
            expires_at: url_expiry(&format.url),
            format,
            source: ExtractorKind::YtDlp,
        })
    }
}
