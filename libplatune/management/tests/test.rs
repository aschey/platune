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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_sync_no_folder() {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;

    let mut receiver = config.sync().await;
    let mut msgs = vec![];
    while let Some(msg) = receiver.recv().await {
        msgs.push(msg);
    }
    db.close().await;
    assert_eq!(Vec::<f32>::new(), msgs);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
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

pub struct SongTest {
    title: Option<&'static str>,
    artist: Option<&'static str>,
}

impl Default for SongTest {
    fn default() -> Self {
        Self {
            title: None,
            artist: None,
        }
    }
}

#[rstest(
    songs,
    results,
    search,
    case(vec![
        SongTest {
            title: Some("asdf"), 
            ..Default::default()
        }],
        vec!["asdf"],
        "asdf"),
    case(vec![
        SongTest {
            title: Some("bless"),
            ..Default::default()
        },
        SongTest {
            title: Some("bliss"),
            ..Default::default()
        }],
        vec!["bless", "bliss"], 
        "blss"),
    case(vec![
        SongTest {
            title: Some("bless blah blah"),
            ..Default::default()
        },
        SongTest {
            title: Some("bliss"),
            ..Default::default()
        }],
        vec!["bliss", "bless blah blah"], 
        "blss"),
    case(vec![
        SongTest {
            title: Some("blah bless blah"),
            ..Default::default()
        },
        SongTest {
            title: Some("bliss"),
            ..Default::default()
        }],
        vec!["bliss", "blah bless blah"], 
        "blss"),
    case(vec![
        SongTest {
            title: Some("bless"),
            ..Default::default()
        },
        SongTest {
            title: Some("asdf"),
            ..Default::default()
        }],
        vec!["bless"], 
        "blss"),
    case(vec![
        SongTest {
            artist: Some("red hot chili peppers"),
            ..Default::default()
        },
        SongTest {
            artist: Some("real hearty chopped pies"),
            ..Default::default()
        }],
        vec!["red hot chili peppers", "real hearty chopped pies"], 
        "rhcp"),
    case(vec![
        SongTest {
            artist: Some("red hot chili peppers"),
            ..Default::default()
        },
        SongTest {
            artist: Some("rhcpa"),
            ..Default::default()
        }],
        vec!["red hot chili peppers", "rhcpa"], 
        "rhcp"),
    case(vec![
        SongTest {
            artist: Some("a/b"),
            ..Default::default()
        }],
        vec!["a/b"], 
        "a b"),
    case(vec![
        SongTest {
            artist: Some("a/b"),
            ..Default::default()
        }],
        vec!["a/b"], 
        "a/b"),
    case(vec![
        SongTest {
            artist: Some("a & b"),
            ..Default::default()
        }],
        vec!["a and b"], 
        "a & b"),
    case(vec![
        SongTest {
            artist: Some("a & b"),
            ..Default::default()
        }],
        vec!["a and b"], 
        "a and b"),
    case(vec![
        SongTest {
            artist: Some("a & b"),
            ..Default::default()
        },
        SongTest {
            artist: Some("a and b"),
            ..Default::default()
        }],
        vec!["a and b"], 
        "a & b"),
    case(vec![
        SongTest {
            artist: Some("a & b"),
            ..Default::default()
        },
        SongTest {
            artist: Some("a and b"),
            ..Default::default()
        }],
        vec!["a and b"], 
        "a and b"),
    case(vec![
        SongTest {
            artist: Some("a & b"),
            ..Default::default()
        },
        SongTest {
            artist: Some("a and b"),
            ..Default::default()
        }],
        vec!["a and b"], 
        "a b"),
    case(vec![
        SongTest {
            artist: Some("bad bad"),
            ..Default::default()
        },
        SongTest {
            artist: Some("bag"),
            ..Default::default()
        }],
        vec!["bag", "bad bad"], 
        "bag"),
    case(vec![
        SongTest {
            artist: Some("bad bad"),
            ..Default::default()
        },
        SongTest {
            artist: Some("bad bag"),
            ..Default::default()
        },
        SongTest {
            artist: Some("bag"),
            ..Default::default()
        }],
        vec!["bag", "bad bag", "bad bad"], 
        "bag"),
    case(vec![
        SongTest {
            artist: Some("qwerty"),
            title: Some("untitled")
        },
        SongTest {
            artist: Some("qwerty"),
            title: Some("untitled 2")
        },
        SongTest {
            artist: Some("bag"),
            title: Some("untitled")
        }],
        vec!["untitled", "untitled 2"], 
        "untitled artist:qwerty"),
    case(vec![
        SongTest {
            artist: Some("red hot chili peppers"),
            title: Some("untitled")
        },
        SongTest {
            artist: Some("red hot chili peppers"),
            title: Some("untitled 2")
        },
        SongTest {
            artist: Some("bag"),
            title: Some("untitled")
        }],
        vec!["untitled", "untitled 2"], 
        "untitled artist:rhcp")
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_search(songs: Vec<SongTest>, results: Vec<&str>, search: &str) {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    for (i, song) in songs.iter().enumerate() {
        let song_path = inner_dir.join(format!("test{}.mp3", i));
        fs::copy("../player/tests/assets/test.mp3", song_path.clone()).unwrap();
        let t = katatsuki::Track::from_path(&song_path, None).unwrap();

        if let Some(title) = song.title {
            t.set_title(title);
        }
        if let Some(artist) = song.artist {
            t.set_artist(artist);
        }
        t.save();
    }

    config.add_folder(music_dir.to_str().unwrap()).await;
    let mut receiver = config.sync().await;

    while let Some(_) = receiver.recv().await {}

    let res = db.search(search, Default::default()).await;
    db.close().await;
    println!("{:?}", res);

    assert!(res.len() == results.len());

    for (i, result) in results.iter().enumerate() {
        assert_matches!(&res[i], a if &a.entry == result);
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
