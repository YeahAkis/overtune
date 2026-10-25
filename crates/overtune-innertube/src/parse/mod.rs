pub mod list_item;
pub mod player;
pub mod search;
pub mod thumbnail;

use serde_json::Value;

use crate::models::Suggestions;

const SUGGESTED_SONGS: usize = 3;

pub fn at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(value, |v, key| match v {
        Value::Array(items) => key.parse::<usize>().ok().and_then(|i| items.get(i)),
        Value::Object(map) => map.get(*key),
        _ => None,
    })
}

pub fn str_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    at(value, path).and_then(Value::as_str)
}

pub fn runs(value: &Value) -> &[Value] {
    value
        .get("runs")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

pub fn text(value: &Value) -> Option<String> {
    if let Some(s) = value.get("simpleText").and_then(Value::as_str) {
        return Some(s.to_owned());
    }
    let joined: String = runs(value)
        .iter()
        .filter_map(|r| r.get("text").and_then(Value::as_str))
        .collect();
    (!joined.is_empty()).then_some(joined)
}

pub fn array<'a>(value: &'a Value, path: &[&str]) -> &'a [Value] {
    at(value, path)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

pub fn parse_duration(text: &str) -> Option<u64> {
    let parts: Vec<&str> = text.trim().split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return None;
    }
    let mut total = 0u64;
    for part in parts {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        total = total * 60 + part.parse::<u64>().ok()?;
    }
    Some(total * 1000)
}

pub fn suggestions(response: &Value) -> Suggestions {
    let mut out = Suggestions::default();
    for section in array(response, &["contents"]) {
        for item in array(section, &["searchSuggestionsSectionRenderer", "contents"]) {
            if let Some(query) = str_at(
                item,
                &[
                    "searchSuggestionRenderer",
                    "navigationEndpoint",
                    "searchEndpoint",
                    "query",
                ],
            ) {
                out.queries.push(query.to_owned());
            } else if let Some(renderer) = item.get("musicResponsiveListItemRenderer")
                && out.songs.len() < SUGGESTED_SONGS
                && list_item::is_song(renderer)
                && let Some(song) = list_item::song(renderer)
            {
                out.songs.push(song);
            }
        }
    }
    out
}
