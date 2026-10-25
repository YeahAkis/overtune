use overtune_player::{Extractor, ExtractorKind, YtDlpExtractor};

#[tokio::test]
#[ignore]
async fn live_ytdlp_resolves() {
    let x = YtDlpExtractor::find().expect("yt-dlp on PATH");
    let s = x.resolve("J7p4bzqLvCw").await.unwrap();
    assert_eq!(s.source, ExtractorKind::YtDlp);
    assert_eq!(s.format.itag, 251);
}
