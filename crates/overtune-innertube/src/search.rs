use percent_encoding::percent_decode_str;
use serde_json::{Map, Value};

use crate::client::Innertube;
use crate::context::ClientContext;
use crate::error::Result;
use crate::parse;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchKind {
    Songs,
    Artists,
    Albums,
    Playlists,
}

impl SearchKind {
    pub const ALL: [SearchKind; 4] = [
        SearchKind::Songs,
        SearchKind::Artists,
        SearchKind::Albums,
        SearchKind::Playlists,
    ];

    fn params(self) -> String {
        let encoded = match self {
            SearchKind::Songs => "EgWKAQIIAWoOEAMQBBAJEAoQBRAQEBU%3D",
            SearchKind::Albums => "EgWKAQIYAWoOEAMQBBAJEAoQBRAQEBU%3D",
            SearchKind::Artists => "EgWKAQIgAWoOEAMQBBAJEAoQBRAQEBU%3D",
            SearchKind::Playlists => "EgeKAQQoAEABag4QAxAEEAkQChAFEBAQFQ%3D%3D",
        };
        percent_decode_str(encoded).decode_utf8_lossy().into_owned()
    }
}

impl Innertube {
    pub async fn raw_search(&self, query: &str, kind: SearchKind) -> Result<Value> {
        let mut body = Map::new();
        body.insert("query".into(), Value::from(query));
        body.insert("params".into(), Value::from(kind.params()));
        self.post(
            &Innertube::music_url("search"),
            &ClientContext::WEB_REMIX,
            body,
        )
        .await
    }

    pub async fn raw_search_more(&self, continuation: &str) -> Result<Value> {
        let mut body = Map::new();
        body.insert("continuation".into(), Value::from(continuation));
        self.post(
            &Innertube::music_url("search"),
            &ClientContext::WEB_REMIX,
            body,
        )
        .await
    }

    pub async fn search(&self, query: &str, kind: SearchKind) -> Result<parse::search::SearchPage> {
        Ok(parse::search::page(
            &self.raw_search(query, kind).await?,
            kind,
        ))
    }

    pub async fn search_more(
        &self,
        kind: SearchKind,
        continuation: &str,
    ) -> Result<parse::search::SearchPage> {
        Ok(parse::search::page(
            &self.raw_search_more(continuation).await?,
            kind,
        ))
    }
}
