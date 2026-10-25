use rand::RngExt;
use serde_json::{Map, Value};

use crate::client::Innertube;
use crate::context::ClientContext;
use crate::error::Result;
use crate::parse;

pub use crate::parse::player::{AudioFormat, PlayerInfo, pick_format};

const PLAYER: &str = "https://youtubei.googleapis.com/youtubei/v1/player";
const NONCE_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn nonce(len: usize) -> String {
    let mut rng = rand::rng();
    (0..len)
        .map(|_| NONCE_ALPHABET[rng.random_range(0..NONCE_ALPHABET.len())] as char)
        .collect()
}

impl Innertube {
    pub async fn raw_player(&self, video_id: &str) -> Result<Value> {
        let mut body = Map::new();
        body.insert("videoId".into(), Value::from(video_id));
        body.insert("cpn".into(), Value::from(nonce(16)));
        body.insert("contentCheckOk".into(), Value::from("true"));
        body.insert("racyCheckOn".into(), Value::from("true"));
        let t = nonce(12);
        self.post_with_query(
            PLAYER,
            &ClientContext::VISIONOS,
            body,
            &[("t", t.as_str()), ("id", video_id)],
        )
        .await
    }

    async fn try_player(&self, video_id: &str) -> Result<PlayerInfo> {
        parse::player::player_info(&self.raw_player(video_id).await?)
    }

    pub async fn player(&self, video_id: &str) -> Result<PlayerInfo> {
        if self.visitor_data().await.is_err() {
            tracing::debug!("visitor data unavailable, using default");
        }
        match self.try_player(video_id).await {
            Ok(info) => Ok(info),
            Err(first) => {
                tracing::debug!(
                    "player failed for {video_id}: {first}, retrying with fresh visitor data"
                );
                if self.refresh_visitor_data().await.is_err() {
                    return Err(first);
                }
                self.try_player(video_id).await
            }
        }
    }
}
