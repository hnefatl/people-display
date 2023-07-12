#![feature(never_type)]
#![feature(exhaustive_patterns)]

use clock_pb::clock_service_server::{ClockService, ClockServiceServer};
use clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};
use std::net::Ipv4Addr;
use tokio;

mod config;
mod homeassistant;
mod photo_manager;

pub struct ClockServer {
    homeassistant_connection_config: config::HomeAssistantConfig,
    person_ids: Vec<homeassistant::PersonId>,
    photo_manager: photo_manager::PhotoManager,
}

#[tonic::async_trait]
impl ClockService for ClockServer {
    async fn get_people_locations(
        &self,
        request: tonic::Request<GetPeopleLocationsRequest>,
    ) -> tonic::Result<tonic::Response<GetPeopleLocationsResponse>> {
        let snapshot = homeassistant::get_snapshot(
            &self.homeassistant_connection_config.host,
            self.homeassistant_connection_config.port,
            &self.homeassistant_connection_config.access_token,
            &self.person_ids,
        )
        .await
        .map_err(|e| tonic::Status::unavailable(format!("Failed to query home assistant: {e}")))?;

        Ok(tonic::Response::new(GetPeopleLocationsResponse {
            people: snapshot.people.iter().map()
        }))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ! {
    let config = config::get_config_from_environment_variables().unwrap();
    println!("Read config: {:?}", config);

    let addr = std::net::SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), 12733);

    println!("Starting server.");
    loop {
        let clock_service = ClockServiceServer::new(ClockServer {
            homeassistant_connection_config: config.homeassistant.clone(),
            person_ids: config.person_entity_ids.clone(),
            photo_manager: photo_manager::PhotoManager::new(config.photo_directory.clone()),
        });
        let serve = tonic::transport::Server::builder()
            .add_service(clock_service)
            .serve(addr);
        if let Err(e) = serve.await {
            println!("Server halted with error: {e:?}");
        } else {
            println!("Server halted silently, restarting.");
        }
    }
}
