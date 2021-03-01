#[cfg(test)]
mod test {
    use core::panic;
    use dummy_player::PlayerAction;
    use futures::{future::join_all, FutureExt, TryFutureExt};
    use gst::glib::{translate::FromGlib, SignalHandlerId};
    use gstreamer as gst;
    use gstreamer::ClockTime;
    use gstreamer_player::PlayerState;
    use lazy_static::lazy_static;
    use std::{
        borrow::{Borrow, BorrowMut},
        cell::{Ref, RefCell},
        sync::{
            atomic::{AtomicU32, Ordering},
            Arc, Mutex,
        },
        thread,
        time::Duration,
    };
    use tokio::time::{timeout, Timeout};

    use crate::{
        dummy_player::{self, DummyPlayer},
        player_actor::PlayerCommand,
        song_queue_actor::QueueCommand,
        start_tasks,
    };

    lazy_static! {
        static ref ACTIONS1: Mutex<Vec<PlayerAction>> = Mutex::new(vec!());
        static ref ACTIONS2: Mutex<Vec<PlayerAction>> = Mutex::new(vec!());
    }

    #[tokio::test]
    async fn test() {
        gst::init().unwrap();

        let mock1 = DummyPlayer {
            actions: &ACTIONS1,
            current_uri: "".to_owned(),
        };
        let mock2 = DummyPlayer {
            actions: &ACTIONS2,
            current_uri: "".to_owned(),
        };

        let (mut tasks, player_tx, queue_tx) = start_tasks(mock1, mock2);
        queue_tx
            .send(QueueCommand::SetQueue {
                songs: vec!["file://uri1".to_owned(), "file://uri2".to_owned()],
            })
            .await
            .unwrap();
        player_tx.send(PlayerCommand::Play { id: 0 }).await.unwrap();
        let _ = timeout(Duration::from_secs(1), tasks).await;
        //println!("{:?}", URIS1.lock().unwrap()[0]);

        let actions1 = ACTIONS1.lock().unwrap();

        match &actions1[0] {
            PlayerAction::SetUri { uri } => {
                assert_eq!("file://uri1", uri)
            }
            _ => panic!("wrong type"),
        }
    }
}
