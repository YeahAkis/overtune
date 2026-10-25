use serde_json::Value;

use crate::models::Thumbnail;
use crate::parse::{array, at};

pub fn thumbnails(value: &Value) -> Vec<Thumbnail> {
    let candidates = [
        &["musicThumbnailRenderer", "thumbnail", "thumbnails"][..],
        &[
            "thumbnail",
            "musicThumbnailRenderer",
            "thumbnail",
            "thumbnails",
        ][..],
        &["croppedSquareThumbnailRenderer", "thumbnail", "thumbnails"][..],
        &["thumbnails"][..],
    ];
    let list = candidates
        .iter()
        .map(|p| array(value, p))
        .find(|l| !l.is_empty())
        .unwrap_or_default();
    list.iter()
        .filter_map(|t| {
            Some(Thumbnail {
                url: at(t, &["url"])?.as_str()?.to_owned(),
                width: at(t, &["width"]).and_then(Value::as_u64).unwrap_or(0) as u32,
                height: at(t, &["height"]).and_then(Value::as_u64).unwrap_or(0) as u32,
            })
        })
        .collect()
}

pub fn best_thumbnail(list: &[Thumbnail], min: u32) -> Option<&Thumbnail> {
    list.iter()
        .filter(|t| t.width >= min)
        .min_by_key(|t| t.width)
        .or_else(|| list.iter().max_by_key(|t| t.width))
}

pub fn resize_thumbnail(url: &str, size: u32) -> String {
    let Some(eq) = url.rfind("=w") else {
        return url.to_owned();
    };
    let rest = &url[eq + 2..];
    let w_end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    if w_end == 0 || !rest[w_end..].starts_with("-h") {
        return url.to_owned();
    }
    let after_h = &rest[w_end + 2..];
    let h_end = after_h
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after_h.len());
    if h_end == 0 {
        return url.to_owned();
    }
    format!("{}=w{size}-h{size}{}", &url[..eq], &after_h[h_end..])
}
