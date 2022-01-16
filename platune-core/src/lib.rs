use crossbeam_channel::{unbounded, Receiver, SendError, Sender, TryRecvError};

mod audio_processor;
mod decoder;
mod dto;
mod event_loop;
mod http_stream_reader;
mod output;
mod player;
mod source;

pub mod platune_player {
    use crossbeam_channel::{unbounded, Sender};
    use std::thread;
    use std::time::Duration;
    use tokio::sync::broadcast;
    use tracing::{error, warn};

    pub use crate::dto::audio_status::AudioStatus;
    pub use crate::dto::player_event::PlayerEvent;
    pub use crate::dto::player_state::PlayerState;
    pub use crate::dto::player_status::PlayerStatus;
    use crate::event_loop::{
        decode_loop, CurrentTime, DecoderCommand, DecoderResponse, PlayerResponse,
    };
    use crate::{dto::command::Command, event_loop::main_loop};
    use crate::{two_way_channel, two_way_channel_async, TwoWaySender, TwoWaySenderAsync};
    use std::fs::remove_file;

    #[derive(Debug, Clone)]
    pub struct PlayerError(String);

    #[derive(Debug)]
    pub struct PlatunePlayer {
        cmd_sender: TwoWaySenderAsync<Command, PlayerResponse>,
        decoder_tx: TwoWaySender<DecoderCommand, DecoderResponse>,
        event_tx: broadcast::Sender<PlayerEvent>,
    }

    impl Default for PlatunePlayer {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PlatunePlayer {
        pub fn new() -> Self {
            Self::clean_temp_files();

            let (event_tx, _) = broadcast::channel(32);
            let event_tx_ = event_tx.clone();
            let (cmd_tx, cmd_rx) = two_way_channel_async();
            let cmd_tx_ = cmd_tx.clone();
            let (queue_tx, queue_rx) = crossbeam_channel::bounded(2);
            let queue_rx_ = queue_rx.clone();
            let (decoder_tx, decoder_rx) = two_way_channel();
            let decoder_tx_ = decoder_tx.clone();

            let main_loop_fn =
                async move { main_loop(cmd_rx, event_tx_, queue_tx, queue_rx, decoder_tx_).await };
            let decoder_fn = || decode_loop(queue_rx_, decoder_rx, cmd_tx_);

            tokio::spawn(main_loop_fn);
            thread::spawn(decoder_fn);

            PlatunePlayer {
                cmd_sender: cmd_tx,
                event_tx,
                decoder_tx,
            }
        }

        fn clean_temp_files() {
            match std::env::temp_dir().read_dir() {
                Ok(temp_dir) => {
                    for entry in temp_dir.flatten() {
                        if entry
                            .file_name()
                            .to_string_lossy()
                            .starts_with("platunecache")
                        {
                            if let Err(e) = remove_file(entry.path()) {
                                error!("Error removing temp file {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading temp dir {:?}", e);
                }
            }
        }

        pub fn subscribe(&self) -> broadcast::Receiver<PlayerEvent> {
            self.event_tx.subscribe()
        }

        pub async fn set_queue(&self, queue: Vec<String>) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::SetQueue(queue))
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn add_to_queue(&self, songs: Vec<String>) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::AddToQueue(songs))
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn seek(&self, time: Duration) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Seek(time))
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn get_current_status(&self) -> Result<PlayerStatus, PlayerError> {
            let track_status = match self
                .cmd_sender
                .get_response(Command::GetCurrentStatus)
                .await
            {
                Ok(PlayerResponse::StatusResponse(track_status)) => track_status,
                Err(e) => return Err(PlayerError(format!("{:?}", e))),
            };

            match track_status.status {
                AudioStatus::Stopped => Ok(PlayerStatus {
                    current_time: CurrentTime {
                        current_time: None,
                        retrieval_time: None,
                    },
                    track_status,
                }),
                _ => {
                    match self
                        .decoder_tx
                        .get_response(DecoderCommand::GetCurrentTime)
                        .await
                        .unwrap()
                    {
                        DecoderResponse::CurrentTimeResponse(current_time) => Ok(PlayerStatus {
                            current_time,
                            track_status,
                        }),
                        _ => unreachable!(),
                    }
                }
            }
        }

        pub async fn stop(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Stop)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn set_volume(&self, volume: f64) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::SetVolume(volume))
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn pause(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Pause)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn resume(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Resume)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn next(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Next)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn previous(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Previous)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn join(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Shutdown)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }
    }

    impl Drop for PlatunePlayer {
        fn drop(&mut self) {
            if let Err(e) = self.cmd_sender.try_send(Command::Shutdown) {
                // Receiver may already be terminated so this may not be an error
                warn!("Unable to send shutdown command {:?}", e);
            }
        }
    }
}

pub(crate) trait Channel<T> {
    fn send(msg: T);
}

