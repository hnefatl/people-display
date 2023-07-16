use std::sync::mpsc;
use std::time::Duration;

use lib::clock_pb;
use lib::clock_pb::clock_service_client::ClockServiceClient;
use lib::clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};

pub struct Snapshot {
    pub people: Vec<clock_pb::Person>,
    pub zones: std::collections::HashMap<String, clock_pb::Zone>,
}
impl From<GetPeopleLocationsResponse> for Snapshot {
    fn from(response: GetPeopleLocationsResponse) -> Self {
        Snapshot {
            people: response.people,
            zones: response
                .zones
                .into_iter()
                .map(|z| (z.id.clone(), z))
                .collect(),
        }
    }
}
pub type EndpointSnapshots = Vec<Snapshot>;

pub struct SnapshotManager {
    snapshot_interval: Duration,
    endpoint_uris: Vec<tonic::transport::Uri>,
    tx: mpsc::Sender<EndpointSnapshots>,
}
impl SnapshotManager {
    pub async fn initialise(
        snapshot_interval: Duration,
        endpoint_uris: &Vec<tonic::transport::Uri>,
    ) -> (Self, mpsc::Receiver<EndpointSnapshots>) {
        let (tx, rx) = mpsc::channel();
        let snapshot_manager = SnapshotManager {
            snapshot_interval,
            endpoint_uris: endpoint_uris.clone(),
            tx,
        };
        return (snapshot_manager, rx);
    }

    pub async fn start_loop(mut self) {
        loop {
            match self.update_snapshots().await {
                // Other end of the pipe has already closed, just terminate.
                Ok(true) => break,
                Err(e) => log::error!("{}", e),
                _ => (),
            }
            tokio::time::sleep(self.snapshot_interval).await;
        }
    }

    async fn update_snapshots(&mut self) -> Result<bool, String> {
        let mut snapshots = vec![];
        for endpoint in &self.endpoint_uris {
            log::info!("Connecting to {}", endpoint);
            let mut client = ClockServiceClient::connect(endpoint.clone())
                .await
                .map_err(|e| format!("Failed to connect: {e}"))?;
            log::info!("Connected");
            let rpc = client
                .get_people_locations(GetPeopleLocationsRequest {})
                .await
                .map_err(|s| format!("Bad response from server: {s}"))?;
            let response = rpc.into_inner();

            log::trace!("Got response: {response:?}");
            snapshots.push(response.into());
        }
        let has_hung_up = self.tx.send(snapshots).is_err();
        Ok(has_hung_up)
    }
}
