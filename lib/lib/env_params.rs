use secstr::SecStr;
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
impl ConfigParamFromEnv for SecStr {
    fn parse(val: &str) -> Result<SecStr, String> {
        Ok(SecStr::from(val))
    }
}

fn _get_env_variable<T: ConfigParamFromEnv>(key: &str) -> Result<Option<T>, String> {
    match std::env::var(key) {
        Ok(v) => Ok(Some(ConfigParamFromEnv::parse(&v)?)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
pub fn get_optional_env_variable<T: ConfigParamFromEnv>(key: &str) -> Result<Option<T>, String> {
    if let Some(v) = _get_env_variable(key)? {
        return Ok(Some(v));
    } else if let Some(path) = _get_env_variable::<String>(&format!("{key}_FILE"))? {
        let contents =
            std::fs::read_to_string(&path).map_err(|e| format!("When opening {path}: {e}"))?;
        return Ok(Some(ConfigParamFromEnv::parse(contents.trim())?));
    }
    Ok(None)
}

pub fn get_env_variable<T: ConfigParamFromEnv>(key: &str) -> Result<T, String> {
    get_optional_env_variable(key)?.ok_or(format!("Environment variable '{key}' not set."))
}
pub fn get_env_variable_with_default<T: ConfigParamFromEnv>(
    key: &str,
    default: T,
) -> Result<T, String> {
    get_optional_env_variable(key).map(|v| v.unwrap_or(default))
}
