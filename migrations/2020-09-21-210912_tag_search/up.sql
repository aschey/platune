CREATE TRIGGER after_tag_insert AFTER INSERT ON tag BEGIN
    INSERT INTO search_index (
        assoc_id,
        entry_value,
        entry_type
    )
    VALUES(
        new.tag_id,
        new.tag_name,
        'tag'
    );
END;

CREATE TRIGGER after_tag_update UPDATE OF tag_name ON tag BEGIN
    UPDATE search_index SET entry_value = new.tag_name WHERE assoc_id = old.tag_id and entry_type = 'tag';
END;

CREATE TRIGGER after_tag_delete AFTER DELETE ON tag BEGIN
    DELETE FROM search_index WHERE assoc_id = old.tag_id and entry_type = 'tag'; 
END;