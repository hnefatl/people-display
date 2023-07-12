#![feature(never_type)]
#![feature(exhaustive_patterns)]

use clock_pb::clock_service_server::{ClockService, ClockServiceServer};
use clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};
use std::net::Ipv4Addr;
use tokio;

mod config;
use config::CONFIG;
mod homeassistant;

pub struct ClockServer {
    person_ids: Vec<homeassistant::PersonId>,
}

#[tonic::async_trait]
impl ClockService for ClockServer {
    async fn get_people_locations(
        &self,
        request: tonic::Request<GetPeopleLocationsRequest>,
    ) -> tonic::Result<tonic::Response<GetPeopleLocationsResponse>> {
        let snapshot = homeassistant::get_snapshot(
            &CONFIG.home_assistant_host,
            CONFIG.home_assistant_port,
            &CONFIG.home_assistant_access_token,
            &self.person_ids,
        )
        .await
        .map_err(|e| tonic::Status::unavailable(format!("Failed to query home assistant: {e}")))?;

        Ok(tonic::Response::new(GetPeopleLocationsResponse {
            people: vec![],
        }))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ! {
    println!("Read config: {:?}", *CONFIG);
    let addr = std::net::SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED), 12733);

    let testing_entity_ids = vec!["person.keith".into()];
    let testing_snapshot = homeassistant::get_snapshot(
        &CONFIG.home_assistant_host,
        CONFIG.home_assistant_port,
        &CONFIG.home_assistant_access_token,
        &testing_entity_ids,
    )
    .await;
    println!("{testing_snapshot:?}");

    println!("Starting server.");
    loop {
        let serve = tonic::transport::Server::builder()
            .add_service(ClockServiceServer::new(ClockServer {
                person_ids: CONFIG.person_entity_ids.clone(),
            }))
            .serve(addr);
        if let Err(e) = serve.await {
            println!("Server halted with error: {e:?}");
        } else {
            println!("Server halted silently, restarting.");
        }
    }
}
