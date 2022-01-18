use dasp::{
    interpolate::{linear::Linear, sinc::Sinc},
    ring_buffer::Fixed,
    signal::{
        from_interleaved_samples_iter, interpolate::Converter, FromInterleavedSamplesIterator,
        UntilExhausted,
    },
    Frame, Signal,
};
use rubato::{FftFixedIn, FftFixedInOut, Resampler};

use std::{cell::RefCell, rc::Rc};

use crate::{
    audio_processor::{AudioProcessor, AudioProcessorState},
    dto::{command::Command, player_response::PlayerResponse, queue_source::QueueSource},
    output::{AudioOutput, CpalAudioOutput},
    player::Player,
    settings::resample_mode::ResampleMode,
    source::Source,
    stereo_stream::StereoStream,
    TwoWayReceiverAsync,
};
use crossbeam_channel::{Receiver, TryRecvError};
use tracing::{error, info};

pub(crate) fn decode_loop(
    queue_rx: Receiver<QueueSource>,
    processor_state: Rc<RefCell<AudioProcessorState>>,
) {
    let mut output = CpalAudioOutput::new_output().unwrap();
    let mut resampler = FftFixedInOut::<f64>::new(44_100 as usize, 48_000, 1024, 2);

    loop {
        match queue_rx.try_recv() {
            Ok(QueueSource {
                source,
                resample_mode,
                force_restart_output,
            }) => {
                if force_restart_output {
                    output.stop();
                    output.start();
                }
                decode_source(
                    source,
                    processor_state.clone(),
                    &resample_mode,
                    &mut output,
                    &mut resampler,
                );
            }
            Err(TryRecvError::Empty) => {
                // If no pending source, stop the output to preserve cpu
                output.stop();
                match queue_rx.recv() {
                    Ok(QueueSource {
                        source,
                        resample_mode,
                        ..
                    }) => {
                        output.start();
                        decode_source(
                            source,
                            processor_state.clone(),
                            &resample_mode,
                            &mut output,
                            &mut resampler,
                        );
                    }
                    Err(_) => break,
                };
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
        }
    }
}

fn decode_source(
    source: Box<dyn Source>,
    processor_state: Rc<RefCell<AudioProcessorState>>,
    resample_mode: &ResampleMode,
    output: &mut Box<dyn AudioOutput>,
    resampler: &mut FftFixedInOut<f64>,
) {
    let output_channels = output.channels();
    let mut processor = AudioProcessor::new(source, output_channels, processor_state);

    // No need to resample if sample rates are equal
    let source_sample_rate = processor.sample_rate();
    let output_sample_rate = output.sample_rate();
    // if source_sample_rate == output_sample_rate || resample_mode == &ResampleMode::None {
    //     // info!(
    //     //     "Not resampling. Source sample rate={}, output sample rate={}.",
    //     //     source_sample_rate, output_sample_rate
    //     // );
    //     // output.write_stream(Box::new(processor));
    //     return;
    // }

    let output_sample_rate = output.sample_rate() as f64;

    match (output_channels, &resample_mode) {
        (1, ResampleMode::Linear) => {
            // let left = processor.next().unwrap();
            // let right = processor.next().unwrap();
            // let resampled = linear_resample(left, right, processor, output_sample_rate);

            // output.write_stream(resampled);
        }
        (2, _) => {
            let n_frames = resampler.nbr_frames_needed();

            let mut buf = [vec![0.0; n_frames], vec![0.0; n_frames]];

            let mut buf2 = vec![0.0; n_frames * 2];
            processor.next();
            let mut frame_pos = 0;
            loop {
                let mut i = 0;
                while i < n_frames {
                    buf[0][i] = processor.current()[frame_pos];
                    buf[1][i] = processor.current()[frame_pos + 1];
                    i += 1;
                    frame_pos += 2;
                    if frame_pos == processor.current().len() {
                        if processor.next().is_none() {
                            return;
                        }
                        frame_pos = 0;
                    }
                }
                //println!("{:?}", buf);

                let resampled = resampler.process(&buf).unwrap();
                let out_len = resampled[0].len();
                if out_len * 2 > buf2.len() {
                    buf2.clear();
                    buf2.resize(out_len * 2, 0.0);
                }

                //let mut i = 0;
                let l = &resampled[0];
                let r = &resampled[1];
                let mut j = 0;
                for i in 0..out_len {
                    buf2[j] = l[i];
                    buf2[j + 1] = r[i];
                    j += 2;
                }

                // for sample in resampled {
                //     buf2[i] = sample[0];
                //     buf2[i + 1] = sample[1];
                //     i += 2;
                // }
                output.write(&buf2);
            }

            // let left = [processor.next().unwrap(), processor.next().unwrap()];
            // let right = [processor.next().unwrap(), processor.next().unwrap()];
            // let resampled = linear_resample(left, right, processor, output_sample_rate);
            // let stereo_resampled = Box::new(StereoStream::new(resampled));

            // output.write_stream(stereo_resampled);
        }
        (1, ResampleMode::Sinc) => {
            // let resampled = sinc_resample::<f64>(processor, output_sample_rate);

            // output.write_stream(Box::new(resampled));
        }
        (2, ResampleMode::Sinc) => {
            // let resampled = sinc_resample::<[f64; 2]>(processor, output_sample_rate);

            // output.write_stream(Box::new(StereoStream::new(resampled)));
        }
        _ => {}
    }
}

// fn sinc_resample<T>(
//     processor: AudioProcessor,
//     output_sample_rate: f64,
// ) -> Box<UntilExhausted<Converter<FromInterleavedSamplesIterator<AudioProcessor, T>, Sinc<[T; 128]>>>>
// where
//     T: Frame<Sample = f64>,
// {
//     let buf = [T::EQUILIBRIUM; 128];
//     let source_sample_rate = processor.sample_rate();
//     let signal = from_interleaved_samples_iter(processor);
//     let ring_buffer = Fixed::from(buf);

//     let converter = Sinc::new(ring_buffer);

//     let new_signal = signal.from_hz_to_hz(converter, source_sample_rate as f64, output_sample_rate);
//     Box::new(new_signal.until_exhausted())
// }

// fn linear_resample<T>(
//     left: T,
//     right: T,
//     processor: AudioProcessor,
//     output_sample_rate: f64,
// ) -> Box<UntilExhausted<Converter<FromInterleavedSamplesIterator<AudioProcessor, T>, Linear<T>>>>
// where
//     T: Frame<Sample = f64>,
// {
//     let source_sample_rate = processor.sample_rate();
//     let signal = from_interleaved_samples_iter(processor);

//     let converter = Linear::new(left, right);

//     let new_signal = signal.from_hz_to_hz(converter, source_sample_rate as f64, output_sample_rate);
//     Box::new(new_signal.until_exhausted())
// }

pub(crate) async fn main_loop(
    mut receiver: TwoWayReceiverAsync<Command, PlayerResponse>,
    mut player: Player,
) {
    while let Some(next_command) = receiver.recv().await {
        info!("Got command {:?}", next_command);
        match next_command {
            Command::SetQueue(songs) => {
                player.set_queue(songs).await;
            }
            Command::AddToQueue(song) => {
                player.add_to_queue(song).await;
            }
            Command::Seek(millis) => {
                player.seek(millis).await;
            }
            Command::SetVolume(volume) => {
                player.set_volume(volume).await;
            }
            Command::Pause => {
                player.pause().await;
            }
            Command::Resume => {
                player.play().await;
            }
            Command::Stop => {
                player.stop().await;
            }
            Command::Ended => {
                player.on_ended();
            }
            Command::Next => {
                player.go_next().await;
            }
            Command::Previous => {
                player.go_previous().await;
            }
            Command::GetCurrentStatus => {
                let current_status = player.get_current_status();
                if let Err(e) = receiver.respond(PlayerResponse::StatusResponse(current_status)) {
                    error!("Error sending player status");
                }
            }
            Command::Shutdown => {
                return;
            }
        }
        info!("Completed command");
    }
    info!("Request loop completed");
}
