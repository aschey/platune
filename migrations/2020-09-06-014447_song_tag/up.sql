CREATE TABLE IF NOT EXISTS song_tag (
    song_tag_id INTEGER PRIMARY KEY NOT NULL,
    song_id INTEGER NOT NULL,
    tag_id INTEGER NULL,
    FOREIGN KEY(song_id) REFERENCES song(song_id),
    FOREIGN KEY(tag_id) REFERENCES tag(tag_id)
)