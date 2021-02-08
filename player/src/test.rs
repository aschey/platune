#[cfg(test)]
mod test {
    use crate::{dummy_player::DummyPlayer, song_queue_actor::QueueCommand, start_tasks};

    #[tokio::test]
    async fn test() {
        let (player_tx, queue_tx) = start_tasks::<DummyPlayer>();
        queue_tx
        .send(QueueCommand::SetQueue {
            songs: vec!["file://c/shared_files/Music/Between the Buried and Me/Colors/04 Sun Of Nothing.m4a".to_owned(),
            "file://c/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a".to_owned()],
        })
        .await
        .unwrap();
    }
}
