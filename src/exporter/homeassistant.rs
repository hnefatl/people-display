use reqwest;
use serde;
use serde_json;
use std::string::ToString;

use lib::env_params;

/// An entity ID like `zone.home`. The "prefix" type param would be `zone` and the suffix would be `home`.
/// This is some magic but allows passing around strongly-typed entity IDs with validation of their format.
/// Any constructors/parsers will accept either e.g. `zone.home` or `home` and convert to canonical form.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct EntityId<const PREFIX: &'static str> {
    suffix: String,
}
impl<const PREFIX: &'static str> EntityId<PREFIX> {
    pub fn new<S: ToString>(value: &S) -> Self {
        EntityId {
            // Remove any existing prefix: turn either `home` or `zone.home` to `zone.home`.
            suffix: value.to_string().trim_start_matches(PREFIX).to_string(),
        }
    }
}
impl<const P: &'static str> std::fmt::Display for EntityId<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{P}{}", self.suffix))
    }
}
impl<'de, const P: &'static str> serde::Deserialize<'de> for EntityId<P> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Buncha random stuff to be able to parse a generic string? Not sure why this isn't
        // made public from the serde module.
        struct StringVisitor;
        impl<'de> serde::de::Visitor<'de> for StringVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("string")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(s.to_string())
            }
        }

        deserializer
            .deserialize_string(StringVisitor)
            .map(|s| EntityId::new(&s))
    }
}

impl<const P: &'static str> env_params::ConfigParamFromEnv for EntityId<P> {
    fn parse(val: &str) -> Result<Self, String> {
        Ok(EntityId::new(&val.to_string()))
    }
}

pub type PersonId = EntityId<"person.">;
pub type ZoneId = EntityId<"zone.">;

#[derive(serde::Deserialize, Debug)]
pub struct Person {
    #[serde(rename = "entity_id")]
    pub id: PersonId,
    #[serde(rename = "state")]
    pub zone_id: ZoneId,

    // TODO: somehow get this from attributes
    #[serde(default)]
    pub friendly_name: Option<String>,
}
#[derive(serde::Deserialize, Debug)]
pub struct Zone {
    #[serde(rename = "entity_id")]
    pub id: ZoneId,

    // TODO: somehow get this from attributes
    #[serde(default)]
    pub friendly_name: Option<String>,
}
/// Generic "thing that can have state fetched" trait, for tying together entity types and their IDs.
pub trait Entity: for<'a> serde::Deserialize<'a> {
    type Id: std::string::ToString;
}
impl Entity for Person {
    type Id = PersonId;
}
impl Entity for Zone {
    type Id = ZoneId;
}

#[derive(Debug)]
pub enum Error {
    InvalidHeaderValue(reqwest::header::InvalidHeaderValue),
    ReqwestError(reqwest::Error),
    UrlParseError(String),
    InvalidResponseBody(reqwest::Error),
    JsonDecodeError(reqwest::Url, serde_json::Error, String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidHeaderValue(e) => f.write_fmt(format_args!("Invalid header value: {e}")),
            Error::ReqwestError(e) => f.write_fmt(format_args!("Reqwest error: {e}")),
            Error::UrlParseError(e) => f.write_fmt(format_args!("URL parse error: {e}")),
            Error::InvalidResponseBody(e) => {
                f.write_fmt(format_args!("Invalid response body: {e}"))
            }
            Error::JsonDecodeError(url, e, body) => f.write_fmt(format_args!(
                "JSON decode error from '{url}': {e}.\n\nFull text: {body}"
            )),
        }
    }
}

pub struct Client {
    client: reqwest::Client,
    server_endpoint: reqwest::Url,
}
impl Client {
    pub fn new(access_token: &str, endpoint: &str) -> Result<Self, Error> {
        let headers = Client::_make_headers(access_token).map_err(Error::InvalidHeaderValue)?;
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(Error::ReqwestError)?;
        Ok(Client {
            client,
            server_endpoint: reqwest::Url::parse(endpoint)
                .map_err(|e| Error::UrlParseError(e.to_string()))?,
        })
    }

    fn _make_headers(
        access_token: &str,
    ) -> Result<reqwest::header::HeaderMap, reqwest::header::InvalidHeaderValue> {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth_header = format!("Bearer {access_token}");
        headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse()?);
        headers.insert(reqwest::header::AUTHORIZATION, auth_header.parse()?);
        Ok(headers)
    }

    pub async fn get_entity<T: Entity>(&self, id: &T::Id) -> Result<T, Error> {
        let url = self
            .server_endpoint
            .join("/api/states/")
            .and_then(|u| u.join(&id.to_string()))
            .map_err(|e| Error::UrlParseError(e.to_string()))?;
        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(Error::ReqwestError)?;
        let body = response.text().await.map_err(Error::InvalidResponseBody)?;
        serde_json::from_str(&body).map_err(|e| Error::JsonDecodeError(url, e, body))
    }
}

#[derive(Debug)]
pub struct Snapshot {
    pub people: Vec<Person>,
    pub zones: std::collections::HashMap<ZoneId, Zone>,
}

pub async fn get_snapshot(client: &Client, person_ids: &Vec<PersonId>) -> Snapshot {
    // A naive not-very-async implementation. This could be significantly parallelised, but using e.g.
    // tokio::task::JoinSet requires fiddling with lifetimes and moved data.

    let mut people: Vec<Person> = vec![];
    for person_id in person_ids {
        match client.get_entity::<Person>(&person_id).await {
            Ok(person) => {
                people.push(person);
            }
            Err(e) => log::warn!("Failed to get person state: {e}"),
        }
    }

    let mut zones = std::collections::HashMap::new();
    for person in &people {
        if zones.contains_key(&person.zone_id) {
            continue;
        }
        match client.get_entity::<Zone>(&person.zone_id).await {
            Ok(zone) => {
                zones.insert(zone.id.clone(), zone);
            }
            Err(e) => log::warn!("Failed to get zone state: {e}"),
        }
    }

    Snapshot { people, zones }
}
