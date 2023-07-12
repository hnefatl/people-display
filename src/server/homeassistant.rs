use hass_rs::{HassClient, HassEntity, HassResult};

pub type PersonId = String;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Person {
    pub name: String,
    pub photo: Vec<u8>,
}
#[derive(Debug)]
pub struct Zone {
    pub name: String,
}
#[derive(Debug)]
pub struct Snapshot {
    pub people: Vec<(Person, Zone)>,
}

// Get all the states of the given people IDs that we can find.
async fn get_people_states(
    client: &mut HassClient,
    entity_ids: &Vec<PersonId>,
) -> HassResult<Vec<HassEntity>> {
    // Preprocess the entity ids to make membership tests faster.
    let entity_id_lookup = std::collections::HashSet::<_>::from_iter(entity_ids);

    let all_entity_states = client.get_states().await?;
    let mut relevant_people_states = vec![];
    for entity in all_entity_states {
        if entity_id_lookup.contains(&entity.entity_id) {
            relevant_people_states.push(entity);
        }
    }
    return Ok(relevant_people_states);
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

    let states = get_people_states(&mut client, entity_ids).await?;
    return Ok(Snapshot {
        people: states
            .into_iter()
            .map(|s| {
                (
                    Person {
                        name: s.entity_id,
                        photo: vec![],
                    },
                    Zone { name: s.state },
                )
            })
            .collect(),
    });
}
