use std::str::FromStr;

use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub home_assistant_host: String,
    pub home_assistant_port: u16,
    pub home_assistant_access_token: String,
}

fn get_env_variable(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|e| format!("Failed to get environment variable '{key}': {e}"))
}
fn parse_env_variable<T>(key: &str) -> Result<T, String>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    get_env_variable(key).and_then(|v| {
        v.parse()
            .map_err(|e| format!("Invalid format for {key}: {e}"))
    })
}

lazy_static! {
    pub static ref CONFIG: Config = Config {
        port: parse_env_variable("PORT").unwrap(),
        home_assistant_host: get_env_variable("HOME_ASSISTANT_HOST").unwrap(),
        home_assistant_port: parse_env_variable("HOME_ASSISTANT_PORT").unwrap(),
        home_assistant_access_token: get_env_variable("HOME_ASSISTANT_ACCESS_TOKEN").unwrap(),
    };
}
