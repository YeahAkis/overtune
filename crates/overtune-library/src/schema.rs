pub const VITUNE_TABLES: &[&str] = &[
    r#"CREATE TABLE IF NOT EXISTS `Song` (`id` TEXT NOT NULL, `title` TEXT NOT NULL, `artistsText` TEXT, `durationText` TEXT, `thumbnailUrl` TEXT, `likedAt` INTEGER, `totalPlayTimeMs` INTEGER NOT NULL, `loudnessBoost` REAL, `blacklisted` INTEGER NOT NULL DEFAULT false, `explicit` INTEGER NOT NULL DEFAULT false, PRIMARY KEY(`id`))"#,
    r#"CREATE TABLE IF NOT EXISTS `Artist` (`id` TEXT NOT NULL, `name` TEXT, `thumbnailUrl` TEXT, `timestamp` INTEGER, `bookmarkedAt` INTEGER, PRIMARY KEY(`id`))"#,
    r#"CREATE TABLE IF NOT EXISTS `Album` (`id` TEXT NOT NULL, `title` TEXT, `description` TEXT, `thumbnailUrl` TEXT, `year` TEXT, `authorsText` TEXT, `shareUrl` TEXT, `timestamp` INTEGER, `bookmarkedAt` INTEGER, `otherInfo` TEXT, PRIMARY KEY(`id`))"#,
    r#"CREATE TABLE IF NOT EXISTS `Playlist` (`id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, `name` TEXT NOT NULL, `browseId` TEXT, `thumbnail` TEXT)"#,
    r#"CREATE TABLE IF NOT EXISTS `SongPlaylistMap` (`songId` TEXT NOT NULL, `playlistId` INTEGER NOT NULL, `position` INTEGER NOT NULL, PRIMARY KEY(`songId`, `playlistId`), FOREIGN KEY(`songId`) REFERENCES `Song`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE , FOREIGN KEY(`playlistId`) REFERENCES `Playlist`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE )"#,
    r#"CREATE INDEX IF NOT EXISTS `index_SongPlaylistMap_songId` ON `SongPlaylistMap` (`songId`)"#,
    r#"CREATE INDEX IF NOT EXISTS `index_SongPlaylistMap_playlistId` ON `SongPlaylistMap` (`playlistId`)"#,
    r#"CREATE TABLE IF NOT EXISTS `SongArtistMap` (`songId` TEXT NOT NULL, `artistId` TEXT NOT NULL, PRIMARY KEY(`songId`, `artistId`), FOREIGN KEY(`songId`) REFERENCES `Song`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE , FOREIGN KEY(`artistId`) REFERENCES `Artist`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE )"#,
    r#"CREATE INDEX IF NOT EXISTS `index_SongArtistMap_songId` ON `SongArtistMap` (`songId`)"#,
    r#"CREATE INDEX IF NOT EXISTS `index_SongArtistMap_artistId` ON `SongArtistMap` (`artistId`)"#,
    r#"CREATE TABLE IF NOT EXISTS `SongAlbumMap` (`songId` TEXT NOT NULL, `albumId` TEXT NOT NULL, `position` INTEGER, PRIMARY KEY(`songId`, `albumId`), FOREIGN KEY(`songId`) REFERENCES `Song`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE , FOREIGN KEY(`albumId`) REFERENCES `Album`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE )"#,
    r#"CREATE INDEX IF NOT EXISTS `index_SongAlbumMap_songId` ON `SongAlbumMap` (`songId`)"#,
    r#"CREATE INDEX IF NOT EXISTS `index_SongAlbumMap_albumId` ON `SongAlbumMap` (`albumId`)"#,
    r#"CREATE TABLE IF NOT EXISTS `Event` (`id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, `songId` TEXT NOT NULL, `timestamp` INTEGER NOT NULL, `playTime` INTEGER NOT NULL, FOREIGN KEY(`songId`) REFERENCES `Song`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE )"#,
    r#"CREATE INDEX IF NOT EXISTS `index_Event_songId` ON `Event` (`songId`)"#,
    r#"CREATE TABLE IF NOT EXISTS `Lyrics` (`songId` TEXT NOT NULL, `fixed` TEXT, `synced` TEXT, `wordSynced` TEXT, `wordSyncedChecked` INTEGER, `startTime` INTEGER, PRIMARY KEY(`songId`), FOREIGN KEY(`songId`) REFERENCES `Song`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE )"#,
    r#"CREATE TABLE IF NOT EXISTS `Format` (`songId` TEXT NOT NULL, `itag` INTEGER, `mimeType` TEXT, `bitrate` INTEGER, `contentLength` INTEGER, `lastModified` INTEGER, `loudnessDb` REAL, PRIMARY KEY(`songId`), FOREIGN KEY(`songId`) REFERENCES `Song`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE )"#,
    r#"CREATE TABLE IF NOT EXISTS `SearchQuery` (`id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, `query` TEXT NOT NULL)"#,
    r#"CREATE UNIQUE INDEX IF NOT EXISTS `index_SearchQuery_query` ON `SearchQuery` (`query`)"#,
    r#"CREATE VIEW IF NOT EXISTS `SortedSongPlaylistMap` AS SELECT * FROM SongPlaylistMap ORDER BY position"#,
];

pub const OVERTUNE_TABLES: &[&str] = &[
    r#"CREATE TABLE IF NOT EXISTS `ot_session` (`id` INTEGER PRIMARY KEY CHECK (`id` = 1), `queue_json` TEXT NOT NULL, `current_index` INTEGER, `position_ms` INTEGER NOT NULL, `shuffle` INTEGER NOT NULL, `repeat` TEXT NOT NULL, `volume` REAL NOT NULL, `right_panel` TEXT NOT NULL)"#,
    r#"CREATE TABLE IF NOT EXISTS `ot_meta` (`key` TEXT PRIMARY KEY NOT NULL, `value` TEXT NOT NULL)"#,
];
