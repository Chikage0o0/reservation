use std::path::Path;

use abi::config::Config;
use reservation_service::RsvpService;
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};

#[tokio::main]
async fn main() {
    let config_path = std::env::var("RERESERVE_CONFIG").unwrap_or_else(|_| {
        let p1 = Path::new("./config.toml");
        let path = shellexpand::tilde("~/.config/rereserve/config.toml").to_string();
        let p2 = Path::new(&path);
        let p3 = Path::new("/etc/rereserve/config.toml");

        let path = match (p1.exists(), p2.exists(), p3.exists()) {
            (true, _, _) => p1,
            (_, true, _) => p2,
            (_, _, true) => p3,
            _ => panic!("Config file not found"),
        };

        println!("Using config file: {:?}", path.display());

        path.to_str().unwrap().to_string()
    });

    let config = Config::load(config_path).unwrap();
    let service = RsvpService::from_config(&config).await.unwrap();

    let listen = format!("{}:{}", config.server.host, config.server.port);

    println!("Listening on {}", listen);
    println!("Press Ctrl-C to stop");

    let mut signals = Signals::new([SIGINT, SIGTERM]).unwrap();

    let (stop_signal_tx, handler) = reservation_service::run(listen.parse().unwrap(), service)
        .await
        .unwrap();

    if signals.forever().next().is_some() {
        println!("\nReceived signal, exiting");
        stop_signal_tx.send(()).unwrap();
    }

    handler.await.unwrap().unwrap();
}
