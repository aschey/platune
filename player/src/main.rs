use byte_slice_cast::AsSliceOf;
use gst::GstBinExtManual;
use gst::{prelude::*, ClockTime};
use gstreamer as gst;
use gstreamer::{glib, prelude::Cast, Pipeline};
use gstreamer_app as gst_app;
use gstreamer_audio as gst_audio;
use gstreamer_player::{
    Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher, PlayerState,
};
use std::{
    cell::RefCell,
    fs::{self, File},
    io::{Cursor, Read, Seek, SeekFrom},
    sync::{mpsc, Arc, Mutex},
};

fn main() {
    gst::init().unwrap();
    let uri = "file://c/shared_files/Music/4 Strings/Believe/01 Intro.m4a";
    //let uri = "C:\\shared_files\\Music\\The Mars Volta\\Frances the Mute\\06 Cassandra Gemini.mp3";
    //let uri = "C:\\shared_files\\Music\\Between the Buried and Me\\Colors\\05 Ants of the Sky.m4a";
    let main_loop = glib::MainLoop::new(None, false);

    let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
    let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));
    let clock = gst::SystemClock::obtain();
    let loaded = RefCell::new(false);

    player.connect_media_info_updated(move |player, info| {
        let duration = info.get_duration().unwrap_or_default();
        let clock_weak = clock.downgrade();
        if duration > 0 {
            if *loaded.borrow() {
                println!("loaded");
                return;
            }
            *loaded.borrow_mut() = true;
            let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
            let player2 = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));
            player2.set_uri(
                "file://c/shared_files/Music/4 Strings/Believe/02 Take Me Away (Into The Night).m4a",
            );
            player2.pause();
            player.connect_state_changed(move |player, player_state| {
                println!("{:?}", player_state);
                let clock = clock_weak.upgrade().unwrap();
                if player_state == PlayerState::Playing {
                    let time = clock.get_time();
                    let nseconds = time.nseconds().unwrap();
                    let new_time = ClockTime::from_nseconds(nseconds + duration);

                    let shot_id = clock.new_single_shot_id(new_time).unwrap();
                    //let player_weak = player.downgrade();
                    let player2_weak = player2.downgrade();
                    
                    shot_id
                        .wait_async(move |_, _, _| {
                            let player2 = player2_weak.upgrade().unwrap();
                            //let player = player_weak.upgrade().unwrap();
                            
                            player2.play();
                        })
                        .unwrap();

                }
            });
            player.play();
        }
    });

    player.set_uri(uri);
    // Start player so media data is loaded but don't play yet
    player.pause();

    main_loop.run();
}
