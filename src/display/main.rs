use env_logger;
use log;
use tokio;

use lib::clock_pb::clock_service_client::ClockServiceClient;
use lib::clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};
use lib::env_params::get_env_variable;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let endpoint_uris: Vec<tonic::transport::Uri> = get_env_variable("ENDPOINTS").unwrap();

    log::info!("Connecting to {}", endpoint_uris[0]);
    let mut client = ClockServiceClient::connect(endpoint_uris[0].clone())
        .await
        .unwrap();
    log::info!("Sending request");
    let rpc = client
        .get_people_locations(GetPeopleLocationsRequest {})
        .await
        .unwrap();
    let response = rpc.into_inner();

    log::info!("Got response: {response:?}");
    Ok(())
}
