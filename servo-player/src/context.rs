use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use servo_media::{ClientContextId, ServoMedia};
use servo_media_audio::context::{AudioContext, AudioContextOptions, RealTimeAudioContextOptions};

lazy_static! {
    pub static ref CONTEXT: Arc<Mutex<AudioContext>> = {
        ServoMedia::init::<servo_media_auto::Backend>();
        let servo_media = ServoMedia::get().unwrap();
        servo_media.create_audio_context(
            &ClientContextId::build(1, 1),
            AudioContextOptions::RealTimeAudioContext(RealTimeAudioContextOptions::default()),
        )
    };
}
