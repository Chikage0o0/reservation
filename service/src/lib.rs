mod service;

use abi::Reservation;

pub use service::RsvpService;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::mpsc::Receiver;
use tokio_stream::Stream;
use tonic::Status;

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;

#[cfg(test)]
mod test {
    use std::{env, path::Path};

    use abi::{config::Config, reservation_service_server::ReservationServiceServer};

    use super::*;

    #[tokio::test]
    async fn test_start_service() {
        let config = load_service_config_from_env_file();
        let service = RsvpService::from_config(&config).await.unwrap();

        let svc = ReservationServiceServer::new(service);
        let listen = format!("{}:{}", config.server.host, config.server.port);

        let (tx, rx) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(svc)
                .serve_with_shutdown(listen.parse().unwrap(), async {
                    rx.await.ok();
                })
                .await
                .unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let _client = tonic::transport::Channel::from_shared(format!(
            "http://{}:{}",
            config.server.host, config.server.port
        ))
        .unwrap()
        .connect()
        .await
        .unwrap();

        tx.send(()).unwrap();

        server.await.unwrap();
    }

    // Load service configuration from .env file
    fn load_service_config_from_env_file() -> Config {
        let env_file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(".env");
        dotenvy::from_path(env_file).unwrap();
        const HOST: &str = "127.0.0.1";
        let port = find_free_port(HOST);

        Config {
            db: abi::config::DbConfig {
                host: env::var("POSTGRES_HOSTNAME").unwrap(),
                port: env::var("POSTGRES_PORT").unwrap().parse().unwrap(),
                user: env::var("POSTGRES_USER").unwrap(),
                password: env::var("POSTGRES_PASSWORD").unwrap(),
                database: env::var("POSTGRES_DB").unwrap(),
            },
            server: abi::config::ServerConfig {
                host: HOST.to_string(),
                port,
            },
        }
    }

    // Find a free port to use
    fn find_free_port(host: &str) -> u16 {
        for port in 10000..65535 {
            if let Ok(listener) = std::net::TcpListener::bind((host, port)) {
                drop(listener);
                return port;
            }
        }
        panic!("No free port found");
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
