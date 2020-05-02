table! {
    album (album_id) {
        album_id -> Integer,
        album_name -> Text,
        album_year -> Integer,
        album_month -> Integer,
        album_day -> Integer,
        album_artist_id -> Integer,
    }
}

table! {
    artist (artist_id) {
        artist_id -> Integer,
        artist_name -> Text,
    }
}

table! {
    file_size (file_size_id) {
        file_size_id -> Integer,
        song_id -> Integer,
        song_file_size -> Integer,
        file_modified_date -> Integer,
    }
}

table! {
    folder (folder_id) {
        folder_id -> Integer,
        full_path_unix -> Text,
        full_path_windows -> Text,
    }
}

table! {
    import_temp (import_id) {
        import_id -> Integer,
        import_song_path_windows -> Text,
        import_song_path_unix -> Text,
        import_artist -> Text,
        import_album_artist -> Text,
        import_title -> Text,
        import_album -> Text,
        import_track_number -> Integer,
        import_disc_number -> Integer,
        import_year -> Integer,
    }
}

table! {
    mount (mount_id) {
        mount_id -> Integer,
        unix_path -> Text,
        windows_path -> Text,
    }
}

table! {
    song (song_id) {
        song_id -> Integer,
        song_path_unix -> Text,
        song_path_windows -> Text,
        metadata_modified_date -> Integer,
        artist_id -> Integer,
        song_title -> Text,
        album_id -> Integer,
        track_number -> Integer,
        play_count -> Integer,
        disc_number -> Integer,
        song_year -> Integer,
        song_month -> Integer,
        song_day -> Integer,
        is_deleted -> Bool,
    }
}

table! {
    user_setting (setting_id) {
        setting_id -> Integer,
        setting_name -> Text,
        setting_value -> Text,
    }
}

joinable!(album -> artist (album_artist_id));
joinable!(file_size -> song (song_id));
joinable!(song -> album (album_id));
joinable!(song -> artist (artist_id));

allow_tables_to_appear_in_same_query!(
    album,
    artist,
    file_size,
    folder,
    import_temp,
    mount,
    song,
    user_setting,
);
