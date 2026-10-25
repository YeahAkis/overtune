use overtune_player::{Extractor, ExtractorKind, YtDlpExtractor};

#[tokio::test]
#[ignore]
async fn live_ytdlp_resolves() {
    let x = YtDlpExtractor::find().expect("yt-dlp on PATH");
    let s = x.resolve("J7p4bzqLvCw").await.unwrap();
    assert_eq!(s.source, ExtractorKind::YtDlp);
    assert_eq!(s.format.itag, 251);
}

#[tokio::test]
#[ignore]
async fn live_engine_streams_youtube_audio() {
    use std::time::{Duration, Instant};

    use overtune_innertube::Innertube;
    use overtune_player::{Engine, EngineEvent, InnertubeExtractor};

    let stream = InnertubeExtractor(Innertube::new("en", "US"))
        .resolve("J7p4bzqLvCw")
        .await
        .unwrap();
    let (engine, rx) = Engine::with_audio_sink("fakesink");
    engine.load(&stream.url, 60_000, true);
    let end = Instant::now() + Duration::from_secs(15);
    let mut best = 0;
    while Instant::now() < end && best < 62_000 {
        match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Ok(EngineEvent::Position { position_ms, .. })) => best = best.max(position_ms),
            Ok(Ok(EngineEvent::StreamError { message, .. })) => panic!("{message}"),
            _ => {}
        }
    }
    assert!(best >= 62_000, "only reached {best}");
}
