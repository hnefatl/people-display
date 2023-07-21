use serde;
use lib::env_params;

/// An entity ID like `zone.home`. The "prefix" type param would be `zone` and the suffix would be `home`.
/// This is some magic but allows passing around strongly-typed entity IDs with validation of their format.
/// Any constructors/parsers will accept either e.g. `zone.home` or `home` and convert to canonical form.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct EntityId<const PREFIX: &'static str> {
    suffix: String,
}
pub type PrefixType = &'static str;
impl<const PREFIX: PrefixType> EntityId<PREFIX> {
    pub fn new<S: ToString>(value: &S) -> Self {
        EntityId {
            // Remove any existing prefix: turn either `home` or `zone.home` to `zone.home`.
            suffix: value.to_string().trim_start_matches(PREFIX).to_string(),
        }
    }
}
impl<const P: PrefixType> std::fmt::Display for EntityId<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{P}{}", self.suffix))
    }
}
impl<'de, const P: PrefixType> serde::Deserialize<'de> for EntityId<P> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        Ok(EntityId::new(&s))
    }
}

impl<const P: PrefixType> env_params::ConfigParamFromEnv for EntityId<P> {
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
