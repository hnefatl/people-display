#![feature(never_type)]
#![feature(exhaustive_patterns)]

use clock_pb::clock_service_server::{ClockService, ClockServiceServer};
use clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};
use std::net::Ipv4Addr;
use tokio;

mod config;
use config::CONFIG;
mod homeassistant;

pub struct ClockServer {}

#[tonic::async_trait]
impl ClockService for ClockServer {
    async fn get_people_locations(
        &self,
        request: tonic::Request<GetPeopleLocationsRequest>,
    ) -> tonic::Result<tonic::Response<GetPeopleLocationsResponse>> {
        let person_ids: &Vec<homeassistant::PersonId> = &request.into_inner().person_entity_ids;
        let snapshot = homeassistant::get_snapshot(
            &CONFIG.home_assistant_host,
            CONFIG.home_assistant_port,
            &CONFIG.home_assistant_access_token,
            person_ids,
        )
        .await
        .map_err(|e| tonic::Status::unavailable(format!("Failed to query home assistant: {e}")))?;

        Ok(tonic::Response::new(GetPeopleLocationsResponse {
            locations: std::collections::HashMap::new(),
        }))
    }
}

async fn start_server(addr: std::net::SocketAddr) -> Result<(), tonic::transport::Error> {
    tonic::transport::Server::builder()
        .add_service(ClockServiceServer::new(ClockServer {}))
        .serve(addr)
        .await
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ! {
    println!("Read config: {:?}", *CONFIG);
    let addr = std::net::SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), 12733);

    println!("Starting server.");
    loop {
        if let Err(e) = start_server(addr).await {
            println!("Server halted with error: {e:?}");
        } else {
            println!("Server halted silently, restarting.");
        }
    }
}
