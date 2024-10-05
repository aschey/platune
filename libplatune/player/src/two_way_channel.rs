use flume::{Receiver, RecvError, SendError, Sender, TryRecvError};
use tokio::sync::oneshot::{Sender as OneShotSender, channel as oneshot_channel};

pub(crate) fn two_way_channel<TIn, TOut>() -> (TwoWaySender<TIn, TOut>, TwoWayReceiver<TIn, TOut>) {
    let (main_tx, main_rx) = flume::unbounded();
    (TwoWaySender::new(main_tx), TwoWayReceiver::new(main_rx))
}

#[derive(Clone, Debug)]
pub(crate) struct TwoWaySender<TIn, TOut> {
    main_tx: Sender<(TIn, Option<OneShotSender<TOut>>)>,
}

#[derive(Debug)]
pub(crate) struct TwoWayReceiver<TIn, TOut> {
    main_rx: Receiver<(TIn, Option<OneShotSender<TOut>>)>,
    oneshot: Option<OneShotSender<TOut>>,
}

type Responder<TIn, TOut> = (TIn, Option<OneShotSender<TOut>>);

impl<TIn, TOut> TwoWaySender<TIn, TOut> {
    pub(crate) fn new(main_tx: Sender<Responder<TIn, TOut>>) -> Self {
        Self { main_tx }
    }

    pub(crate) async fn send_async(
        &self,
        message: TIn,
    ) -> Result<(), SendError<Responder<TIn, TOut>>> {
        self.main_tx.send_async((message, None)).await
    }

    pub(crate) fn send(&self, message: TIn) -> Result<(), SendError<Responder<TIn, TOut>>> {
        self.main_tx.send((message, None))
    }

    pub(crate) async fn get_response(&self, message: TIn) -> Result<TOut, String> {
        let (oneshot_tx, oneshot_rx) = oneshot_channel();
        self.main_tx
            .send_async((message, Some(oneshot_tx)))
            .await
            .map_err(|e| format!("Error sending oneshot response{e:?}"))?;
        oneshot_rx
            .await
            .map_err(|e| format!("Error receiving oneshot response {e:?}"))
    }
}

impl<TIn, TOut> TwoWayReceiver<TIn, TOut> {
    pub(crate) fn new(main_rx: flume::Receiver<Responder<TIn, TOut>>) -> Self {
        Self {
            main_rx,
            oneshot: None,
        }
    }

    pub(crate) async fn recv_async(&mut self) -> Result<TIn, RecvError> {
        match self.main_rx.recv_async().await {
            Ok((res, oneshot)) => {
                self.oneshot = oneshot;
                Ok(res)
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) fn try_recv(&mut self) -> Result<TIn, TryRecvError> {
        match self.main_rx.try_recv() {
            Ok((res, oneshot)) => {
                self.oneshot = oneshot;
                Ok(res)
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) fn recv(&mut self) -> Result<TIn, RecvError> {
        match self.main_rx.recv() {
            Ok((res, oneshot)) => {
                self.oneshot = oneshot;
                Ok(res)
            }
            Err(e) => Err(e),
        }
    }

    pub(crate) fn respond(&mut self, response: TOut) -> Result<(), TOut> {
        if let Some(oneshot) = self.oneshot.take() {
            oneshot.send(response)
        } else {
            Ok(())
        }
    }
}
