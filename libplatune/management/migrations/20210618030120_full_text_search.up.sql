CREATE VIRTUAL TABLE search_index USING fts5(
    entry_value,
    entry_type unindexed,
    assoc_id unindexed,
    tokenize = 'unicode61 remove_diacritics 2'
);
CREATE VIRTUAL TABLE search_vocab USING fts5vocab(search_index, row);
CREATE VIRTUAL TABLE search_spellfix USING spellfix1;
-- Song
CREATE TRIGGER after_song_insert
AFTER
INSERT ON song BEGIN
INSERT INTO search_index (
        assoc_id,
        entry_value,
        entry_type
    )
VALUES(
        new.song_id,
        new.song_title,
        'song'
    );
END;
CREATE TRIGGER after_song_update
UPDATE OF song_title ON song BEGIN
UPDATE search_index
SET entry_value = new.song_title
WHERE assoc_id = old.song_id
    and entry_type = 'song';
END;
CREATE TRIGGER after_song_delete
AFTER DELETE ON song BEGIN
DELETE FROM search_index
WHERE assoc_id = old.song_id
    and entry_type = 'song';
END;
-- Album
CREATE TRIGGER after_album_insert
AFTER
INSERT ON album BEGIN
INSERT INTO search_index (
        assoc_id,
        entry_value,
        entry_type
    )
VALUES(
        new.album_id,
        new.album_name,
        'album'
    );
END;
CREATE TRIGGER after_album_update
UPDATE OF album_name ON album BEGIN
UPDATE search_index
SET entry_value = new.album_name
WHERE assoc_id = old.album_id
    and entry_type = 'album';
END;
CREATE TRIGGER after_album_delete
AFTER DELETE ON album BEGIN
DELETE FROM search_index
WHERE assoc_id = old.album_id
    and entry_type = 'album';
END;
-- Artist
CREATE TRIGGER after_artist_insert
AFTER
INSERT ON artist BEGIN
INSERT INTO search_index (
        assoc_id,
        entry_value,
        entry_type
    )
VALUES(
        new.artist_id,
        new.artist_name,
        'artist'
    );
END;
CREATE TRIGGER after_artist_update
UPDATE OF artist_name ON artist BEGIN
UPDATE search_index
SET entry_value = new.artist_name
WHERE assoc_id = old.artist_id
    and entry_type = 'artist';
END;
CREATE TRIGGER after_artist_delete
AFTER DELETE ON artist BEGIN
DELETE FROM search_index
WHERE assoc_id = old.artist_id
    and entry_type = 'artist';
END;
-- Album Artist
CREATE TRIGGER after_album_artist_insert
AFTER
INSERT ON album_artist BEGIN
INSERT INTO search_index (
        assoc_id,
        entry_value,
        entry_type
    )
VALUES(
        new.album_artist_id,
        new.album_artist_name,
        'album_artist'
    );
END;
CREATE TRIGGER after_album_artist_update
UPDATE OF album_artist_name ON album_artist BEGIN
UPDATE search_index
SET entry_value = new.album_artist_name
WHERE assoc_id = old.album_artist_id
    and entry_type = 'album_artist';
END;
CREATE TRIGGER after_album_artist_delete
AFTER DELETE ON album_artist BEGIN
DELETE FROM search_index
WHERE assoc_id = old.album_artist_id
    and entry_type = 'album_artist';
END;