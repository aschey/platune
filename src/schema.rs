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
    user_setting (setting_id) {
        setting_id -> Integer,
        setting_name -> Text,
        setting_value -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    folder,
    mount,
    user_setting,
);
