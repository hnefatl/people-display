use crate::config;
use crate::homeassistant;
use crate::photo_manager;

use lib::clock_pb;
use lib::clock_pb::clock_service_server::{ClockService, ClockServiceServer};
use lib::clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};
use lib::password::CheckPassword;

fn get_entity_photo(
    entity_id: &impl homeassistant::EntityId,
    photo_manager: &photo_manager::PhotoManager,
) -> Option<Vec<u8>> {
    match photo_manager.get_photo(entity_id) {
        Ok(data) => Some(data),
        Err(e) => {
            log::warn!("Unable to load photo for entity id '{entity_id}': {e}");
            None
        }
    }
}

pub struct ClockServer {
    homeassistant_connection_config: config::HomeAssistantConfig,
    person_ids: Vec<homeassistant::PersonId>,
    privacy_switch_entity_id: Option<homeassistant::InputBooleanId>,
    photo_manager: photo_manager::PhotoManager,
}
impl ClockServer {
    pub fn make_server(
        password: secstr::SecStr,
        homeassistant_connection_config: config::HomeAssistantConfig,
        person_ids: Vec<homeassistant::PersonId>,
        privacy_switch_entity_id: &Option<homeassistant::InputBooleanId>,
        photo_manager: photo_manager::PhotoManager,
    ) -> tonic::service::interceptor::InterceptedService<
        ClockServiceServer<ClockServer>,
        CheckPassword,
    > {
        let server = ClockServer {
            homeassistant_connection_config,
            person_ids,
            privacy_switch_entity_id: privacy_switch_entity_id.clone(),
            photo_manager,
        };
        ClockServiceServer::with_interceptor(server, CheckPassword::new(password))
    }

    async fn snapshot_to_response(
        &self,
        client: &homeassistant::Client,
        snapshot: homeassistant::Snapshot,
    ) -> GetPeopleLocationsResponse {
        let mut people = vec![];
        for person in snapshot.people {
            let photo_data: Option<Vec<u8>>;
            if let Some(pd) = get_entity_photo(&person.id, &self.photo_manager) {
                photo_data = Some(pd);
            } else {
                log::info!(
                    "No photo file for '{}', trying to fetch from HA",
                    &person.id
                );
                match client.get_photo(&person).await {
                    Ok(pd) => photo_data = pd,
                    Err(e) => {
                        log::error!("Failed to get photo for {}: {}", &person.id, e);
                        photo_data = None;
                    }
                }
            }

            people.push(clock_pb::Person {
                photo_data,
                id: person.id.to_string(),
                zone_id: person.zone_id.map(|id| id.to_string()),
            })
        }

        let privacy_enabled: bool;
        if let Some(id) = &self.privacy_switch_entity_id {
            match client.get_entity::<homeassistant::InputBoolean>(id).await {
                Ok(privacy_input_boolean) => privacy_enabled = privacy_input_boolean.into(),
                Err(e) => {
                    log::warn!(
                        "Unable to fetch {} from HA, assuming privacy is enabled: {e}",
                        &id
                    );
                    privacy_enabled = true;
                }
            }
        } else {
            privacy_enabled = false
        }

        let mut zones = vec![];
        if !privacy_enabled {
            for zone_id in snapshot.zones.keys() {
                zones.push(clock_pb::Zone {
                    photo_data: get_entity_photo(zone_id, &self.photo_manager),
                    id: zone_id.to_string(),
                })
            }
        }

        GetPeopleLocationsResponse { people, zones }
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
            Ok(client) => match homeassistant::get_snapshot(&client, &self.person_ids).await {
                Ok(snapshot) => {
                    log::trace!("Got snapshot: {snapshot:?}");

                    let response = self.snapshot_to_response(&client, snapshot).await;
                    log::trace!("Responding with: {response:?}");
                    Ok(tonic::Response::new(response))
                }
                Err(e) => {
                    log::error!("Failed to get snapshot from HA: {e}");
                    return Err(tonic::Status::unavailable(e.to_string()));
                }
            },
            Err(e) => {
                let status = tonic::Status::internal(e.to_string());
                log::error!("Responding with: {e}");
                return Err(status);
            }
        }
    }
}
