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
