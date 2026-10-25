use rusqlite::{OptionalExtension, params};

use crate::db::{Library, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SongRecord {
    pub id: String,
    pub title: String,
    pub artists_text: Option<String>,
    pub duration_text: Option<String>,
    pub thumbnail_url: Option<String>,
    pub liked_at: Option<i64>,
    pub total_play_time_ms: i64,
    pub blacklisted: bool,
    pub explicit: bool,
}

impl Library {
    pub fn upsert_song(&self, song: &SongRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO Song (id, title, artistsText, durationText, thumbnailUrl, likedAt, totalPlayTimeMs, blacklisted, explicit)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
               title = excluded.title,
               artistsText = COALESCE(excluded.artistsText, artistsText),
               durationText = COALESCE(excluded.durationText, durationText),
               thumbnailUrl = COALESCE(excluded.thumbnailUrl, thumbnailUrl),
               explicit = excluded.explicit",
            params![
                song.id,
                song.title,
                song.artists_text,
                song.duration_text,
                song.thumbnail_url,
                song.liked_at,
                song.total_play_time_ms,
                song.blacklisted,
                song.explicit
            ],
        )?;
        Ok(())
    }

    pub fn song(&self, id: &str) -> Result<Option<SongRecord>> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, title, artistsText, durationText, thumbnailUrl, likedAt, totalPlayTimeMs, blacklisted, explicit
                 FROM Song WHERE id = ?1",
                [id],
                |r| {
                    Ok(SongRecord {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        artists_text: r.get(2)?,
                        duration_text: r.get(3)?,
                        thumbnail_url: r.get(4)?,
                        liked_at: r.get(5)?,
                        total_play_time_ms: r.get(6)?,
                        blacklisted: r.get(7)?,
                        explicit: r.get(8)?,
                    })
                },
            )
            .optional()?)
    }
}
