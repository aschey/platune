#[cfg(test)]
mod test {
    use futures::{future::join_all, FutureExt};
    use gst::glib::{translate::FromGlib, SignalHandlerId};
    use gstreamer as gst;
    use gstreamer::ClockTime;
    use lazy_static::lazy_static;
    use std::{
        borrow::{Borrow, BorrowMut},
        cell::{Ref, RefCell},
        sync::{
            atomic::{AtomicU32, Ordering},
            Arc,
        },
        thread,
        time::Duration,
    };
    use tokio::time::{timeout, Timeout};

    use crate::{
        dummy_player::{self, DummyPlayer},
        song_queue_actor::QueueCommand,
        start_tasks,
    };

    // mock! {
    //     DummyPlayer {}
    //     impl Clone for DummyPlayer {
    //         fn clone(&self) -> Self;
    //     }

    //     impl PlayerBackend for DummyPlayer {
    //         fn play(&self) {

    //         }

    //         fn pause(&self);

    //         fn set_uri(&self, uri: &str);

    //         fn get_position(&self) -> ClockTime;

    //         fn get_duration(&self) -> ClockTime;

    //         fn seek(&self, position: ClockTime);

    //         fn connect_media_info_updated(&self, f: FnMediaInfo) -> gstreamer::glib::SignalHandlerId;

    //         fn connect_state_changed(&self, f: FnPlayerState) -> gstreamer::glib::SignalHandlerId;
    //     }
    // }

    lazy_static! {
        static ref incr1: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
        static ref incr2: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    }

    #[tokio::test]
    async fn test() {
        gst::init().unwrap();

        let mock1 = DummyPlayer { incr: &incr1 };
        let mock2 = DummyPlayer { incr: &incr2 };

        let (mut tasks, player_tx, queue_tx) = start_tasks(mock1, mock2);
        queue_tx
        .send(QueueCommand::SetQueue {
            songs: vec!["file://c/shared_files/Music/Between the Buried and Me/Colors/04 Sun Of Nothing.m4a".to_owned(),
            "file://c/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a".to_owned()],
        })
        .await
        .unwrap();

        let res = timeout(Duration::from_secs(5), tasks).await;
        println!("{:?}", incr1.load(Ordering::SeqCst));
        res.unwrap();
    }
}
