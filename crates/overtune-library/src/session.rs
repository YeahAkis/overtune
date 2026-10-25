use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::db::{Library, Result};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Repeat {
    #[default]
    Off,
    All,
    One,
}

impl Repeat {
    pub fn as_str(self) -> &'static str {
        match self {
            Repeat::Off => "off",
            Repeat::All => "all",
            Repeat::One => "one",
        }
    }

    pub fn parse(s: &str) -> Repeat {
        match s {
            "all" => Repeat::All,
            "one" => Repeat::One,
            _ => Repeat::Off,
        }
    }

    pub fn cycle(self) -> Repeat {
        match self {
            Repeat::Off => Repeat::All,
            Repeat::All => Repeat::One,
            Repeat::One => Repeat::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RightPanel {
    #[default]
    Hidden,
    NowPlaying,
    Queue,
}

impl RightPanel {
    pub fn as_str(self) -> &'static str {
        match self {
            RightPanel::Hidden => "hidden",
            RightPanel::NowPlaying => "now_playing",
            RightPanel::Queue => "queue",
        }
    }

    pub fn parse(s: &str) -> RightPanel {
        match s {
            "now_playing" => RightPanel::NowPlaying,
            "queue" => RightPanel::Queue,
            _ => RightPanel::Hidden,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub queue_json: String,
    pub current_index: Option<usize>,
    pub position_ms: u64,
    pub shuffle: bool,
    pub repeat: Repeat,
    pub volume: f64,
    pub right_panel: RightPanel,
}

impl Library {
    pub fn save_session(&self, s: &Session) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO ot_session (id, queue_json, current_index, position_ms, shuffle, repeat, volume, right_panel)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                s.queue_json,
                s.current_index.map(|i| i as i64),
                s.position_ms as i64,
                s.shuffle,
                s.repeat.as_str(),
                s.volume,
                s.right_panel.as_str()
            ],
        )?;
        Ok(())
    }

    pub fn load_session(&self) -> Result<Option<Session>> {
        Ok(self
            .conn
            .query_row(
                "SELECT queue_json, current_index, position_ms, shuffle, repeat, volume, right_panel FROM ot_session WHERE id = 1",
                [],
                |r| {
                    Ok(Session {
                        queue_json: r.get(0)?,
                        current_index: r.get::<_, Option<i64>>(1)?.map(|i| i as usize),
                        position_ms: r.get::<_, i64>(2)? as u64,
                        shuffle: r.get(3)?,
                        repeat: Repeat::parse(&r.get::<_, String>(4)?),
                        volume: r.get(5)?,
                        right_panel: RightPanel::parse(&r.get::<_, String>(6)?),
                    })
                },
            )
            .optional()?)
    }
}
