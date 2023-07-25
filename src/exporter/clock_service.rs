use crate::config;
use crate::homeassistant;
use crate::photo_manager;

use lib::clock_pb;
use lib::clock_pb::clock_service_server::{ClockService, ClockServiceServer};
use lib::clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};
use lib::password::CheckPassword;

use log;

fn get_photo<const P: homeassistant::PrefixType>(
    entity_id: &homeassistant::EntityId<P>,
    photo_manager: &photo_manager::PhotoManager,
) -> Option<Vec<u8>> {
    // Replace `.` with `_` so that setting a `.png`/`.jpg` extension is easier.
    let filename = entity_id.to_string().replace(".", "_");
    match photo_manager.get_photo(std::path::Path::new(&filename)) {
        Ok(data) => Some(data),
        Err(e) => {
            log::warn!("Unable to load photo for entity id '{entity_id}': {e}");
            None
        }
    }
}

async fn get_photo_fallback_to_homeassistant(
    person: &homeassistant::Person,
    photo_manager: &photo_manager::PhotoManager,
    client: &homeassistant::Client,
) -> Result<Option<Vec<u8>>, homeassistant::Error> {
    match get_photo(&person.id, photo_manager) {
        Some(photo) => Ok(Some(photo)),
        None => {
            log::info!(
                "No photo file for '{}', trying to fetch from HA",
                &person.id
            );
            let photo = client.get_photo(person).await?;
            if photo.is_none() {
                log::info!("No photo for {} available from HA", &person.id);
            }
            Ok(photo)
        }
    }
}

fn zone_to_proto(
    zone: homeassistant::Zone,
    photo_manager: &photo_manager::PhotoManager,
) -> clock_pb::Zone {
    clock_pb::Zone {
        photo_data: get_photo(&zone.id, photo_manager),
        id: zone.id.to_string(),
    }
}
async fn person_to_proto(
    person: homeassistant::Person,
    photo_manager: &photo_manager::PhotoManager,
    client: &homeassistant::Client,
) -> Result<clock_pb::Person, homeassistant::Error> {
    Ok(clock_pb::Person {
        photo_data: get_photo_fallback_to_homeassistant(&person, photo_manager, client).await?,
        id: person.id.to_string(),
        zone_id: person.zone_id.to_string(),
    })
}

pub struct ClockServer {
    homeassistant_connection_config: config::HomeAssistantConfig,
    person_ids: Vec<homeassistant::PersonId>,
    photo_manager: photo_manager::PhotoManager,
}
impl ClockServer {
    pub fn make_server(
        password: secstr::SecStr,
        homeassistant_connection_config: config::HomeAssistantConfig,
        person_ids: Vec<homeassistant::PersonId>,
        photo_manager: photo_manager::PhotoManager,
    ) -> tonic::service::interceptor::InterceptedService<
        ClockServiceServer<ClockServer>,
        CheckPassword,
    > {
        let server = ClockServer {
            homeassistant_connection_config,
            person_ids,
            photo_manager,
        };
        ClockServiceServer::with_interceptor(server, CheckPassword::new(password))
    }
}

#[tonic::async_trait]
impl ClockService for ClockServer {
    async fn get_people_locations(
        &self,
        _: tonic::Request<GetPeopleLocationsRequest>,
    ) -> tonic::Result<tonic::Response<GetPeopleLocationsResponse>> {
        log::info!("Got request");
        let maybe_client = homeassistant::Client::new(
            &self.homeassistant_connection_config.access_token,
            &self.homeassistant_connection_config.endpoint,
        );
        match maybe_client {
            Ok(client) => {
                let snapshot = homeassistant::get_snapshot(&client, &self.person_ids).await;

                let mut people = vec![];
                for person in snapshot.people {
                    match person_to_proto(person.clone(), &self.photo_manager, &client).await {
                        Ok(person_proto) => {
                            people.push(person_proto);
                        }
                        Err(e) => log::error!("Failed to get photo for {}: {}", &person.id, e),
                    }
                }
                let zones = snapshot
                    .zones
                    .into_values()
                    .map(|z| zone_to_proto(z, &self.photo_manager))
                    .collect();

                let response = GetPeopleLocationsResponse { people, zones };
                log::trace!("Responding with: {response:?}");
                Ok(tonic::Response::new(response))
            }
            Err(e) => {
                let status = tonic::Status::internal(e.to_string());
                log::error!("Responding with: {e}");
                return Err(status);
            }
        }
    }
}
