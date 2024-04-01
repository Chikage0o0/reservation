use std::{env, path::Path};

use abi::{config::Config, utils::datetime_to_timestamp};
use reservation_service::RsvpService;
use sqlx::types::chrono::{DateTime, Utc};
use tokio_stream::StreamExt;

#[tokio::test]
async fn grpc_server_should_work() {
    migrate_revert();
    migrate_run();
    let config = load_service_config_from_env_file();
    let service = RsvpService::from_config(&config).await.unwrap();

    let listen = format!("{}:{}", config.server.host, config.server.port);

    let (stop_signal_tx, handler) = reservation_service::run(listen.parse().unwrap(), service)
        .await
        .unwrap();

    let client = tonic::transport::Channel::from_shared(format!(
        "http://{}:{}",
        config.server.host, config.server.port
    ))
    .unwrap()
    .connect()
    .await
    .unwrap();

    let mut client = abi::reservation_service_client::ReservationServiceClient::new(client);

    test(&mut client).await;

    stop_signal_tx.send(()).unwrap();

    handler.await.unwrap().unwrap();
    migrate_revert();
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

fn generation_reservation() -> abi::Reservation {
    let start = Utc::now();
    std::thread::sleep(std::time::Duration::from_millis(1));
    let end = Utc::now();
    assert!(start < end);
    abi::Reservation::new_pendding(
        "user",
        "resource",
        start.fixed_offset(),
        end.fixed_offset(),
        "note",
    )
}

fn migrate_run() {
    let cmd = std::process::Command::new("sqlx")
        .arg("migrate")
        .arg("run")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
        .output()
        .expect("failed to execute process");

    assert!(cmd.status.success());
}

fn migrate_revert() {
    loop {
        let cmd = std::process::Command::new("sqlx")
            .arg("migrate")
            .arg("revert")
            .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
            .output();
        match cmd {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8(output.stdout).unwrap();
                    if stdout.contains("No migrations available to revert") {
                        break;
                    }
                }
            }
            Err(_) => break,
        }
    }
}