pub(crate) fn two_way_channel<TIn, TOut>() -> (TwoWaySender<TIn, TOut>, TwoWayReceiver<TIn, TOut>) {
    let (main_tx, main_rx) = unbounded();
    (TwoWaySender::new(main_tx), TwoWayReceiver::new(main_rx))
}

#[derive(Clone, Debug)]
pub(crate) struct TwoWaySender<TIn, TOut> {
    main_tx: Sender<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>,
}

pub(crate) struct TwoWayReceiver<TIn, TOut> {
    main_rx: Receiver<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>,
    oneshot: Option<tokio::sync::oneshot::Sender<TOut>>,
}

impl<TIn, TOut> TwoWaySender<TIn, TOut> {
    fn new(main_tx: Sender<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>) -> Self {
        Self { main_tx }
    }

    async fn send(
        &self,
        message: TIn,
    ) -> Result<(), SendError<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>> {
        self.main_tx.send((message, None))
    }

    async fn get_response(
        &self,
        message: TIn,
    ) -> Result<TOut, tokio::sync::oneshot::error::RecvError> {
        let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();
        self.main_tx.send((message, Some(oneshot_tx))).unwrap();
        oneshot_rx.await
    }
}

impl<TIn, TOut> TwoWayReceiver<TIn, TOut> {
    fn new(main_rx: Receiver<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>) -> Self {
        Self {
            main_rx,
            oneshot: None,
        }
    }

    fn recv(&mut self) -> TIn {
        let (res, oneshot) = self.main_rx.recv().unwrap();
        self.oneshot = oneshot;
        res
    }

    fn try_recv(&mut self) -> Result<TIn, TryRecvError> {
        match self.main_rx.try_recv() {
            Ok((res, oneshot)) => {
                self.oneshot = oneshot;
                Ok(res)
            }
            Err(e) => Err(e),
        }
    }

    fn respond(&mut self, response: TOut) -> Result<(), TOut> {
        if let Some(oneshot) = self.oneshot.take() {
            oneshot.send(response)
        } else {
            Ok(())
        }
    }
}

pub(crate) fn two_way_channel_async<TIn, TOut>(
) -> (TwoWaySenderAsync<TIn, TOut>, TwoWayReceiverAsync<TIn, TOut>) {
    let (main_tx, main_rx) = tokio::sync::mpsc::channel(32);
    (
        TwoWaySenderAsync::new(main_tx),
        TwoWayReceiverAsync::new(main_rx),
    )
}

#[derive(Clone, Debug)]
pub(crate) struct TwoWaySenderAsync<TIn, TOut> {
    main_tx: tokio::sync::mpsc::Sender<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>,
}

pub(crate) struct TwoWayReceiverAsync<TIn, TOut> {
    main_rx: tokio::sync::mpsc::Receiver<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>,
    oneshot: Option<tokio::sync::oneshot::Sender<TOut>>,
}

impl<TIn, TOut> TwoWaySenderAsync<TIn, TOut> {
    fn new(
        main_tx: tokio::sync::mpsc::Sender<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>,
    ) -> Self {
        Self { main_tx }
    }

    async fn send(
        &self,
        message: TIn,
    ) -> Result<
        (),
        tokio::sync::mpsc::error::SendError<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>,
    > {
        self.main_tx.send((message, None)).await
    }

    fn try_send(
        &self,
        message: TIn,
    ) -> Result<
        (),
        tokio::sync::mpsc::error::TrySendError<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>,
    > {
        self.main_tx.try_send((message, None))
    }

    async fn get_response(
        &self,
        message: TIn,
    ) -> Result<TOut, tokio::sync::oneshot::error::RecvError> {
        let (oneshot_tx, oneshot_rx) = tokio::sync::oneshot::channel();
        self.main_tx.send((message, Some(oneshot_tx))).await;
        oneshot_rx.await
    }
}

impl<TIn, TOut> TwoWayReceiverAsync<TIn, TOut> {
    fn new(
        main_rx: tokio::sync::mpsc::Receiver<(TIn, Option<tokio::sync::oneshot::Sender<TOut>>)>,
    ) -> Self {
        Self {
            main_rx,
            oneshot: None,
        }
    }

    async fn recv(&mut self) -> Option<TIn> {
        match self.main_rx.recv().await {
            Some((res, oneshot)) => {
                self.oneshot = oneshot;
                Some(res)
            }
            None => None,
        }
    }

    fn try_recv(&mut self) -> Result<TIn, tokio::sync::mpsc::error::TryRecvError> {
        match self.main_rx.try_recv() {
            Ok((res, oneshot)) => {
                self.oneshot = oneshot;
                Ok(res)
            }
            Err(e) => Err(e),
        }
    }

    fn respond(&mut self, response: TOut) -> Result<(), TOut> {
        if let Some(oneshot) = self.oneshot.take() {
            oneshot.send(response)
        } else {
            Ok(())
        }
    }
}
