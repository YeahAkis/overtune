use overtune_library::Repeat;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueItem {
    pub video_id: String,
    pub title: String,
    pub artists: String,
    pub duration_ms: Option<u64>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Queue {
    items: Vec<QueueItem>,
    order: Vec<usize>,
    index: Option<usize>,
    shuffle: bool,
    repeat: Repeat,
}

impl Queue {
    pub fn new() -> Queue {
        Queue::default()
    }

    pub fn set(&mut self, items: Vec<QueueItem>, start: usize) {
        self.items = items;
        self.order = (0..self.items.len()).collect();
        self.index = if self.items.is_empty() {
            None
        } else {
            Some(start.min(self.items.len() - 1))
        };
        if self.shuffle {
            self.reshuffle();
        }
    }

    pub fn current(&self) -> Option<&QueueItem> {
        self.index.map(|i| &self.items[self.order[i]])
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> Vec<&QueueItem> {
        self.order.iter().map(|&i| &self.items[i]).collect()
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn repeat(&self) -> Repeat {
        self.repeat
    }

    pub fn set_repeat(&mut self, repeat: Repeat) {
        self.repeat = repeat;
    }

    pub fn advance(&mut self) -> Option<&QueueItem> {
        if self.repeat == Repeat::One {
            return self.current();
        }
        self.skip()
    }

    pub fn skip(&mut self) -> Option<&QueueItem> {
        let i = self.index?;
        if i + 1 < self.order.len() {
            self.index = Some(i + 1);
        } else if self.repeat != Repeat::Off {
            self.index = Some(0);
        } else {
            return None;
        }
        self.current()
    }

    pub fn previous(&mut self) -> Option<&QueueItem> {
        let i = self.index?;
        if i > 0 {
            self.index = Some(i - 1);
        } else if self.repeat != Repeat::Off {
            self.index = Some(self.order.len() - 1);
        } else {
            return None;
        }
        self.current()
    }

    pub fn jump(&mut self, position: usize) -> Option<&QueueItem> {
        if position < self.order.len() {
            self.index = Some(position);
        }
        self.current()
    }

    pub fn push_back(&mut self, item: QueueItem) {
        self.items.push(item);
        self.order.push(self.items.len() - 1);
        if self.index.is_none() {
            self.index = Some(0);
        }
    }

    pub fn insert_next(&mut self, item: QueueItem) {
        self.items.push(item);
        let new = self.items.len() - 1;
        match self.index {
            Some(i) => self.order.insert(i + 1, new),
            None => {
                self.order.push(new);
                self.index = Some(0);
            }
        }
    }

    pub fn set_shuffle(&mut self, shuffle: bool) {
        if shuffle == self.shuffle {
            return;
        }
        self.shuffle = shuffle;
        if shuffle {
            self.reshuffle();
        } else {
            let current = self.index.map(|i| self.order[i]);
            self.order = (0..self.items.len()).collect();
            self.index = current;
        }
    }

    fn reshuffle(&mut self) {
        let Some(i) = self.index else {
            self.order.shuffle(&mut rand::rng());
            return;
        };
        let current = self.order.remove(i);
        self.order.shuffle(&mut rand::rng());
        self.order.insert(0, current);
        self.index = Some(0);
    }
}
