use std::fs;

use overtune_innertube::Innertube;

fn save(name: &str, value: &serde_json::Value) {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    fs::write(path, serde_json::to_string(value).unwrap()).unwrap();
}

#[tokio::test]
#[ignore]
async fn record_suggestions() {
    let it = Innertube::new("en", "US");
    save(
        "suggestions_blinding.json",
        &it.raw_search_suggestions("blinding").await.unwrap(),
    );
}

#[tokio::test]
#[ignore]
async fn live_suggestions() {
    let it = Innertube::new("en", "US");
    let s = it.search_suggestions("blinding").await.unwrap();
    assert!(!s.queries.is_empty());
    assert!(!s.songs.is_empty());
}

#[tokio::test]
#[ignore]
async fn live_visitor_data() {
    let it = Innertube::new("en", "US");
    assert!(it.visitor_data().await.unwrap().len() > 10);
}

#[tokio::test]
#[ignore]
async fn record_search() {
    use overtune_innertube::SearchKind;
    let it = Innertube::new("en", "US");
    let songs = it.raw_search("blinding", SearchKind::Songs).await.unwrap();
    save("search_songs_blinding.json", &songs);
    let cont = find_continuation(&songs).expect("continuation");
    save(
        "search_songs_blinding_continuation.json",
        &it.raw_search_more(&cont).await.unwrap(),
    );
    save(
        "search_artists_weeknd.json",
        &it.raw_search("the weeknd", SearchKind::Artists)
            .await
            .unwrap(),
    );
    save(
        "search_albums_after_hours.json",
        &it.raw_search("after hours", SearchKind::Albums)
            .await
            .unwrap(),
    );
    save(
        "search_playlists_synthwave.json",
        &it.raw_search("synthwave", SearchKind::Playlists)
            .await
            .unwrap(),
    );
    save(
        "search_songs_jvke.json",
        &it.raw_search(
            "this is what falling in love feels like jvke",
            SearchKind::Songs,
        )
        .await
        .unwrap(),
    );
}

fn find_continuation(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::Object(m) => {
            if let Some(c) = m
                .get("nextContinuationData")
                .and_then(|n| n.get("continuation"))
                .and_then(|c| c.as_str())
            {
                return Some(c.to_owned());
            }
            if let Some(c) = m
                .get("continuationCommand")
                .and_then(|n| n.get("token"))
                .and_then(|c| c.as_str())
            {
                return Some(c.to_owned());
            }
            m.values().find_map(find_continuation)
        }
        serde_json::Value::Array(a) => a.iter().find_map(find_continuation),
        _ => None,
    }
}

#[tokio::test]
#[ignore]
async fn live_search_all_kinds() {
    use overtune_innertube::SearchKind;
    let it = Innertube::new("en", "US");
    for kind in SearchKind::ALL {
        let page = it.search("the weeknd", kind).await.unwrap();
        assert!(!page.results.is_empty(), "{kind:?}");
    }
}

#[tokio::test]
#[ignore]
async fn live_player_returns_playable_url() {
    let it = Innertube::new("en", "US");
    let info = it.player("J7p4bzqLvCw").await.unwrap();
    let format = overtune_innertube::pick_format(&info.formats).unwrap();
    let response = reqwest::Client::new()
        .get(&format.url)
        .header("range", "bytes=0-1023")
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "{}", response.status());
}
