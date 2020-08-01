CREATE VIRTUAL TABLE song_index USING fts5(song)

CREATE TRIGGER after_song_insert AFTER INSERT ON song BEGIN
    INSERT INTO song_index (
        rowid,
        song
    )
    VALUES(
        new.song_id,
        new.song_title
    );
    END;

CREATE TRIGGER after_song_update UPDATE OF song_title ON song BEGIN
    UPDATE song_index SET song_title = new.song_title WHERE rowid = old.song_id;
END;

CREATE TRIGGER after_song_delete AFTER DELETE ON song BEGIN
    DELETE FROM song_index WHERE rowid = old.song_id;
END;