#![feature(never_type)]
#![feature(exhaustive_patterns)]

use clock_pb::clock_service_server::{ClockService, ClockServiceServer};
use clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};
use std::net::Ipv4Addr;
use tokio;

mod config;
use config::CONFIG;

type PersonId = String;

// Get all the states of the given people IDs that we can find.
async fn get_people_states(
    client: &mut hass_rs::HassClient,
    entity_ids: &Vec<PersonId>,
) -> hass_rs::HassResult<Vec<hass_rs::HassEntity>> {
    // Preprocess the entity ids to make membership tests faster.
    let entity_id_lookup = std::collections::HashSet::<_>::from_iter(entity_ids);

    let all_entity_states = client.get_states().await?;
    let mut relevant_people_states = vec![];
    for entity in all_entity_states {
        if entity_id_lookup.contains(&entity.entity_id) {
            relevant_people_states.push(entity);
        }
    }
    return Ok(relevant_people_states);
}

async fn open_hass_client() -> hass_rs::HassResult<hass_rs::HassClient> {
    let mut client =
        hass_rs::connect(&CONFIG.home_assistant_host, CONFIG.home_assistant_port).await?;
    client
        .auth_with_longlivedtoken(&CONFIG.home_assistant_access_token)
        .await?;
    return Ok(client);
}

pub struct ClockServer {}

#[tonic::async_trait]
impl ClockService for ClockServer {
    async fn get_people_locations(
        &self,
        request: tonic::Request<GetPeopleLocationsRequest>,
    ) -> tonic::Result<tonic::Response<GetPeopleLocationsResponse>> {
        let mut client = open_hass_client().await.map_err(|e| {
            tonic::Status::unavailable(format!("Failed to connect to home assistant instance: {e}"))
        })?;

        let person_ids = &request.into_inner().person_entity_ids;
        let person_states = get_people_states(&mut client, person_ids)
            .await
            .map_err(|e| {
                tonic::Status::unavailable(format!("Failed to query home assistant: {e}"))
            })?;

        Ok(tonic::Response::new(GetPeopleLocationsResponse {
            locations: person_states
                .into_iter()
                .map(|s| (s.entity_id, s.state))
                .collect(),
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
