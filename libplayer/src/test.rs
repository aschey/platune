#[cfg(test)]
mod test {
    use core::fmt;
    use flexi_logger::{LogTarget, Logger};
    use std::{env::current_dir, thread, time::Duration};

    use crate::libplayer::PlatunePlayer;
    use crate::libplayer::PlayerEvent;
    use assert_matches::*;
    use postage::prelude::Stream;

    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    async fn my_test() {
        gstreamer::init().unwrap();
        Logger::with_str("info")
            .log_target(LogTarget::StdOut)
            .start()
            .unwrap();

        let (mut player, mut receiver) = PlatunePlayer::new_with_events();
        let song1 = "test.mp3".to_owned();
        let song2 = "test2.mp3".to_owned();
        let dir = current_dir().unwrap().to_str().unwrap().to_owned();
        player.set_queue(vec![
            format!("{}/test_files/{}", dir, song1).to_owned(),
            format!("{}/test_files/{}", dir, song2).to_owned(),
        ]);
        let msg1 = receiver.recv().await;
        assert_matches!(msg1, Some(PlayerEvent::Play { file }) if file == song1);
        let msg2 = receiver.recv().await;
        assert_matches!(msg2, Some(PlayerEvent::Ended { file }) if file == song1);
        let msg3 = receiver.recv().await;
        assert_matches!(msg3, Some(PlayerEvent::Ended { file }) if file == song2);
    }
}
