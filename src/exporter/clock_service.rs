use crate::config;
use crate::homeassistant;
use crate::photo_manager;

use homeassistant::Entity;
use lib::clock_pb;
use lib::clock_pb::clock_service_server::{ClockService, ClockServiceServer};
use lib::clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};

use log;

fn get_photo<const P: homeassistant::PrefixType>(
    photo_manager: &photo_manager::PhotoManager,
    entity_id: &homeassistant::EntityId<P>,
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

trait ToProto<T> {
    fn to_proto(self, photo_manager: &photo_manager::PhotoManager) -> T;
}
impl ToProto<clock_pb::Person> for homeassistant::Person {
    fn to_proto(self, photo_manager: &photo_manager::PhotoManager) -> clock_pb::Person {
        clock_pb::Person {
            photo_data: get_photo(photo_manager, &self.id),
            id: self.id.to_string(),
            name: self.get_friendly_name(),
            zone_id: self.zone_id.to_string(),
        }
    }
}
impl ToProto<clock_pb::Zone> for homeassistant::Zone {
    fn to_proto(self, photo_manager: &photo_manager::PhotoManager) -> clock_pb::Zone {
        clock_pb::Zone {
            photo_data: get_photo(photo_manager, &self.id),
            id: self.id.to_string(),
            name: self.get_friendly_name(),
        }
    }
}

pub struct ClockServer {
    homeassistant_connection_config: config::HomeAssistantConfig,
    person_ids: Vec<homeassistant::PersonId>,
    photo_manager: photo_manager::PhotoManager,
}
impl ClockServer {
    pub fn make_server(
        homeassistant_connection_config: config::HomeAssistantConfig,
        person_ids: Vec<homeassistant::PersonId>,
        photo_manager: photo_manager::PhotoManager,
    ) -> ClockServiceServer<ClockServer> {
        ClockServiceServer::new(ClockServer {
            homeassistant_connection_config,
            person_ids,
            photo_manager,
        })
    }
}

#[tonic::async_trait]
impl ClockService for ClockServer {
    async fn get_people_locations(
        &self,
        _: tonic::Request<GetPeopleLocationsRequest>,
    ) -> tonic::Result<tonic::Response<GetPeopleLocationsResponse>> {
        log::info!("Got request");
        match homeassistant::Client::new(
            &self.homeassistant_connection_config.access_token,
            &self.homeassistant_connection_config.endpoint,
        ) {
            Ok(client) => {
                let snapshot = homeassistant::get_snapshot(&client, &self.person_ids).await;

                let result = Ok(tonic::Response::new(GetPeopleLocationsResponse {
                    people: snapshot
                        .people
                        .into_iter()
                        .map(|p| p.to_proto(&self.photo_manager))
                        .collect(),
                    zones: snapshot
                        .zones
                        .into_values()
                        .map(|z| z.to_proto(&self.photo_manager))
                        .collect(),
                }));

                log::trace!("Responding with: {result:?}");
                return result;
            }
            Err(e) => {
                let status = tonic::Status::internal(e.to_string());
                log::error!("Responding with: {e}");
                return Err(status);
            }
        }
    }
}