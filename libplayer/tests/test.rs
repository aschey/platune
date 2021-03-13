#[cfg(test)]
mod test {
    use core::fmt;
    use flexi_logger::{LogTarget, Logger};
    use rstest::*;
    use std::{
        env::current_dir,
        thread,
        time::{Duration, Instant},
    };

    use assert_matches::*;
    use platune_libplayer::libplayer::PlatunePlayer;
    use platune_libplayer::libplayer::PlayerEvent;
    use postage::prelude::Stream;

    #[cfg(not(target_os = "windows"))]
    pub static SEPARATOR: &str = "/";
    #[cfg(target_os = "windows")]
    pub static SEPARATOR: &str = "\\";

    struct SongInfo {
        path: String,
        name: String,
        duration_estimate_millis: u128,
    }

    trait SongVec {
        fn get_paths(&self) -> Vec<String>;
    }

    impl SongVec for Vec<SongInfo> {
        fn get_paths(&self) -> Vec<String> {
            self.iter().map(|s| s.path.to_owned()).collect()
        }
    }

    #[ctor::ctor]
    fn init() {
        gstreamer::init().unwrap();
        // Logger::with_str("info")
        //     .log_target(LogTarget::StdOut)
        //     .start()
        //     .unwrap();
    }

    fn get_path(song: &str) -> String {
        let dir = current_dir().unwrap().to_str().unwrap().to_owned();
        format!("{1}{0}tests{0}assets{0}{2}", SEPARATOR, dir, song).to_string()
    }

    fn get_test_files(num_songs: usize) -> Vec<SongInfo> {
        let song1 = SongInfo {
            name: "test.mp3".to_owned(),
            path: get_path("test.mp3"),
            duration_estimate_millis: 444,
        };
        let song2 = SongInfo {
            name: "test2.mp3".to_owned(),
            path: get_path("test2.mp3"),
            duration_estimate_millis: 731,
        };

        match num_songs {
            1 => vec![song1],
            2 => vec![song1, song2],
            _ => vec![],
        }
    }

    fn assert_duration(min: u128, val: u128) {
        assert!((min - 50) <= val && val < min + 50, "duration={}", val);
    }

    #[rstest]
    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    async fn test_one_song() {
        let (mut player, mut receiver) = PlatunePlayer::new_with_events();
        let songs = get_test_files(1);

        player.set_queue(songs.get_paths());

        let song = &songs[0];

        assert_matches!(receiver.recv().await, Some(PlayerEvent::Play { file }) if file == song.name);
        assert_matches!( receiver.recv().await, Some(PlayerEvent::Ended { file }) if file == song.name);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::QueueEnded));
    }

    #[rstest]
    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    async fn test_two_songs() {
        let (mut player, mut receiver) = PlatunePlayer::new_with_events();
        let songs = get_test_files(2);

        player.set_queue(songs.get_paths());

        let song1 = &songs[0];
        let song2 = &songs[1];

        assert_matches!( receiver.recv().await, Some(PlayerEvent::Play { file }) if file == song1.name);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::Ended { file }) if file == song1.name);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::Ended { file }) if file == song2.name);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::QueueEnded));
    }

    #[rstest]
    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    async fn test_pause() {
        let (mut player, mut receiver) = PlatunePlayer::new_with_events();
        let songs = get_test_files(2);

        player.set_queue(songs.get_paths());

        let song1 = &songs[0];
        let song2 = &songs[1];

        assert_matches!( receiver.recv().await, Some(PlayerEvent::Play { file }) if file == song1.name);
        player.pause();
        assert_matches!(receiver.recv().await, Some(PlayerEvent::Pause { file }) if file == song1.name);
        player.resume();
        assert_matches!(receiver.recv().await, Some(PlayerEvent::Resume { file }) if file == song1.name);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::Ended { file }) if file == song1.name);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::Ended { file }) if file == song2.name);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::QueueEnded));
    }

    #[rstest(seek_time, case(0.1), case(0.5))]
    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    async fn test_seek(seek_time: f64) {
        let (mut player, mut receiver) = PlatunePlayer::new_with_events();
        let songs = get_test_files(2);

        player.set_queue(songs.get_paths());

        let song1 = &songs[0];
        let song2 = &songs[1];

        assert_matches!( receiver.recv().await, Some(PlayerEvent::Play { file }) if file == song1.name);
        player.seek(seek_time);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::Seek { file, time }) if file == song1.name && time == seek_time);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::Ended { file }) if file == song1.name);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::Ended { file }) if file == song2.name);
        assert_matches!(receiver.recv().await, Some(PlayerEvent::QueueEnded));
    }
}
