use crate::snapshot_manager::Snapshot;
use lib::clock_pb;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};

fn bytes_to_texture<'a, T>(
    texture_creator: &'a TextureCreator<T>,
    bytes: &Vec<u8>,
) -> Result<Texture<'a>, String> {
    texture_creator.load_texture_bytes(bytes)
}

pub struct Tile<'a> {
    person_name: Option<String>,
    person_texture: Option<Texture<'a>>,
    background_texture: Texture<'a>,
}
impl<'a> Tile<'a> {
    pub fn new<T>(
        texture_creator: &'a TextureCreator<T>,
        person: &clock_pb::Person,
        zone: Option<&clock_pb::Zone>,
    ) -> Result<Self, String> {
        let person_texture = person
            .photo_data
            .as_ref()
            .map(|b| bytes_to_texture(texture_creator, &b))
            .transpose()
            .map_err(|e| format!("{e}"))?;
        let zone_texture = zone
            .and_then(|z| z.photo_data.as_ref())
            .map(|b| bytes_to_texture(texture_creator, &b))
            .transpose()
            .map_err(|e| format!("{e}"))?;

        let background_texture = match zone_texture {
            Some(t) => t,
            None => {
                log::info!("Using blank texture for {person:?}, no zone photo data provided.");
                texture_creator
                    .create_texture_static(sdl2::pixels::PixelFormatEnum::RGB24, 100, 100)
                    .map_err(|e| format!("Unable to create blank texture: {e}"))?
            }
        };

        Ok(Tile {
            person_name: person.name.clone(),
            person_texture: person_texture,
            background_texture,
        })
    }

    pub fn draw<T>(&self, canvas: &mut Canvas<T>, dest: Rect) -> Result<(), String>
    where
        T: sdl2::render::RenderTarget,
    {
        // TODO: sort scaling/cropping
        const PERSON_RATIO: u32 = 4;
        canvas.copy(&self.background_texture, None, dest)?;
        if let Some(person_texture) = &self.person_texture {
            let person_size =
                std::cmp::min(dest.width() / PERSON_RATIO, dest.height() / PERSON_RATIO);
            let mut person_rect = Rect::new(0, 0, person_size, person_size);
            person_rect.center_on(dest.center());
            canvas.copy(&person_texture, None, person_rect)?;
        }
        Ok(())
    }
}

pub fn snapshot_to_tiles<'a, T>(
    texture_creator: &'a TextureCreator<T>,
    snapshot: &Snapshot,
) -> Vec<Tile<'a>> {
    let mut tiles = vec![];
    for person in &snapshot.people {
        match Tile::new(
            texture_creator,
            &person,
            snapshot.zones.get(&person.zone_id),
        ) {
            Ok(image) => tiles.push(image),
            Err(e) => log::error!("Failed to render {person:?}: {e}"),
        }
    }
    tiles
}