async fn test(
    client: &mut abi::reservation_service_client::ReservationServiceClient<
        tonic::transport::Channel,
    >,
) {
    // Reserve
    let rsvp = abi::Reservation::new_pendding(
        "user",
        "resource",
        DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap(),
        DateTime::parse_from_rfc3339("2021-01-02T00:00:00Z").unwrap(),
        "note",
    );
    let request = tonic::Request::new(abi::ReserveRequest {
        reservation: Some(rsvp),
    });
    let response1 = client.reserve(request).await.unwrap();
    let request = tonic::Request::new(abi::GetRequest {
        id: response1.get_ref().reservation.as_ref().unwrap().id,
    });

    // Get reservation
    let response2 = client.get(request).await.unwrap();
    assert_eq!(
        response1.get_ref().reservation,
        response2.get_ref().reservation
    );

    // Confirm reservation
    let request = tonic::Request::new(abi::ConfirmRequest {
        id: response1.get_ref().reservation.as_ref().unwrap().id,
    });
    let response3 = client.confirm(request).await.unwrap();
    assert_eq!(
        response3.get_ref().reservation.as_ref().unwrap().status,
        abi::ReservationStatus::Confirmed as i32
    );

    // Generate 100 reservations
    for _ in 0..100 {
        let request = tonic::Request::new(abi::ReserveRequest {
            reservation: Some(generation_reservation()),
        });

        let _ = client.reserve(request).await.unwrap();
    }

    // Test filter with no status
    let request = tonic::Request::new(abi::FilterRequest {
        filter: Some(abi::ReservationFilter {
            page_size: 200,
            status: abi::ReservationStatus::Pending as i32,
            ..Default::default()
        }),
    });
    let response4 = client.filter(request).await.unwrap();
    assert_eq!(response4.get_ref().reservation.len(), 100);

    // Test filter with status
    let request = tonic::Request::new(abi::FilterRequest {
        filter: Some(abi::ReservationFilter {
            page_size: 200,
            status: abi::ReservationStatus::Confirmed as i32,
            ..Default::default()
        }),
    });
    let response5 = client.filter(request).await.unwrap();
    assert_eq!(response5.get_ref().reservation.len(), 1);

    // Test filter with pagination
    let request = tonic::Request::new(abi::FilterRequest {
        filter: Some(abi::ReservationFilter {
            page_size: 10,
            status: abi::ReservationStatus::Pending as i32,
            ..Default::default()
        }),
    });
    let response6 = client.filter(request).await.unwrap();
    assert_eq!(response6.get_ref().reservation.len(), 10);
    assert_eq!(response6.get_ref().pager.as_ref().unwrap().prev, None);
    for (i, id) in response6
        .get_ref()
        .reservation
        .iter()
        .map(|r| r.id)
        .enumerate()
    {
        // The first reservation is the 1 but it has been confirmed
        // So the first pending reservation is the 2
        assert_eq!(i + 2, id as usize);
    }

    // Test filter with pagination and  next cursor
    let request = tonic::Request::new(abi::FilterRequest {
        filter: Some(abi::ReservationFilter {
            page_size: 10,
            cursor: response6.get_ref().pager.as_ref().unwrap().next.unwrap(),
            status: abi::ReservationStatus::Pending as i32,
            ..Default::default()
        }),
    });
    let response7 = client.filter(request).await.unwrap();
    assert_eq!(
        response7.get_ref().pager.as_ref().unwrap().prev.unwrap() - 1,
        response6.get_ref().pager.as_ref().unwrap().next.unwrap()
    );
    assert_eq!(response7.get_ref().reservation.len(), 10);
    for (i, id) in response7
        .get_ref()
        .reservation
        .iter()
        .map(|r| r.id)
        .enumerate()
    {
        // The first reservation is the 1 but it has been confirmed
        // So the first pending reservation is the 2
        assert_eq!(i + 12, id as usize);
    }

    // Test filter with pagination and prev cursor
    let request = tonic::Request::new(abi::FilterRequest {
        filter: Some(abi::ReservationFilter {
            page_size: 10,
            cursor: response7.get_ref().pager.as_ref().unwrap().prev.unwrap(),
            status: abi::ReservationStatus::Pending as i32,
            is_prev: true,
            ..Default::default()
        }),
    });
    let response8 = client.filter(request).await.unwrap();
    assert_eq!(response8.get_ref().reservation.len(), 10);
    for (i, id) in response8
        .get_ref()
        .reservation
        .iter()
        .map(|r| r.id)
        .enumerate()
    {
        // The first reservation is the 1 but it has been confirmed
        // So the first pending reservation is the 2
        assert_eq!(i + 2, id as usize);
    }

    // Test filter with pagination with desc order
    let request = tonic::Request::new(abi::FilterRequest {
        filter: Some(abi::ReservationFilter {
            page_size: 10,
            status: abi::ReservationStatus::Pending as i32,
            is_desc: true,
            ..Default::default()
        }),
    });
    let response9 = client.filter(request).await.unwrap();
    assert_eq!(response9.get_ref().reservation.len(), 10);
    for (i, id) in response9
        .get_ref()
        .reservation
        .iter()
        .map(|r| r.id)
        .enumerate()
    {
        // The first reservation is the 1 but it has been confirmed
        // So the first pending reservation is the 2
        assert_eq!(101 - i, id as usize);
    }

    // Test filter with pagination with desc order and next cursor
    let request = tonic::Request::new(abi::FilterRequest {
        filter: Some(abi::ReservationFilter {
            page_size: 10,
            cursor: response9.get_ref().pager.as_ref().unwrap().next.unwrap(),
            status: abi::ReservationStatus::Pending as i32,
            is_desc: true,
            ..Default::default()
        }),
    });
    let response10 = client.filter(request).await.unwrap();
    assert_eq!(response10.get_ref().reservation.len(), 10);
    for (i, id) in response10
        .get_ref()
        .reservation
        .iter()
        .map(|r| r.id)
        .enumerate()
    {
        // The first reservation is the 1 but it has been confirmed
        // So the first pending reservation is the 2
        assert_eq!(91 - i, id as usize);
    }

    // Test filter with pagination with desc order and prev cursor
    let request = tonic::Request::new(abi::FilterRequest {
        filter: Some(abi::ReservationFilter {
            page_size: 10,
            cursor: response10.get_ref().pager.as_ref().unwrap().prev.unwrap(),
            status: abi::ReservationStatus::Pending as i32,
            is_desc: true,
            is_prev: true,
            ..Default::default()
        }),
    });
    let response11 = client.filter(request).await.unwrap();
    assert_eq!(response11.get_ref().reservation.len(), 10);
    for (i, id) in response11
        .get_ref()
        .reservation
        .iter()
        .map(|r| r.id)
        .enumerate()
    {
        // The first reservation is the 1 but it has been confirmed
        // So the first pending reservation is the 2
        assert_eq!(101 - i, id as usize);
    }

    // Test query
    let request = tonic::Request::new(abi::QueryRequest {
        query: Some(
            abi::ReservationQueryBuilder::default()
                .page_size(500)
                .end(datetime_to_timestamp(Utc::now()))
                .build()
                .unwrap(),
        ),
    });

    let mut response12 = client.query(request).await.unwrap();
    let rsvp = response12.get_mut();
    let mut count = 0;
    while let Some(Ok(r)) = rsvp.next().await {
        count += 1;
        assert_eq!(r.id, count as i64);
    }
}
