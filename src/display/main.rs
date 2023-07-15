use std::sync::Arc;

use env_logger;
use gtk::gdk::gdk_pixbuf::Pixbuf;
use gtk::gdk::Texture;
use gtk::glib::{clone, MainContext, Priority};
use gtk::{prelude::*, Image};
use gtk::{Application, ApplicationWindow, Button};
use gtk4 as gtk;
use gtk4::glib;
use log;
use prost::encoding::bytes;
use tokio;

use lib::clock_pb;
use lib::clock_pb::clock_service_client::ClockServiceClient;
use lib::clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};
use lib::env_params::{get_env_variable, get_env_variable_with_default};

async fn get_snapshots(
    endpoint_uris: &Vec<tonic::transport::Uri>,
) -> Vec<GetPeopleLocationsResponse> {
    let mut responses = vec![];
    for endpoint in endpoint_uris {
        log::info!("Connecting to {}", endpoint);
        let mut client = ClockServiceClient::connect(endpoint.clone()).await.unwrap();
        log::info!("Connected");
        let rpc = client
            .get_people_locations(GetPeopleLocationsRequest {})
            .await
            .unwrap();
        let response = rpc.into_inner();

        log::info!("Got response: {response:?}");
        responses.push(response);
    }
    return responses;
}

fn bytes_to_image(bytes: &Vec<u8>) -> Result<Image, glib::Error> {
    Texture::from_bytes(&glib::Bytes::from_owned(bytes.clone()))
        .map(|t| Image::from_paintable(Some(&t)))
}

fn entities_to_image(
    person: &clock_pb::Person,
    zone: Option<&clock_pb::Zone>,
) -> Result<Image, String> {
    let person_image = person
        .photo_data
        .as_ref()
        .map(|b| bytes_to_image(&b))
        .transpose()
        .map_err(|e| format!("{e}"))?;
    let zone_image = zone
        .and_then(|z| z.photo_data.as_ref())
        .map(|b| bytes_to_image(&b))
        .transpose()
        .map_err(|e| format!("{e}"))?;

    let blank_pixbuf = Pixbuf::new(gtk::gdk_pixbuf::Colorspace::Rgb, false, 8, 100, 100)
        .ok_or("Unable to parse pixbuf")?;
    blank_pixbuf.fill(0);
    let blank_image = Image::from_pixbuf(Some(&blank_pixbuf));

    Ok(zone_image.unwrap_or(blank_image))
}

fn snapshot_to_images(snapshot: GetPeopleLocationsResponse) -> Vec<Image> {
    let zones: std::collections::HashMap<String, clock_pb::Zone> = snapshot
        .zones
        .into_iter()
        .map(|z| (z.id.clone(), z))
        .collect();
    let mut images = vec![];
    for person in snapshot.people {
        match entities_to_image(&person, zones.get(&person.zone_id)) {
            Ok(image) => images.push(image),
            Err(e) => log::error!("Failed to render {person:?}: {e}"),
        }
    }
    images
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let application = Application::builder()
        .application_id("clock.Display")
        .build();

    application.connect_activate(|app| {
        let endpoint_uris: Arc<Vec<tonic::transport::Uri>> =
            Arc::new(get_env_variable("ENDPOINTS").unwrap());
        let poll_interval: Arc<u16> =
            Arc::new(get_env_variable_with_default("POLL_INTERVAL", 60).unwrap());

        let layout = gtk::FlowBox::builder().homogeneous(false).build();
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Display")
            .fullscreened(true)
            .hexpand(true)
            .vexpand(true)
            .child(&layout)
            .build();

        let (sender, receiver) =
            MainContext::channel::<Vec<GetPeopleLocationsResponse>>(Priority::default());
        let main_context = MainContext::default();
        receiver.attach(
            None,
            clone!(@strong layout => @default-return Continue(true), move |responses| {
                // Clear the grid
                while let Some(child) = layout.first_child() {
                    layout.remove(&child);
                }
                // Add the new images
                for response in responses {
                    for image in snapshot_to_images(response) {
                        layout.set_hexpand(true);
                        layout.set_vexpand(true);
                        layout.append(&image);
                    }
                }
                Continue(true)
            }),
        );
        main_context.spawn_local(clone!(@strong sender, @strong endpoint_uris => async move {
            loop {
                let snapshots = get_snapshots(&endpoint_uris).await;
                match sender.send(snapshots) {
                    Ok(_) => log::info!("Received response from server"),
                    Err(e) => log::error!("Failed to send event to UI: {e}"),
                }
                tokio::time::sleep(std::time::Duration::from_secs(*poll_interval as u64)).await;
            }
        }));
        window.show()
    });

    application.run();

    Ok(())
}
