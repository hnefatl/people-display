use std::{collections::HashMap, string::ToString};

// Re-export the types for convenience.
pub use crate::homeassistant_types::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("JSON decode error from '{0}': {1}. Full text: {2:?}")]
    JsonDecode(reqwest::Url, serde_json::Error, String),
    #[error("JSON encode error from '{0}': {1}.")]
    JsonEncode(reqwest::Url, serde_json::Error),
    #[error("Invalid access token: {0}")]
    InvalidAccessToken(#[from] std::str::Utf8Error),
}

pub struct Client {
    client: reqwest::Client,
    server_endpoint: reqwest::Url,
}
impl Client {
    pub fn new(access_token: &secstr::SecStr, endpoint: &str) -> Result<Self, Error> {
        let headers = Client::make_headers(access_token)?;
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        Ok(Client {
            client,
            server_endpoint: reqwest::Url::parse(endpoint)?,
        })
    }

    fn make_headers(access_token: &secstr::SecStr) -> Result<reqwest::header::HeaderMap, Error> {
        let access_token_str = std::str::from_utf8(access_token.unsecure())?;
        let mut auth_header: reqwest::header::HeaderValue =
            format!("Bearer {access_token_str}").parse()?;
        auth_header.set_sensitive(true);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse()?);
        headers.insert(reqwest::header::AUTHORIZATION, auth_header);
        Ok(headers)
    }

    fn make_url(&self, path: &str) -> reqwest::Url {
        let mut url = self.server_endpoint.clone();
        url.set_path(path);
        url
    }

    async fn get(&self, url: &reqwest::Url) -> Result<reqwest::Response, Error> {
        Ok(self.client.get(url.clone()).send().await?)
    }

    pub async fn get_entity<T: Entity>(&self, id: &T::Id) -> Result<T, Error> {
        // Risk of parameter injection? Nah, no way.
        let url = self.make_url(&format!("/api/states/{}", &id.to_string()));
        let response = self.get(&url).await?;
        let body = response.text().await?;
        serde_json::from_str(&body).map_err(|e| Error::JsonDecode(url, e, body))
    }

    pub async fn get_template<T: serde::de::DeserializeOwned>(
        &self,
        template: String,
    ) -> Result<T, Error> {
        let url = self.make_url("/api/template");
        let body = serde_json::to_string(&HashMap::from([("template", template)]))
            .map_err(|e| Error::JsonEncode(url.clone(), e))?;

        let response = self
            .client
            .post(url.clone())
            .body(body)
            .send()
            .await?
            .text()
            .await?;
        serde_json::from_str(&response).map_err(|e| Error::JsonDecode(url, e, response))
    }

    pub async fn get_photo(&self, person: &Person) -> Result<Option<Vec<u8>>, Error> {
        match person.get_entity_picture_path() {
            Some(entity_picture_path) => {
                let url = self.make_url(&entity_picture_path);
                let response = self.get(&url).await?;
                let bytes = response.bytes().await?;
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

pub async fn get_snapshot(client: &Client, person_ids: &Vec<PersonId>) -> Result<Snapshot, Error> {
    // A naive not-very-async implementation. This could be significantly parallelised, but using e.g.
    // tokio::task::JoinSet requires fiddling with lifetimes and moved data.

    let mut people = HashMap::new();
    for person_id in person_ids {
        let person = client.get_entity::<Person>(person_id).await?;
        people.insert(person_id, person);
    }

    // Get all zone IDs
    let template = r#"{{states.zone|list|map(attribute="entity_id")|list|to_json}}"#;

    let mut zones = std::collections::HashMap::new();
    let zone_ids: Vec<ZoneId> = client.get_template(template.to_string()).await?;
    log::trace!("All zone ids: {zone_ids:?}");
    for zone_id in zone_ids {
        let zone = client.get_entity::<Zone>(&zone_id).await?;

        // If there's no people in this zone, then we don't need to transmit any data about it.
        let Some(AttributeValue::List(contained_people_ids)) = zone.attributes.get("persons")
        else {
            continue;
        };
        if contained_people_ids.is_empty() {
            continue;
        }
        log::trace!(
            "Zone {} contains people {:?}",
            zone.id,
            contained_people_ids
        );

        // Link any people in this zone.
        for contained_person_id in contained_people_ids {
            let AttributeValue::String(id) = contained_person_id else {
                log::warn!(
                    "Got a non-string person ID in zone {}: {:?}",
                    &zone_id,
                    &contained_person_id
                );
                continue;
            };
            if let Some(person) = people.get_mut(&PersonId::new(id)) {
                person.zone_id = Some(zone_id.clone());
            }
        }

        zones.insert(zone_id, zone);
    }

    Ok(Snapshot {
        people: people.into_values().collect(),
        zones,
    })
}
