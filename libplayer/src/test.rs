#[cfg(test)]
mod test {
    use crate::libplayer::PlatunePlayer;
    use crate::libplayer::PlayerEvent;
    use assert_matches::*;
    use postage::prelude::Stream;

    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    async fn my_test() {
        let (mut player, mut receiver) = PlatunePlayer::new_with_events();
        player.set_queue(vec!["C:\\\\shared_files\\Music\\emoisdead\\Peu Etre - Langue Et Civilisation Hardcore (199x)\\Peu Etre-17-Track 17.mp3".to_owned()]);
        let msg = receiver.recv().await;

        let song = "Peu Etre-17-Track 17.mp3".to_owned();
        assert_matches!(msg, Some(PlayerEvent::Play { file }) if file == song);
    }
}
