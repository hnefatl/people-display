#![allow(incomplete_features)]
#![feature(unsized_const_params)]

use std::net::Ipv4Addr;

mod clock_service;
mod config;
mod homeassistant;
mod homeassistant_types;
mod photo_manager;
mod web_ui;

#[tokio::main(flavor = "multi_thread", worker_threads = 5)]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = config::get_config_from_environment_variables().unwrap();
    log::info!("Read config: {:?}", config);

    let addr = std::net::SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);

    let status = run(&config, addr).await;
    log::info!("Server halted with error: {status:?}\nRestarting after delay.");
}

async fn run(config: &config::Config, addr: std::net::SocketAddr) -> anyhow::Result<()> {
    let clock_service = clock_service::ClockServer::make_server(
        config.password.clone(),
        config.homeassistant.clone(),
        config.person_entity_ids.clone(),
        &config.privacy_switch_entity_id,
        photo_manager::PhotoManager::new(config.photo_directory.clone()),
    );
    let clock_server = tonic::transport::Server::builder().add_service(clock_service);
    let rocket = web_ui::rocket(config.clone()).ignite().await?;
    let rocket_shutdown = rocket.shutdown();

    // Handle mutual graceful shutdown of both servers.
    let clock_server_task = async move || {
        log::info!("Starting ClockServer on {addr}");
        let status = clock_server
            .serve_with_shutdown(addr, rocket_shutdown.clone())
            .await;
        log::info!("ClockServer stopped with status: {status:?}");
        rocket_shutdown.notify();
        status
    };

    let (rocket_status, clock_server_status) = tokio::join!(
        tokio::spawn(rocket.launch()),
        tokio::spawn(clock_server_task())
    );
    rocket_status??;
    clock_server_status??;
    Ok(())
}
