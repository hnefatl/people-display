use hass_rs::{HassClient, HassEntity, HassResult};

pub type EntityId = String;
pub type PersonId = EntityId;
pub type ZoneId = EntityId;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Person {
    /// HomeAssistant entity id like `person.keith`
    pub id: PersonId,
    /// Human-readable name like "Keith"
    pub name: Option<String>,
    pub zone_id: ZoneId,
}
#[derive(Debug)]
pub struct Zone {
    pub id: String,
    pub name: Option<String>,
}
#[derive(Debug)]
pub struct Snapshot {
    pub people: Vec<Person>,
    pub zones: std::collections::HashMap<ZoneId, Zone>,
}

type EntityStates = std::collections::HashMap<EntityId, HassEntity>;

async fn get_states(
    client: &mut HassClient,
) -> HassResult<std::collections::HashMap<String, HassEntity>> {
    Ok(client
        .get_states()
        .await?
        .into_iter()
        .map(|e| (e.entity_id.clone(), e))
        .collect())
}

fn get_friendly_name(entity: &HassEntity) -> Option<String> {
    entity
        .attributes
        .get("friendly_name")?
        .as_str()
        .map(str::to_string)
}

fn get_zone(zone_id: &ZoneId, entity_states: &EntityStates) -> Option<Zone> {
    let zone_entity = entity_states.get(zone_id)?;
    Some(Zone {
        id: zone_id.clone(),
        name: get_friendly_name(&zone_entity),
    })
}
fn get_person(person_id: &PersonId, entity_states: &EntityStates) -> Option<Person> {
    let person_entity = entity_states.get(person_id)?;
    Some(Person {
        id: person_id.clone(),
        name: get_friendly_name(&person_entity),
        zone_id: "zone.".to_string() + &person_entity.state,
    })
}

async fn open_hass_client(host: &str, port: u16, access_token: &str) -> HassResult<HassClient> {
    let mut client = hass_rs::connect(host, port).await?;
    client.auth_with_longlivedtoken(access_token).await?;
    return Ok(client);
}

pub async fn get_snapshot(
    host: &str,
    port: u16,
    access_token: &str,
    entity_ids: &Vec<PersonId>,
) -> HassResult<Snapshot> {
    let mut client = open_hass_client(host, port, access_token).await?;
    let states = get_states(&mut client).await?;

    let people: Vec<Person> = entity_ids
        .iter()
        .filter_map(|i| get_person(i, &states))
        .collect();

    let zone_ids: std::collections::HashSet<ZoneId> =
        people.iter().map(|p| p.zone_id.clone()).collect();
    let zones = zone_ids
        .iter()
        .filter_map(|i| get_zone(i, &states))
        .map(|e| (e.id.clone(), e))
        .collect();
    return Ok(Snapshot { people, zones });
}
