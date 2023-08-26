use std::{str::FromStr, time::Duration};

use lib::env_params::ConfigParamFromEnv;
use serde::Deserialize;
use tonic;

#[derive(Deserialize, Debug)]
pub struct Endpoint {
    #[serde(deserialize_with = "deserialize_uri")]
    pub uri: tonic::transport::Uri,
    #[serde(deserialize_with = "deserialize_secstr")]
    pub password: secstr::SecStr,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub endpoints: Vec<Endpoint>,

    #[serde(rename = "poll_interval_seconds")]
    #[serde(default = "default_poll_interval")]
    #[serde(deserialize_with = "deserialize_seconds")]
    /// In the config, this is provided as `poll_interval_seconds: u64`, but at runtime
    /// it's converted directly into a `Duration` for ease of use.
    pub poll_interval: Duration,

    // The largest message that can be received from an exporter. Essentially, the limit
    // on photo size.
    #[serde(default = "default_max_received_message_size")]
    pub max_received_message_size: usize,
}
impl ConfigParamFromEnv for Config {
    fn parse(val: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        serde_json::from_str(val).map_err(|e| e.to_string())
    }
}

const fn default_poll_interval() -> Duration {
    Duration::from_secs(60)
}
const fn default_max_received_message_size() -> usize {
    30 * 1024 * 1024 // 30 MiB
}

fn deserialize_uri<'de, D>(deserializer: D) -> Result<tonic::transport::Uri, D::Error>
where
    D: serde::Deserializer<'de>,
{
    tonic::transport::Uri::from_str(Deserialize::deserialize(deserializer)?)
        .map_err(|e| serde::de::Error::custom(e.to_string()))
}
fn deserialize_secstr<'de, D>(deserializer: D) -> Result<secstr::SecStr, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Bit weird, to work around the Infallible error.
    let Ok(result) = secstr::SecStr::from_str(Deserialize::deserialize(deserializer)?);
    Ok(result)
}
fn deserialize_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Bit weird, to work around the Infallible error.
    Ok(Duration::from_secs(Deserialize::deserialize(deserializer)?))
}
