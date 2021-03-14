// use std::sync::{Arc, Mutex, Once};

// use lazy_static::lazy_static;
// use servo_media::{BackendInit, ClientContextId, ServoMedia};
// use servo_media_audio::context::{AudioContext, AudioContextOptions, RealTimeAudioContextOptions};

// lazy_static! {
//     pub static ref CONTEXT: Arc<Mutex<AudioContext>> = {
//         // #[cfg(test)]
//         // ServoMedia::init::<servo_media_auto::DummyBackend>();
//         // #[cfg(not(test))]
//         //ServoMedia::init::<servo_media_auto::Backend>();
//         let servo_media = ServoMedia::get().unwrap();
//         let context = servo_media.create_audio_context(
//             &ClientContextId::build(1, 1),
//             AudioContextOptions::RealTimeAudioContext(RealTimeAudioContextOptions::default()),
//         );
//         context.lock().unwrap().resume().unwrap();
//         context
//     };
// }

// static INITIALIZER: Once = Once::new();

// // pub fn init<B: BackendInit>() {
// //     INITIALIZER.call_once(|| {
// //         INSTANCE = servo_media.create_audio_context(
// //             &ClientContextId::build(1, 1),
// //             AudioContextOptions::RealTimeAudioContext(RealTimeAudioContextOptions::default()),
// //         );
// //     });
// // }
