use reqwest;
use serde_json;
use std::string::ToString;

// Re-export the types for convenience.
pub use crate::homeassistant_types::*;

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
        let headers = Client::make_headers(access_token).map_err(Error::InvalidHeaderValue)?;
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

    fn make_headers(
        access_token: &str,
    ) -> Result<reqwest::header::HeaderMap, reqwest::header::InvalidHeaderValue> {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth_header = format!("Bearer {access_token}");
        headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse()?);
        headers.insert(reqwest::header::AUTHORIZATION, auth_header.parse()?);
        Ok(headers)
    }

    fn make_url(&self, path: &str) -> reqwest::Url {
        let mut url = self.server_endpoint.clone();
        url.set_path(path);
        return url;
    }

    async fn get(&self, url: &reqwest::Url) -> Result<reqwest::Response, Error> {
        self.client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| Error::ReqwestError(e))
    }

    pub async fn get_entity<T: Entity>(&self, id: &T::Id) -> Result<T, Error> {
        // Risk of parameter injection? Nah, no way.
        let url = self.make_url(&format!("/api/states/{}", &id.to_string()));
        let response = self.get(&url).await?;
        let body = response.text().await.map_err(Error::InvalidResponseBody)?;
        serde_json::from_str(&body).map_err(|e| Error::JsonDecodeError(url, e, body))
    }

    pub async fn get_photo(&self, person: &Person) -> Result<Option<Vec<u8>>, Error> {
        match person.get_entity_picture_path() {
            Some(entity_picture_path) => {
                let url = self.make_url(&entity_picture_path);
                let response = self.get(&url).await?;
                let bytes = response
                    .bytes()
                    .await
                    .map_err(|e| Error::InvalidResponseBody(e))?;
                Ok(Some(bytes.into()))
            }
            None => Ok(None),
        }
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
