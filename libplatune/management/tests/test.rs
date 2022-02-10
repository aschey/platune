use futures::StreamExt;
use libplatune_management::{config::Config, database::Database, manager::Manager};
use rstest::*;
use std::{
    fs::{self, create_dir, create_dir_all},
    path::Path,
    time::Duration,
};
use tempfile::TempDir;
use tokio::time::timeout;
use tracing::{info, Level};

#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt()
        .pretty()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_test_writer()
        .with_max_level(Level::INFO)
        .init();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_empty() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    create_dir(music_dir.clone()).unwrap();
    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();
    let mut receiver = manager.sync(None).await.unwrap();
    let mut msgs = vec![];
    while let Some(msg) = receiver.next().await {
        msgs.push(msg.unwrap());
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(vec![1.], msgs);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_no_folder() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;

    let mut receiver = manager.sync(None).await.unwrap();
    let mut msgs = vec![];
    while let Some(msg) = receiver.next().await {
        msgs.push(msg.unwrap());
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(Vec::<f32>::new(), msgs);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_basic() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    fs::copy("../test_assets/test.mp3", inner_dir.join("test.mp3")).unwrap();
    fs::copy("../test_assets/test2.mp3", inner_dir.join("test2.mp3")).unwrap();
    fs::copy("../test_assets/test3.mp3", inner_dir.join("test3.mp3")).unwrap();

    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();
    let mut receiver = manager.sync(None).await.unwrap();

    let mut msgs = vec![];
    while let Some(msg) = receiver.next().await {
        msgs.push(msg.unwrap());
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(vec![0., 1.], msgs);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_multiple() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");

    for i in 0..10 {
        create_dir_all(inner_dir.clone().join(format!("test{}", i))).unwrap();

        fs::copy(
            "../test_assets/test.mp3",
            inner_dir.join(format!("test{}/test.mp3", i)),
        )
        .unwrap();
        fs::copy(
            "../test_assets/test2.mp3",
            inner_dir.join(format!("test{}/test2.mp3", i)),
        )
        .unwrap();
        fs::copy(
            "../test_assets/test3.mp3",
            inner_dir.join(format!("test{}/test3.mp3", i)),
        )
        .unwrap();
    }

    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();

    let mut receiver1 = manager.sync(None).await.unwrap();
    let mut receiver2 = manager.sync(None).await.unwrap();

    let handle1 = tokio::spawn(async move {
        let mut msgs = vec![];
        while let Some(msg) = receiver1.next().await {
            msgs.push(msg.unwrap());
        }
        msgs
    });

    let handle2 = tokio::spawn(async move { while receiver2.next().await.is_some() {} });

    let msgs = handle1.await.unwrap();
    handle2.await.unwrap();

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(vec![0., 1.], msgs);
}

async fn setup_delete(inner_dir: &Path, music_dir: &Path, manager: &mut Manager) {
    fs::copy("../test_assets/test.mp3", inner_dir.join("test.mp3")).unwrap();
    fs::copy("../test_assets/test2.mp3", inner_dir.join("test2.mp3")).unwrap();
    let last_song = inner_dir.join("test3.mp3");
    fs::copy("../test_assets/test3.mp3", last_song.clone()).unwrap();

    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();
    let mut receiver = manager.sync(None).await.unwrap();

    while receiver.next().await.is_some() {}

    fs::remove_file(last_song.clone()).unwrap();
    // We store unix timestamps at a granularity of seconds so we need to wait for enough time to pass
    std::thread::sleep(Duration::from_secs(2));

    let mut receiver = manager.sync(None).await.unwrap();
    while receiver.next().await.is_some() {}

    // sync twice after deleting to ensure no unique constraint errors
    let mut receiver = manager.sync(None).await.unwrap();
    while receiver.next().await.is_some() {}
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_delete() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    setup_delete(&inner_dir, &music_dir, &mut manager).await;

    let deleted = manager.get_deleted_songs().await.unwrap();

    manager
        .delete_tracks(vec![deleted[0].song_id])
        .await
        .unwrap();

    let deleted2 = manager.get_deleted_songs().await.unwrap();

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    let last_song = inner_dir.join("test3.mp3");

    assert_eq!(1, deleted.len());
    assert_eq!(
        deleted[0].song_path,
        last_song.to_string_lossy().to_string().replace("\\", "/")
    );

    // TODO: add test to check if number of songs decreased once we have an endpoint to get all songs
    assert_eq!(0, deleted2.len());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_delete_and_readd() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    setup_delete(&inner_dir, &music_dir, &mut manager).await;
    let deleted = manager.get_deleted_songs().await.unwrap();

    let last_song = inner_dir.join("test3.mp3");

    fs::copy("../test_assets/test3.mp3", last_song.clone()).unwrap();

    let mut receiver = manager.sync(None).await.unwrap();
    while receiver.next().await.is_some() {}

    let deleted2 = manager.get_deleted_songs().await.unwrap();

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(1, deleted.len());
    assert_eq!(
        deleted[0].song_path,
        last_song.to_string_lossy().to_string().replace("\\", "/")
    );

    // TODO: add test to check if number of songs decreased once we have an endpoint to get all songs
    assert_eq!(0, deleted2.len());
}

#[derive(Default)]
pub struct SongTest {
    title: Option<&'static str>,
    artist: Option<&'static str>,
    album_artist: Option<&'static str>,
    album: Option<&'static str>,
}

#[derive(Debug)]
pub struct SearchResultTest {
    entry: &'static str,
    correlation_ids: Vec<i32>,
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
        vec![SearchResultTest {entry: "asdf", correlation_ids: vec![1]}],
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
        vec![
            SearchResultTest {entry: "bless", correlation_ids: vec![2]},
            SearchResultTest {entry: "bliss", correlation_ids: vec![1]}
        ],
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
        vec![
            SearchResultTest {entry: "bliss", correlation_ids: vec![1]},
            SearchResultTest {entry: "bless blah blah", correlation_ids: vec![2]}
        ],
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
        vec![
            SearchResultTest {entry: "bliss", correlation_ids: vec![1]},
            SearchResultTest {entry: "blah bless blah", correlation_ids: vec![2]}
        ],
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
        vec![
            SearchResultTest {entry: "bless", correlation_ids: vec![2]}
        ],
        "blss"),
    case(vec![
        SongTest {
            title: Some("Color of Autumn"),
            ..Default::default()
        },
        SongTest {
            title: Some("Colors"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "Colors", correlation_ids: vec![2]},
            SearchResultTest {entry: "Color of Autumn", correlation_ids: vec![1]}
        ],
        "colors"),
    case(vec![
        SongTest {
            title: Some("Censored Colors"),
            ..Default::default()
        },
        SongTest {
            title: Some("Colors"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "Colors", correlation_ids: vec![2]},
            SearchResultTest {entry: "Censored Colors", correlation_ids: vec![1]}
        ],
        "colors"),
    case(vec![
        SongTest {
            artist: Some("red hot chili peppers"),
            ..Default::default()
        },
        SongTest {
            artist: Some("real hearty chopped pies"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "red hot chili peppers", correlation_ids: vec![2]},
            SearchResultTest {entry: "real hearty chopped pies", correlation_ids: vec![1]},
        ],
        "rhcp"),
    case(vec![
        SongTest {
            artist: Some("red hot chili peppers"),
            ..Default::default()
        },
        SongTest {
            artist: Some("rad hot chili peppers"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "rad hot chili peppers", correlation_ids: vec![2]},
            SearchResultTest {entry: "red hot chili peppers", correlation_ids: vec![1]},
        ],
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
        vec![
            SearchResultTest {entry: "red hot chili peppers", correlation_ids: vec![2]},
            SearchResultTest {entry: "rhcpa", correlation_ids: vec![1]}
        ],
        "rhcp"),
    case(vec![
        SongTest {
            artist: Some("between the buried and me"),
            ..Default::default()
        },
        SongTest {
            artist: Some("thee"),
            ..Default::default()
        },],
        vec![
            SearchResultTest {entry: "between the buried and me", correlation_ids: vec![2]},
        ],
        "betwen the b"),
    case(vec![
        SongTest {
            artist: Some("a/b"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "a/b", correlation_ids: vec![1]},
        ],
        "a b"),
    case(vec![
        SongTest {
            artist: Some("a/b"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "a/b", correlation_ids: vec![1]},
        ],
        "a/b"),
    case(vec![
        SongTest {
            artist: Some("a & b"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "a and b", correlation_ids: vec![1]},
        ],
        "a & b"),
    case(vec![
        SongTest {
            artist: Some("a & b"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "a and b", correlation_ids: vec![1]},
        ],
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
        vec![
            SearchResultTest {entry: "a and b", correlation_ids: vec![1, 2]},
        ],
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
        vec![
            SearchResultTest {entry: "a and b", correlation_ids: vec![1, 2]},
        ],
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
        vec![
            SearchResultTest {entry: "a and b", correlation_ids: vec![1, 2]},
        ],
        "a b"),
    case(vec![
        SongTest {
            artist: Some("br&nd"),
            ..Default::default()
        },],
        vec![
            SearchResultTest {entry: "br&nd", correlation_ids: vec![1]},
        ],
        "br&nd"),
    case(vec![
        SongTest {
            artist: Some("red hot & chili peppers"),
            ..Default::default()
        },
        SongTest {
            artist: Some("red hot and chili peppers"),
            ..Default::default()
        },],
        vec![
            SearchResultTest {entry: "red hot and chili peppers", correlation_ids: vec![1, 2]},
        ],
        "rhacp"),
    case(vec![
        SongTest {
            artist: Some("red hot & chili peppers"),
            ..Default::default()
        },
        SongTest {
            artist: Some("red hot and chili peppers"),
            ..Default::default()
        },],
        vec![
            SearchResultTest {entry: "red hot and chili peppers", correlation_ids: vec![1, 2]},
        ],
        "rh&cp"),
    case(vec![
        SongTest {
            artist: Some("bad bad"),
            ..Default::default()
        },
        SongTest {
            artist: Some("bag"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "bag", correlation_ids: vec![1]},
            SearchResultTest {entry: "bad bad", correlation_ids: vec![2]},
        ],
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
        vec![
            SearchResultTest {entry: "bag", correlation_ids: vec![1]},
            SearchResultTest {entry: "bad bag", correlation_ids: vec![2]},
            SearchResultTest {entry: "bad bad", correlation_ids: vec![3]},
        ],
        "bag"),
    case(vec![
        SongTest {
            artist: Some("qwerty"),
            title: Some("untitled"),
            ..Default::default()
        },
        SongTest {
            artist: Some("qwerty"),
            title: Some("untitled 2"),
            ..Default::default()
        },
        SongTest {
            artist: Some("bag"),
            title: Some("untitled"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "untitled", correlation_ids: vec![3]},
            SearchResultTest {entry: "untitled 2", correlation_ids: vec![2]},
        ],
        "untitled artist:qwerty"),
    case(vec![
        SongTest {
            artist: Some("red hot chili peppers"),
            title: Some("untitled"),
            ..Default::default()
        },
        SongTest {
            artist: Some("red hot chili peppers"),
            title: Some("untitled 2"),
            ..Default::default()
        },
        SongTest {
            artist: Some("bag"),
            title: Some("untitled"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "untitled", correlation_ids: vec![3]},
            SearchResultTest {entry: "untitled 2", correlation_ids: vec![2]},
        ],
        "untitled artist:rhcp"),
    case(vec![
        SongTest {
            artist: Some("abc test"),
            album_artist: Some("abc test"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "abc test", correlation_ids: vec![1]},
        ],
        "abc"),
    case(vec![
        SongTest {
            album_artist: Some("abc test"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "abc test", correlation_ids: vec![1]},
        ],
        "abc"),
    case(vec![
        SongTest {
            album_artist: Some("abc test"),
            ..Default::default()
        },
        SongTest {
            artist: Some("abc test"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "abc test", correlation_ids: vec![1]},
        ],
        "abc"),
    case(vec![
        SongTest {
            album_artist: Some("abc test1"),
            ..Default::default()
        },
        SongTest {
            artist: Some("abc test2"),
            ..Default::default()
        }],
        vec![
            SearchResultTest {entry: "abc test1", correlation_ids: vec![1]},
            SearchResultTest {entry: "abc test2", correlation_ids: vec![1]},
        ],
        "abc"),
    case(vec![
        SongTest {
            album: Some("abc test"),
            ..Default::default()
        },],
        vec![
            SearchResultTest {entry: "abc test", correlation_ids: vec![1]},
        ],
        "abc"),
    case(vec![
        SongTest {
            album: Some("abc test"),
            ..Default::default()
        },
        SongTest {
            title: Some("abc test"),
            ..Default::default()
        },
        SongTest {
            artist: Some("abc test"),
            ..Default::default()
        },
        SongTest {
            album_artist: Some("abc test"),
            ..Default::default()
        },],
        vec![
            SearchResultTest {entry: "abc test", correlation_ids: vec![1]},
            SearchResultTest {entry: "abc test", correlation_ids: vec![1]},
            SearchResultTest {entry: "abc test", correlation_ids: vec![1]},
        ],
        "abc"),
    case(vec![
        SongTest {
            title: Some("qwerty song 1"),
            album: Some("qwerty album 1"),
            artist: Some("qwerty artist 1"),
            ..Default::default()
        },
        SongTest {
            title: Some("qwerty song 1"),
            album: Some("qwerty album 2"),
            artist: Some("qwerty artist 2"),
            ..Default::default()
        },],
        vec![
            SearchResultTest {entry: "qwerty song 1", correlation_ids: vec![1]},
            SearchResultTest {entry: "qwerty song 1", correlation_ids: vec![2]},
        ],
        "qwerty song 1"),
    case(vec![
        SongTest {
            title: Some("qwerty song 1"),
            album: Some("qwerty album 1"),
            artist: Some("qwerty artist 1"),
            ..Default::default()
        },
        SongTest {
            title: Some("qwerty song 1"),
            album: Some("qwerty album 2"),
            artist: Some("qwerty artist 1"),
            ..Default::default()
        },],
        vec![
            SearchResultTest {entry: "qwerty song 1", correlation_ids: vec![1]},
            SearchResultTest {entry: "qwerty song 1", correlation_ids: vec![2]},
        ],
        "qwerty song 1"),
    case(vec![
        SongTest {
            title: Some("life 1"),
            ..Default::default()
        },
        SongTest {
            title: Some("life 2"),
            ..Default::default()
        },
        SongTest {
            title: Some("life 3"),
            ..Default::default()
        },
        SongTest {
            title: Some("ligne"),
            ..Default::default()
        },],
        vec![
            SearchResultTest {entry: "life 1", correlation_ids: vec![1]},
            SearchResultTest {entry: "life 2", correlation_ids: vec![2]},
            SearchResultTest {entry: "life 3", correlation_ids: vec![3]},
            SearchResultTest {entry: "ligne", correlation_ids: vec![4]},
        ],
        "lige"),
    )
]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_search(songs: Vec<SongTest>, results: Vec<SearchResultTest>, search: &str) {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    for (i, song) in songs.iter().enumerate() {
        let song_path = inner_dir.join(format!("test{}.mp3", i));
        fs::copy("../test_assets/test.mp3", song_path.clone()).unwrap();
        let t = katatsuki::ReadWriteTrack::from_path(&song_path, None).unwrap();

        if let Some(title) = song.title {
            t.set_title(title);
        }
        if let Some(artist) = song.artist {
            t.set_artist(artist);
        }
        if let Some(album_artist) = song.album_artist {
            t.set_album_artists(album_artist);
        }
        if let Some(album) = song.album {
            t.set_album(album);
        }
        t.save();
    }

    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();
    let mut receiver = manager.sync(None).await.unwrap();

    while receiver.next().await.is_some() {}

    let res = manager.search(search, Default::default()).await.unwrap();

    info!("expected {:?}\n", results);
    info!("actual {:?}", res);

    assert!(res.len() == results.len());

    for (i, result) in results.iter().enumerate() {
        assert_eq!(&res[i].entry, result.entry);
        assert_eq!(&res[i].correlation_ids.len(), &result.correlation_ids.len());
        let mut ids = res[i].correlation_ids.clone();
        ids.sort_unstable();
        // TODO: re-enable after creating function to look up id
        // for (j, id) in result.correlation_ids.iter().enumerate() {
        //     assert_eq!(&ids[j], id);
        // }
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();
}

async fn setup(tempdir: &TempDir) -> (Database, Manager) {
    let sql_path = tempdir.path().join("platune.db");
    info!("{:?}", sql_path);
    let config_path = tempdir.path().join("platuneconfig");
    let db = Database::connect(sql_path, true).await.unwrap();
    db.migrate().await.unwrap();
    let config = Config::new_from_path(config_path).unwrap();
    let manager = Manager::new(&db, &config);
    (db, manager)
}
