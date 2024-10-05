use std::fs::{self, create_dir_all};
use std::sync::Arc;

use futures::StreamExt;
use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey, TagExt};
use pretty_assertions::assert_eq;
use rstest::*;
use tempfile::TempDir;
use tracing::{Level, info};

use crate::config::MemoryConfig;
use crate::database::Database;
use crate::manager::Manager;

#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt()
        .pretty()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_test_writer()
        .with_max_level(Level::INFO)
        .try_init()
        .unwrap_or_default();
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
    let (_, mut manager) = setup().await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    for (i, song) in songs.iter().enumerate() {
        let song_path = inner_dir.join(format!("test{i}.mp3"));
        fs::copy("../test_assets/test.mp3", song_path.clone()).unwrap();
        let mut t = Probe::open(&song_path).unwrap().read().unwrap();
        let tag = t.primary_tag_mut().unwrap();
        if let Some(title) = song.title {
            tag.set_title(title.to_owned());
        }
        if let Some(artist) = song.artist {
            tag.set_artist(artist.to_owned());
        }
        if let Some(album_artist) = song.album_artist {
            tag.insert_text(ItemKey::AlbumArtist, album_artist.to_owned());
        }
        if let Some(album) = song.album {
            tag.set_album(album.to_owned());
        }
        tag.save_to_path(song_path, WriteOptions::new()).unwrap();
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
}

async fn setup() -> (Database, Manager) {
    let db = Database::connect_in_memory().await.unwrap();
    db.sync_database().await.unwrap();
    let config = Arc::new(MemoryConfig::new_boxed());
    let manager = Manager::new(&db, config);
    (db, manager)
}
