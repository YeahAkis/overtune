use serde_json::{Map, Value};

use crate::client::Innertube;
use crate::context::ClientContext;
use crate::error::Result;
use crate::models::Suggestions;
use crate::parse;

impl Innertube {
    pub async fn raw_search_suggestions(&self, input: &str) -> Result<Value> {
        let mut body = Map::new();
        body.insert("input".into(), Value::from(input));
        self.post(
            &Innertube::music_url("music/get_search_suggestions"),
            &ClientContext::WEB_REMIX,
            body,
        )
        .await
    }

    pub async fn search_suggestions(&self, input: &str) -> Result<Suggestions> {
        Ok(parse::suggestions(
            &self.raw_search_suggestions(input).await?,
        ))
    }
}
