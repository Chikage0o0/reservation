mod service;

use abi::Reservation;

use abi::reservation_service_server::ReservationServiceServer;
use anyhow::Result;
pub use service::RsvpService;
use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    sync::{mpsc::Receiver, oneshot::Sender},
    task::JoinHandle,
};
use tokio_stream::Stream;
use tonic::Status;

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

pub async fn run(
    listen: SocketAddr,
    service: RsvpService,
) -> Result<(Sender<()>, JoinHandle<Result<(), tonic::transport::Error>>)> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    let handler = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(ReservationServiceServer::new(service))
            .serve_with_shutdown(listen, async {
                rx.await.ok();
            })
            .await
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    if handler.is_finished() {
        let error = handler.await.unwrap_err();
        Err(error.into())
    } else {
        Ok((tx, handler))
    }
}

#[derive(Debug)]
pub struct TonicReceiverStream<T> {
    inner: Receiver<Result<T, abi::Error>>,
}

impl<T> TonicReceiverStream<T> {
    pub fn new(recv: Receiver<Result<T, abi::Error>>) -> Self {
        Self { inner: recv }
    }

    /// Get back the inner `Receiver`.
    pub fn into_inner(self) -> Receiver<Result<T, abi::Error>> {
        self.inner
    }

    /// Closes the receiving half of a channel without dropping it.
    ///
    /// This prevents any further messages from being sent on the channel while
    /// still enabling the receiver to drain messages that are buffered. Any
    /// outstanding [`Permit`] values will still be able to send messages.
    ///
    /// To guarantee no messages are dropped, after calling `close()`, you must
    /// receive all items from the stream until `None` is returned.
    ///
    /// [`Permit`]: struct@tokio::sync::mpsc::Permit
    pub fn close(&mut self) {
        self.inner.close();
    }
}

impl<T> Stream for TonicReceiverStream<T> {
    type Item = Result<T, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_recv(cx).map_err(|e| e.into())
    }
}
