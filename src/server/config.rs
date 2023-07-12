use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub home_assistant_host: String,
    pub home_assistant_port: u16,
    pub home_assistant_access_token: String,
    pub person_entity_ids: Vec<String>,
}

trait ConfigParamFromEnv {
    fn parse(val: &str) -> Result<Self, String>
    where
        Self: Sized;
}
impl ConfigParamFromEnv for String {
    fn parse(val: &str) -> Result<String, String> {
        Ok(val.into())
    }
}
impl ConfigParamFromEnv for u16 {
    fn parse(val: &str) -> Result<u16, String> {
        val.parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())
    }
}
impl<T> ConfigParamFromEnv for Vec<T>
where
    T: ConfigParamFromEnv,
{
    fn parse(val: &str) -> Result<Vec<T>, String> {
        val.split(',').map(ConfigParamFromEnv::parse).collect()
    }
}

fn get_env_variable<T>(key: &str) -> Result<T, String>
where
    T: ConfigParamFromEnv,
{
    get_optional_env_variable(key)?.ok_or(format!("Environment variable '{key}' not set."))
}
fn get_optional_env_variable<T>(key: &str) -> Result<Option<T>, String>
where
    T: ConfigParamFromEnv,
{
    match std::env::var(key) {
        Ok(v) => ConfigParamFromEnv::parse(&*v).map(Some),
        Err(_) => Ok(None),
    }
}

fn make_config_from_environment_variables() -> Result<Config, String> {
    Ok(Config {
        port: get_env_variable("PORT")?,
        home_assistant_host: get_env_variable("HOME_ASSISTANT_HOST")?,
        home_assistant_port: get_env_variable("HOME_ASSISTANT_PORT")?,
        home_assistant_access_token: get_env_variable("HOME_ASSISTANT_ACCESS_TOKEN")?,
        person_entity_ids: get_env_variable("PERSON_ENTITY_IDS")?,
    })
}

lazy_static! {
    /// The configuration, as parsed from environment variables.
    //
    // This unwraps the result so will panic if any of the environment variables are invalid.
    // This should be "okay" as it's initialised at the start of the program.
    pub static ref CONFIG: Config = make_config_from_environment_variables().unwrap();
}
