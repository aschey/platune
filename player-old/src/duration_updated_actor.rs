// use gstreamer::prelude::Cast;
// use gstreamer_player::{Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher};

// pub struct DurationUpdatedActor {
//     pub cur_song: String,
// }

// impl DurationUpdatedActor {
//     pub fn new() -> DurationUpdatedActor {
//         DurationUpdatedActor {
//             cur_song: "".to_owned(),
//         }
//     }

//     async fn handle(&mut self, song_duration: SongDuration) {
//         if song_duration.song == self.cur_song || song_duration.duration == 0 {
//             return;
//         }
//         let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
//         let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));
//         player.set_uri(
//             //"file://c/shared_files/Music/4 Strings/Believe/02 Take Me Away (Into The Night).m4a",
//             "file://c/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a",
//         );
//         player.pause();
//         self.cur_song = song_duration.song;
//     }
// }

// pub struct SongDuration {
//     pub song: String,
//     pub duration: u64,
// }
