use lib::env_params;
use regex::Regex;

/// An entity ID like `zone.home`. The "prefix" type param would be `zone` and the suffix would be `home`.
/// This is some magic but allows passing around strongly-typed entity IDs with validation of their format.
/// Any constructors/parsers will accept either e.g. `zone.home` or `home` and convert to canonical form.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct EntityIdImpl<const PREFIX: &'static str> {
    suffix: String,
}
type PrefixType = &'static str;
pub trait EntityId: std::fmt::Display
where
    Self: std::marker::Sized,
{
    const PREFIX: PrefixType;
    fn new<S: ToString>(value: S) -> Result<Self, String>;
}
impl<const PREFIX: PrefixType> EntityId for EntityIdImpl<PREFIX> {
    const PREFIX: PrefixType = PREFIX;

    fn new<S: ToString>(value: S) -> Result<Self, String> {
        // Remove any existing prefix: turn either `home` or `zone.home` to `zone.home`.
        let s = value.to_string().trim_start_matches(PREFIX).to_string();
        if !Regex::new(r"^\w+$").unwrap().is_match(&s) {
            return Err(format!("Invalid entity ID: {s}"));
        }
        Ok(EntityIdImpl { suffix: s })
    }
}
impl<const P: PrefixType> std::fmt::Display for EntityIdImpl<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}{}", Self::PREFIX, self.suffix))
    }
}
impl<'de, const P: PrefixType> serde::Deserialize<'de> for EntityIdImpl<P> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        EntityIdImpl::new(s).map_err(serde::de::Error::custom)
    }
}

impl<const P: PrefixType> env_params::ConfigParamFromEnv for EntityIdImpl<P> {
    fn parse(val: &str) -> Result<Self, String> {
        EntityIdImpl::new(val)
    }
}

pub type PersonId = EntityIdImpl<"person.">;
pub type ZoneId = EntityIdImpl<"zone.">;
pub type InputBooleanId = EntityIdImpl<"input_boolean.">;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum AttributeValue {
    String(String),
    Float(f32),
    Int(i32),
    Bool(bool),
    List(Vec<AttributeValue>),
    Map(AttributeMap),
}
pub type AttributeMap = std::collections::HashMap<String, AttributeValue>;

/// Generic "thing that can have state fetched" trait, for tying together entity types and their IDs.
pub trait Entity: for<'a> serde::Deserialize<'a> {
    type Id: std::string::ToString;
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Person {
    #[serde(rename = "entity_id")]
    pub id: PersonId,

    /// The state of a person entity is the friendly name for the zone they're in.
    #[serde(rename = "state")]
    pub zone_friendly_name: String,

    /// The ID of the zone the perso n is in. This can't be gleaned from the entity state,
    /// it should be filled in by looking at the zone entities (which include person IDs).
    #[serde(skip)]
    pub zone_id: Option<ZoneId>,

    #[serde(default)]
    pub attributes: AttributeMap,
}
impl Person {
    pub fn get_entity_picture_path(&self) -> Option<String> {
        match self.attributes.get("entity_picture") {
            Some(AttributeValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }
}
impl Entity for Person {
    type Id = PersonId;
}
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Zone {
    #[serde(rename = "entity_id")]
    pub id: ZoneId,

    #[serde(default)]
    pub attributes: AttributeMap,
}
impl Zone {
    pub fn get_friendly_name(&self) -> Option<String> {
        match self.attributes.get("friendly_name") {
            Some(AttributeValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }
}
impl Entity for Zone {
    type Id = ZoneId;
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct InputBoolean {
    #[serde(rename = "state")]
    _state: String,
}
impl From<InputBoolean> for bool {
    fn from(value: InputBoolean) -> Self {
        // I think this could also be "Unavailable" if the server is starting up.
        // Rather than e.g. implement TryFrom and handle that case, just fail-closed
        // by treating anything other than specifically "off" as "privacy enabled".
        value._state != "off"
    }
}
impl Entity for InputBoolean {
    type Id = InputBooleanId;
}
