use gstreamer::{Clock, ClockExtManual, ClockId, ClockTime, SystemClock};
use std::{thread, time::Duration};
use tokio::sync::mpsc::Sender;

use crate::player_actor::PlayerCommand;

pub struct SongStartActor {
    pub clock_id: Option<ClockId>,
    pub clock: Clock,
    subscriber: Sender<PlayerCommand>,
}

impl SongStartActor {
    pub fn new(subscriber: Sender<PlayerCommand>) -> SongStartActor {
        SongStartActor {
            clock: SystemClock::obtain(),
            clock_id: None,
            subscriber,
        }
    }

    pub async fn handle(&mut self, msg: StartSeconds) -> () {
        if let Some(shot) = &self.clock_id {
            shot.unschedule();
        }

        let clock_id = self
            .clock
            .new_single_shot_id(ClockTime::from_nseconds(msg.nseconds))
            .unwrap();

        let subscriber = self.subscriber.clone();
        println!("{:?}", msg.player_id);

        subscriber.try_send(PlayerCommand::SetUri { id: msg.player_id, uri: "file://c/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a".to_owned()}).unwrap();
        thread::sleep(Duration::from_millis(50));
        subscriber
            .try_send(PlayerCommand::Pause { id: msg.player_id })
            .unwrap();

        clock_id
            .wait_async(move |_, _, _| {
                subscriber
                    .try_send(PlayerCommand::Play { id: msg.player_id })
                    .unwrap();
            })
            .unwrap();

        self.clock_id = Some(clock_id);
    }
}
pub struct StartSeconds {
    pub nseconds: u64,
    pub player_id: usize,
}
