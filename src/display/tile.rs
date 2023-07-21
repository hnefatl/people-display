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

fn get_texture_rect(texture: &Texture) -> Rect {
    let sdl2::render::TextureQuery { width, height, .. } = texture.query();
    Rect::new(0, 0, width.into(), height.into())
}

/// Produce a rect with the same aspect ratio as `dest`, entirely contained within `image`.
/// The intent is to produce a scaled but not stretched background image that can be
/// rendered to fill `dest`, potentially cropping out parts of the image.
fn scale_image_to_dest(image: Rect, dest: Rect) -> Rect {
    let image_aspect_ratio = image.width() as f32 / image.height() as f32;
    let dest_aspect_ratio = dest.width() as f32 / dest.height() as f32;

    // Make a new dest-sized box aligned with the image's top left corner.
    let mut result = Rect::new(0, 0, dest.width(), dest.height());
    // Will we have "black bars" at the top or side, if we scaled the two rects to the same size?
    if image_aspect_ratio > dest_aspect_ratio {
        result.set_height(image.height());
        // Fix the height, maintaining the original aspect ratio.
        result.set_width((image.height() as f32 * dest_aspect_ratio) as u32);
    } else {
        result.set_width(image.width());
        result.set_height((image.width() as f32 / dest_aspect_ratio) as u32);
    }

    // Move to the centre so we crop off both sides evenly, not always the right side.
    result.center_on(image.center());
    return result;
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
        // Scale+crop the photo to fit within the destination without stretching.
        let background_rect = get_texture_rect(&self.background_texture);
        let scaled_background_src = scale_image_to_dest(background_rect, dest);
        canvas.copy(&self.background_texture, scaled_background_src, dest)?;

        const PERSON_RATIO: u32 = 4;
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
