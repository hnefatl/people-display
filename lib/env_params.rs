use std::str::FromStr;

pub trait ConfigParamFromEnv {
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
impl ConfigParamFromEnv for std::path::PathBuf {
    fn parse(val: &str) -> Result<std::path::PathBuf, String> {
        Ok(val.into())
    }
}
impl ConfigParamFromEnv for tonic::transport::Uri {
    fn parse(val: &str) -> Result<tonic::transport::Uri, String> {
        tonic::transport::Uri::from_str(val).map_err(|e| format!("Invalid URI: {e}"))
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

pub fn get_env_variable<T>(key: &str) -> Result<T, String>
where
    T: ConfigParamFromEnv,
{
    get_optional_env_variable(key)?.ok_or(format!("Environment variable '{key}' not set."))
}
pub fn get_optional_env_variable<T>(key: &str) -> Result<Option<T>, String>
where
    T: ConfigParamFromEnv,
{
    match std::env::var(key) {
        Ok(v) => ConfigParamFromEnv::parse(&*v).map(Some),
        Err(_) => Ok(None),
    }
}

pub fn get_env_variable_with_default<T>(key: &str, default: T) -> Result<T, String>
where
    T: ConfigParamFromEnv,
{
    get_optional_env_variable(key).map(|v| v.unwrap_or(default))
}
