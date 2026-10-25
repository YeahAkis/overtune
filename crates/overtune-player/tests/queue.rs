use overtune_library::Repeat;
use overtune_player::queue::{Queue, QueueItem};

fn item(id: &str) -> QueueItem {
    QueueItem {
        video_id: id.into(),
        title: id.to_uppercase(),
        artists: "x".into(),
        duration_ms: Some(1000),
        thumbnail: None,
    }
}

fn ids(q: &Queue) -> Vec<String> {
    q.items().iter().map(|i| i.video_id.clone()).collect()
}

fn current(q: &Queue) -> Option<String> {
    q.current().map(|i| i.video_id.clone())
}

fn abcde() -> Vec<QueueItem> {
    ["a", "b", "c", "d", "e"].iter().map(|s| item(s)).collect()
}

#[test]
fn empty_queue_has_nothing() {
    let mut q = Queue::new();
    assert!(q.current().is_none());
    assert!(q.advance().is_none());
    assert!(q.previous().is_none());
    assert_eq!(q.index(), None);
}

#[test]
fn set_starts_at_index() {
    let mut q = Queue::new();
    q.set(abcde(), 2);
    assert_eq!(current(&q).as_deref(), Some("c"));
    assert_eq!(q.index(), Some(2));
}

#[test]
fn set_clamps_start() {
    let mut q = Queue::new();
    q.set(abcde(), 99);
    assert_eq!(current(&q).as_deref(), Some("e"));
}

#[test]
fn next_and_previous_stop_at_edges_without_repeat() {
    let mut q = Queue::new();
    q.set(abcde(), 4);
    assert!(q.advance().is_none());
    assert_eq!(current(&q).as_deref(), Some("e"));
    q.set(abcde(), 0);
    assert!(q.previous().is_none());
    assert_eq!(current(&q).as_deref(), Some("a"));
    assert_eq!(q.advance().unwrap().video_id, "b");
    assert_eq!(q.previous().unwrap().video_id, "a");
}

#[test]
fn repeat_all_wraps() {
    let mut q = Queue::new();
    q.set(abcde(), 4);
    q.set_repeat(Repeat::All);
    assert_eq!(q.advance().unwrap().video_id, "a");
    assert_eq!(q.previous().unwrap().video_id, "e");
}

#[test]
fn repeat_one_stays() {
    let mut q = Queue::new();
    q.set(abcde(), 1);
    q.set_repeat(Repeat::One);
    assert_eq!(q.advance().unwrap().video_id, "b");
    assert_eq!(q.advance().unwrap().video_id, "b");
}

#[test]
fn skip_ignores_repeat_one() {
    let mut q = Queue::new();
    q.set(abcde(), 1);
    q.set_repeat(Repeat::One);
    assert_eq!(q.skip().unwrap().video_id, "c");
}

#[test]
fn shuffle_keeps_current_first_and_restores_order() {
    let mut q = Queue::new();
    q.set(abcde(), 2);
    q.set_shuffle(true);
    assert!(q.shuffle());
    assert_eq!(q.index(), Some(0));
    assert_eq!(current(&q).as_deref(), Some("c"));
    let mut shuffled = ids(&q);
    shuffled.sort();
    assert_eq!(shuffled, vec!["a", "b", "c", "d", "e"]);
    q.advance();
    let now = current(&q).unwrap();
    q.set_shuffle(false);
    assert_eq!(ids(&q), vec!["a", "b", "c", "d", "e"]);
    assert_eq!(current(&q).unwrap(), now);
}

#[test]
fn set_while_shuffled_starts_on_chosen_item() {
    let mut q = Queue::new();
    q.set_shuffle(true);
    q.set(abcde(), 3);
    assert_eq!(current(&q).as_deref(), Some("d"));
    assert_eq!(q.index(), Some(0));
}

#[test]
fn insert_next_lands_after_current() {
    let mut q = Queue::new();
    q.set(abcde(), 1);
    q.insert_next(item("z"));
    assert_eq!(ids(&q), vec!["a", "b", "z", "c", "d", "e"]);
    assert_eq!(q.advance().unwrap().video_id, "z");
}

#[test]
fn push_back_appends() {
    let mut q = Queue::new();
    q.push_back(item("a"));
    assert_eq!(current(&q).as_deref(), Some("a"));
    q.push_back(item("b"));
    assert_eq!(ids(&q), vec!["a", "b"]);
}

#[test]
fn queue_items_roundtrip_json() {
    let items = abcde();
    let json = serde_json::to_string(&items).unwrap();
    let back: Vec<QueueItem> = serde_json::from_str(&json).unwrap();
    assert_eq!(back, items);
}
