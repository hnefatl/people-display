use secstr::SecStr;

use lib::env_params::{get_env_variable, get_env_variable_from_file};
use crate::homeassistant;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    /// Yeah, just a password. I wondered about using an SSH pub/priv key here, or
    /// session tokens etc, but it's all overkill and very complicated to set up.
    pub password: SecStr,
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
        password: get_env_variable_from_file("PASSWORD_FILE")?,
        homeassistant: HomeAssistantConfig {
            endpoint: get_env_variable("HOME_ASSISTANT_ENDPOINT")?,
            access_token: get_env_variable_from_file("HOME_ASSISTANT_ACCESS_TOKEN_FILE")?,
        },
        person_entity_ids: get_env_variable("PERSON_ENTITY_IDS")?,
        photo_directory: get_env_variable("PHOTO_DIRECTORY")?,
    })
}
