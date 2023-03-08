CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
    entry_value,
    entry_type unindexed,
    assoc_id unindexed,
    tokenize = 'unicode61 remove_diacritics 2'
);
CREATE VIRTUAL TABLE IF NOT EXISTS search_vocab USING fts5vocab(search_index, row);
CREATE VIRTUAL TABLE IF NOT EXISTS search_spellfix USING spellfix1;
-- Song
CREATE TRIGGER IF NOT EXISTS after_song_insert
AFTER
INSERT ON song BEGIN
INSERT INTO search_index (
        assoc_id,
        entry_value,
        entry_type
    )
VALUES(
        new.song_id,
        REPLACE(new.song_title, ' & ', ' and '),
        'song'
    );
END;
CREATE TRIGGER IF NOT EXISTS after_song_update
UPDATE OF song_title ON song BEGIN
UPDATE search_index
SET entry_value = REPLACE(new.song_title, ' & ', ' and ')
WHERE assoc_id = old.song_id
    AND entry_type = 'song';
END;
CREATE TRIGGER IF NOT EXISTS after_song_delete
AFTER DELETE ON song BEGIN
DELETE FROM search_index
WHERE assoc_id = old.song_id
    AND entry_type = 'song';
END;
-- Album
CREATE TRIGGER IF NOT EXISTS after_album_insert
AFTER
INSERT ON album BEGIN
INSERT INTO search_index (
        assoc_id,
        entry_value,
        entry_type
    )
VALUES(
        new.album_id,
        REPLACE(new.album_name, ' & ', ' and '),
        'album'
    );
END;
CREATE TRIGGER IF NOT EXISTS after_album_update
UPDATE OF album_name ON album BEGIN
UPDATE search_index
SET entry_value = REPLACE(new.album_name, ' & ', ' and ')
WHERE assoc_id = old.album_id
    AND entry_type = 'album';
END;
CREATE TRIGGER IF NOT EXISTS after_album_delete
AFTER DELETE ON album BEGIN
DELETE FROM search_index
WHERE assoc_id = old.album_id
    AND entry_type = 'album';
END;
-- Artist
CREATE TRIGGER IF NOT EXISTS after_artist_insert
AFTER
INSERT ON artist BEGIN
INSERT INTO search_index (
        assoc_id,
        entry_value,
        entry_type
    )
VALUES(
        new.artist_id,
        REPLACE(new.artist_name, ' & ', ' and '),
        'artist'
    );
END;
CREATE TRIGGER IF NOT EXISTS after_artist_update
UPDATE OF artist_name ON artist BEGIN
UPDATE search_index
SET entry_value = REPLACE(new.artist_name, ' & ', ' and ')
WHERE assoc_id = old.artist_id
    and entry_type = 'artist';
END;
CREATE TRIGGER IF NOT EXISTS after_artist_delete
AFTER DELETE ON artist BEGIN
DELETE FROM search_index
WHERE assoc_id = old.artist_id
    AND entry_type = 'artist';
END;
-- Album Artist
CREATE TRIGGER IF NOT EXISTS after_album_artist_insert
AFTER
INSERT ON album_artist BEGIN
INSERT INTO search_index (
        assoc_id,
        entry_value,
        entry_type
    )
VALUES(
        new.album_artist_id,
        REPLACE(new.album_artist_name, ' & ', ' and '),
        'album_artist'
    );
END;
CREATE TRIGGER IF NOT EXISTS after_album_artist_update
UPDATE OF album_artist_name ON album_artist BEGIN
UPDATE search_index
SET entry_value = REPLACE(new.album_artist_name, ' & ', ' and ')
WHERE assoc_id = old.album_artist_id
    AND entry_type = 'album_artist';
END;
CREATE TRIGGER IF NOT EXISTS after_album_artist_delete
AFTER DELETE ON album_artist BEGIN
DELETE FROM search_index
WHERE assoc_id = old.album_artist_id
    AND entry_type = 'album_artist';
END;