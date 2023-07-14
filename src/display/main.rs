use env_logger;
use gtk::gdk::Texture;
use gtk::glib::{clone, MainContext, Priority};
use gtk::{prelude::*, Image};
use gtk::{Application, ApplicationWindow, Button};
use gtk4 as gtk;
use gtk4::glib;
use log;
use tokio;

use lib::clock_pb::clock_service_client::ClockServiceClient;
use lib::clock_pb::{GetPeopleLocationsRequest, GetPeopleLocationsResponse};
use lib::env_params::get_env_variable;

async fn test_server() {
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let application = Application::builder()
        .application_id("clock.Display")
        .build();

    application.connect_activate(|app| {
        let layout = gtk::FlowBox::builder().homogeneous(false).build();
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Display")
            .fullscreened(true)
            .hexpand(true)
            .vexpand(true)
            .child(&layout)
            .build();

        let button = Button::builder()
            .label("Click me!")
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .hexpand(true)
            .vexpand(true)
            .build();
        button.connect_clicked(|_| {
            println!("foo");
        });
        let (sender, receiver) =
            MainContext::channel::<GetPeopleLocationsResponse>(Priority::default());
        let main_context = MainContext::default();
        receiver.attach(
            None,
            clone!(@weak layout => @default-return Continue(true), move |response| {
                //let photo_data = response.people[0].photo_data.clone().unwrap();
                let photo_data = vec![];
                match Texture::from_bytes(&glib::Bytes::from_owned(photo_data)) {
                    Ok(texture) => layout.append(&Image::from_paintable(Some(&texture))),
                    Err(e) => log::error!("Failed to parse bytes: {e}"),
                }
                Continue(true)
            }),
        );
        main_context.spawn_local(clone!(@strong sender => async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            match sender.send(GetPeopleLocationsResponse{people: vec![], zones: vec![]}) {
                Ok(_) => log::info!("Receive response from server"),
                Err(e) => log::error!("Failed to send event to UI: {e}"),
            }
        }));
        layout.append(&button);
        window.show()
    });

    application.run();

    Ok(())
}
