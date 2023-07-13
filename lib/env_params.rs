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
impl ConfigParamFromEnv for std::path::PathBuf {
    fn parse(val: &str) -> Result<std::path::PathBuf, String> {
        Ok(val.into())
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
