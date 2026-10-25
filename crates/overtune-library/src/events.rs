use rusqlite::{Error as SqlError, params};

use crate::db::{Library, Result};
use crate::stats::play_threshold_ms;

impl Library {
    pub fn record_listen(
        &self,
        song_id: &str,
        listened_ms: u64,
        duration_ms: Option<u64>,
        at_ms: i64,
    ) -> Result<bool> {
        let tx = self.conn.unchecked_transaction()?;
        let updated = tx.execute(
            "UPDATE Song SET totalPlayTimeMs = totalPlayTimeMs + ?2 WHERE id = ?1",
            params![song_id, listened_ms as i64],
        )?;
        if updated == 0 {
            return Err(SqlError::QueryReturnedNoRows.into());
        }
        let counted = listened_ms > 0 && listened_ms >= play_threshold_ms(duration_ms);
        if counted {
            tx.execute(
                "INSERT INTO Event (songId, timestamp, playTime) VALUES (?1, ?2, ?3)",
                params![song_id, at_ms, listened_ms as i64],
            )?;
        }
        tx.commit()?;
        Ok(counted)
    }

    pub fn event_count(&self, song_id: &str) -> Result<u64> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) FROM Event WHERE songId = ?1",
            [song_id],
            |r| r.get::<_, i64>(0),
        )? as u64)
    }

    pub fn events(&self, song_id: &str) -> Result<Vec<(i64, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, playTime FROM Event WHERE songId = ?1 ORDER BY timestamp",
        )?;
        let rows = stmt.query_map([song_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
