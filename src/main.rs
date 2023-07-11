#![feature(never_type)]
#![feature(exhaustive_patterns)]

use lazy_static::lazy_static;
use tokio;

#[derive(Debug, Clone)]
struct Config {
    host: String,
    port: u16,
    access_token: String,
    person_entity_id: String,
    poll_seconds: Option<u16>,
}

fn get_env_variable(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|e| format!("Failed to get environment variable '{key}': {e}"))
}
fn get_optional_env_variable(key: &str) -> Option<String> {
    get_env_variable(key).ok()
}
lazy_static! {
    static ref CONFIG: Config = Config {
        host: get_env_variable("HOST").unwrap(),
        port: get_env_variable("PORT")
            .and_then(|v| v.parse().map_err(|e| format!("Invalid port: {e}")))
            .unwrap(),
        access_token: get_env_variable("ACCESS_TOKEN").unwrap(),
        person_entity_id: get_env_variable("PERSON_ENTITY_ID").unwrap(),
        poll_seconds: get_optional_env_variable("POLL_SECONDS").map(|v| v
            .parse()
            .map_err(|e| format!("Invalid poll seconds: {e}"))
            .unwrap())
    };
}

async fn get_person_state(
    client: &mut hass_rs::HassClient,
    person_entity_id: &str,
) -> hass_rs::HassResult<Option<hass_rs::HassEntity>> {
    let states = client.get_states().await?;
    for entity in states {
        if entity.entity_id == person_entity_id {
            return Ok(Some(entity));
        }
    }
    return Ok(None);
}

async fn main_loop() -> hass_rs::HassResult<!> {
    let mut client = hass_rs::connect(&CONFIG.host, CONFIG.port).await?;
    println!("Connected, authenticating....");
    client
        .auth_with_longlivedtoken(&CONFIG.access_token)
        .await?;
    println!("Authenticated, starting main loop...");

    loop {
        match get_person_state(&mut client, &CONFIG.person_entity_id).await? {
            Some(person_state) => println!("{person_state:?}"),
            None => println!("Unable to find person {}", CONFIG.person_entity_id),
        }

        tokio::time::sleep(std::time::Duration::from_secs(
            CONFIG.poll_seconds.unwrap_or(60).into(),
        ))
        .await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ! {
    println!("Read config: {:?}", *CONFIG);

    loop {
        let Err(e) = main_loop().await;
        println!("Connection broken with error: {e}");
    }
}
