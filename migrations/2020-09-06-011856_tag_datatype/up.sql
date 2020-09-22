CREATE TABLE IF NOT EXISTS tag_datatype (
    tag_datatype_id INTEGER PRIMARY KEY NOT NULL,
    tag_datatype_name TEXT NOT NULL UNIQUE
);

INSERT OR IGNORE INTO tag_datatype(tag_datatype_name)
VALUES('text');

INSERT OR IGNORE INTO tag_datatype(tag_datatype_name)
VALUES('numeric');

INSERT OR IGNORE INTO tag_datatype(tag_datatype_name)
VALUES('date');