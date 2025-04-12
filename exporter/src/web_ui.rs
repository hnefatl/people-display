use rocket::form::{Contextual, Form};
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::status::{self, Custom};
use rocket::{get, routes, Build, Rocket, State};
use rocket::{http, post, FromForm};

use rocket_dyn_templates::Template;

use crate::homeassistant::{EntityId, ZoneId};
use crate::photo_manager::PhotoManager;
use crate::{config, homeassistant};

pub fn rocket(config: config::Config) -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![page_get, page_post, zone_image])
        //.attach(Template::fairing())
        .manage(config)
}
#[get("/zone?<id>")]
async fn zone_image(config: &State<config::Config>, id: &str) -> Result<NamedFile, Custom<String>> {
    let zone_id = ZoneId::new(id).map_err(|e| status::Custom(Status::BadRequest, e))?;
    let photos = PhotoManager::new(config.photo_directory.clone());
    let Some(path) = photos.find_first_existing_path(&zone_id) else {
        return Err(status::Custom(
            Status::NotFound,
            "No zone image found".to_string(),
        ));
    };
    NamedFile::open(path)
        .await
        .map_err(|e| status::Custom(Status::ServiceUnavailable, e.to_string()))
}

#[get("/")]
async fn page_get(config: &State<config::Config>) -> Result<String, Custom<String>> {
    render_page(config, None).await
}
#[post("/", data = "<form>")]
async fn page_post(
    config: &State<config::Config>,
    form: Form<Contextual<'_, FormData>>,
) -> Result<String, Custom<String>> {
    render_page(config, form.value.as_ref()).await
}

async fn render_page(
    config: &State<config::Config>,
    form: Option<&FormData>,
) -> Result<String, Custom<String>> {
    let zones = get_zones(&config.homeassistant)
        .await
        .map_err(into_unavailable)?;

    Ok(format!("{zones:?}"))
}

async fn get_zones(
    config: &config::HomeAssistantConfig,
) -> anyhow::Result<Vec<homeassistant::Zone>> {
    let client = homeassistant::Client::new(&config.access_token, &config.endpoint)?;
    let zone_ids = client.get_zone_ids().await?;
    let mut zones = vec![];
    // Sequential because CBA
    for id in zone_ids {
        zones.push(client.get_entity::<homeassistant::Zone>(&id).await?);
    }
    Ok(zones)
}

#[derive(Debug, FromForm)]
struct FormData {
    zone_id: Option<String>,
}

fn into_unavailable<E: std::fmt::Display>(e: E) -> Custom<String> {
    Custom(
        http::Status::ServiceUnavailable,
        format!("Unavailable: {e}"),
    )
}
