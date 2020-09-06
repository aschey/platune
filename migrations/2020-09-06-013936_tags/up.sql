CREATE TABLE IF NOT EXISTS tag (
    tag_id INTEGER PRIMARY KEY NOT NULL,
    tag_name TEXT NOT NULL UNIQUE,
    tag_color TEXT NOT NULL,
    tag_datatype_id INTEGER NOT NULL,
    tag_priority INTEGER NOT NULL,
    FOREIGN KEY(tag_datatype_id) REFERENCES tag_datatype(tag_datatype_id)
)