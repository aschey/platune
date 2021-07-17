use assert_matches::*;
use libplatune_management::{
    config::Config,
    database::{Database, SearchRes},
};
use rstest::*;
use std::fs::{self, create_dir, create_dir_all};
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_sync_empty() {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    create_dir(music_dir.clone()).unwrap();
    config.add_folder(music_dir.to_str().unwrap()).await;
    let mut receiver = config.sync().await;
    let mut msgs = vec![];
    while let Some(msg) = receiver.recv().await {
        msgs.push(msg);
    }
    db.close().await;
    assert_eq!(vec![1.], msgs);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
pub async fn test_sync_basic() {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    fs::copy(
        "../player/tests/assets/test.mp3",
        inner_dir.join("test.mp3"),
    )
    .unwrap();
    fs::copy(
        "../player/tests/assets/test2.mp3",
        inner_dir.join("test2.mp3"),
    )
    .unwrap();
    fs::copy(
        "../player/tests/assets/test3.mp3",
        inner_dir.join("test3.mp3"),
    )
    .unwrap();

    config.add_folder(music_dir.to_str().unwrap()).await;
    let mut receiver = config.sync().await;

    let mut msgs = vec![];
    while let Some(msg) = receiver.recv().await {
        msgs.push(msg);
    }

    db.close().await;
    assert_eq!(vec![0., 1.], msgs);
}

#[rstest(
    song1,
    song2,
    artist1,
    artist2,
    result1,
    result2,
    search,
    case(Some("asdf"), None, None, None, Some("asdf"), None, "asdf"),
    case(
        Some("bless"),
        Some("bliss"),
        None,
        None,
        Some("bless"),
        Some("bliss"),
        "blss"
    ),
    case(
        Some("bliss"),
        Some("bless blah blah"),
        None,
        None,
        Some("bliss"),
        Some("bless blah blah"),
        "blss"
    ),
    case(
        Some("bliss"),
        Some("blah bless blah"),
        None,
        None,
        Some("bliss"),
        Some("blah bless blah"),
        "blss"
    ),
    case(Some("bless"), Some("asdf"), None, None, Some("bless"), None, "blss"),
    case(
        None,
        None,
        Some("red hot chili peppers"),
        Some("real hearty chopped pies"),
        Some("red hot chili peppers"),
        Some("real hearty chopped pies"),
        "rhcp"
    ),
    case(Some("a/b"), None, None, None, Some("a/b"), None, "a b"),
    case(Some("a/b"), None, None, None, Some("a/b"), None, "a/b"),
    case(Some("a & b"), None, None, None, Some("a and b"), None, "a & b"),
    case(
        Some("a & b"),
        Some("a and b"),
        None,
        None,
        Some("a and b"),
        None,
        "a & b"
    ),
    case(
        Some("a & b"),
        Some("a and b"),
        None,
        None,
        Some("a and b"),
        None,
        "a and b"
    ),
    case(
        Some("a & b"),
        Some("a and b"),
        None,
        None,
        Some("a and b"),
        None,
        "a b"
    )
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_search(
    song1: Option<&str>,
    song2: Option<&str>,
    artist1: Option<&str>,
    artist2: Option<&str>,
    result1: Option<&str>,
    result2: Option<&str>,
    search: &str,
) {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    fs::copy(
        "../player/tests/assets/test.mp3",
        inner_dir.join("test.mp3"),
    )
    .unwrap();
    fs::copy(
        "../player/tests/assets/test2.mp3",
        inner_dir.join("test2.mp3"),
    )
    .unwrap();
    fs::copy(
        "../player/tests/assets/test3.mp3",
        inner_dir.join("test3.mp3"),
    )
    .unwrap();

    {
        let t1 = katatsuki::Track::from_path(&inner_dir.join("test.mp3"), None).unwrap();

        if let Some(song1) = song1 {
            t1.set_title(song1);
        }
        if let Some(artist1) = artist1 {
            t1.set_artist(artist1);
        }
        t1.save();

        let t2 = katatsuki::Track::from_path(&inner_dir.join("test2.mp3"), None).unwrap();
        if let Some(song2) = song2 {
            t2.set_title(song2);
        }
        if let Some(artist2) = artist2 {
            t2.set_artist(artist2);
        }
        t2.save();
    }

    config.add_folder(music_dir.to_str().unwrap()).await;
    let mut receiver = config.sync().await;

    while let Some(_) = receiver.recv().await {}

    let res = db.search(search, Default::default()).await;
    db.close().await;
    println!("{:?}", res);
    let result_len = vec![result1, result2]
        .iter()
        .filter(|r| r.is_some())
        .count();
    assert!(res.len() == result_len);

    if result_len > 0 {
        assert_matches!(&res[0], a if a.entry == result1.unwrap());
    }

    if result_len > 1 {
        assert_matches!(&res[1], a if a.entry == result2.unwrap());
    }
}

async fn setup(tempdir: &TempDir) -> (Database, Config) {
    let sql_path = tempdir.path().join("platune.db");
    let config_path = tempdir.path().join("platuneconfig");
    let db = Database::connect(sql_path, true).await;
    db.migrate().await;
    let config = Config::new_from_path(&db, config_path);
    (db, config)
}
