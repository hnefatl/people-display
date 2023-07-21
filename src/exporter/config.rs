use lib::env_params::get_env_variable;
use crate::homeassistant;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub homeassistant: HomeAssistantConfig,
    pub person_entity_ids: Vec<homeassistant::PersonId>,
    pub photo_directory: std::path::PathBuf,
}
#[derive(Debug, Clone)]
pub struct HomeAssistantConfig {
    pub endpoint: String,
    pub access_token: String,
}

pub fn get_config_from_environment_variables() -> Result<Config, String> {
    Ok(Config {
        port: get_env_variable("PORT")?,
        homeassistant: HomeAssistantConfig {
            endpoint: get_env_variable("HOME_ASSISTANT_ENDPOINT")?,
            access_token: get_env_variable("HOME_ASSISTANT_ACCESS_TOKEN")?,
        },
        person_entity_ids: get_env_variable("PERSON_ENTITY_IDS")?,
        photo_directory: get_env_variable("PHOTO_DIRECTORY")?,
    })
}
