mod service;

use abi::Reservation;
use anyhow::Result;

pub use service::RsvpService;
use std::pin::Pin;
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
            .join(".devcontainer/.env");
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
