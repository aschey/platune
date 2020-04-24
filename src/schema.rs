table! {
    album (album_id) {
        album_id -> Integer,
        album_name -> Text,
        is_compilation -> Bool,
        release_date -> Integer,
    }
}

table! {
    artist (artist_id) {
        artist_id -> Integer,
        artist_name -> Text,
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
        modified_date -> Integer,
        artist_id -> Integer,
        compilation_artist_id -> Integer,
        song_title -> Text,
        album_id -> Integer,
        play_count -> Integer,
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

joinable!(song -> album (album_id));

allow_tables_to_appear_in_same_query!(
    album,
    artist,
    folder,
    mount,
    song,
    user_setting,
);
