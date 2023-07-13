#![feature(never_type)]
#![feature(exhaustive_patterns)]

use env_logger;
use log;
use std::net::Ipv4Addr;
use tokio;

mod clock_service;
mod config;
mod homeassistant;
mod photo_manager;

#[tokio::main]
async fn main() -> ! {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = config::get_config_from_environment_variables().unwrap();
    log::info!("Read config: {:?}", config);

    let addr = std::net::SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);

    log::info!("Starting server on {addr}");
    loop {
        let clock_service = clock_service::ClockServer::make_server(
            config.homeassistant.clone(),
            config.person_entity_ids.clone(),
            photo_manager::PhotoManager::new(config.photo_directory.clone()),
        );
        let server = tonic::transport::Server::builder().add_service(clock_service);
        if let Err(e) = server.serve(addr).await {
            log::info!("Server halted with error: {e:?}");
        } else {
            log::info!("Server halted silently, restarting.");
        }
    }
}
