#[cfg(test)]
mod test {
    use core::fmt;
    use flexi_logger::{LogTarget, Logger};
    use std::env::current_dir;

    use crate::libplayer::PlatunePlayer;
    use crate::libplayer::PlayerEvent;
    use assert_matches::*;
    use postage::prelude::Stream;

    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    async fn my_test() {
        Logger::with_str("info")
            .log_target(LogTarget::StdOut)
            .start()
            .unwrap();
        println!(
            "{}\\\\src\\test.mp3",
            current_dir().unwrap().to_str().unwrap()
        );
        let (mut player, mut receiver) = PlatunePlayer::new_with_events();
        player.set_queue(vec![format!(
            "{}\\src\\test.mp3",
            current_dir().unwrap().to_str().unwrap()
        )
        .to_owned()]);
        let msg = receiver.recv().await;

        let song = "test.mp3".to_owned();
        assert_matches!(msg, Some(PlayerEvent::Play { file }) if file == song);
    }
}
