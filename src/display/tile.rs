use crate::snapshot_manager::Snapshot;
use lib::clock_pb;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::sys::SDL_UnlockMutex;
use sdl2::ttf::Font;

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

/// Produce a rect with the same aspect ratio as `inner`, entirely contained within `outer`.
/// The intent is to produce a scaled but not stretched image that can be rendered to fill
/// `inner`, potentially cropping out parts of `outer`.
/// The resulting rect has the same top-left corner as `inner`.
fn scale_inner_to_outer(outer: Rect, inner: Rect) -> Rect {
    let image_aspect_ratio = outer.width() as f32 / outer.height() as f32;
    let dest_aspect_ratio = inner.width() as f32 / inner.height() as f32;

    // Make a new dest-sized box aligned with the image's top left corner.
    let mut result = inner.clone();
    // Will we have "black bars" at the top or side, if we scaled the two rects to the same size?
    if image_aspect_ratio > dest_aspect_ratio {
        result.set_height(outer.height());
        // Fix the height, maintaining the original aspect ratio.
        result.set_width((outer.height() as f32 * dest_aspect_ratio) as u32);
    } else {
        result.set_width(outer.width());
        result.set_height((outer.width() as f32 / dest_aspect_ratio) as u32);
    }
    return result;
}

pub struct Tile<'a> {
    person_texture: Option<Texture<'a>>,
    background_texture: Texture<'a>,
    name_texture: Option<Texture<'a>>,
}
impl<'a> Tile<'a> {
    pub fn new<T>(
        texture_creator: &'a TextureCreator<T>,
        font: &Font,
        person: &clock_pb::Person,
        zone: Option<&clock_pb::Zone>,
    ) -> Result<Self, String> {
        let person_texture = person
            .photo_data
            .as_ref()
            .map(|b| bytes_to_texture(texture_creator, &b))
            .transpose()
            .map_err(|e| e.to_string())?;
        let zone_texture = zone
            .and_then(|z| z.photo_data.as_ref())
            .map(|b| bytes_to_texture(texture_creator, &b))
            .transpose()
            .map_err(|e| e.to_string())?;

        let background_texture = match zone_texture {
            Some(t) => t,
            None => {
                log::info!("Using blank texture for {person:?}, no zone photo data provided.");
                texture_creator
                    .create_texture_static(sdl2::pixels::PixelFormatEnum::RGB24, 100, 100)
                    .map_err(|e| format!("Unable to create blank texture: {e}"))?
            }
        };

        let mut name_texture = None;
        if let Some(name) = &person.name {
            let name_surface = font
                .render(&name)
                .solid(sdl2::pixels::Color::YELLOW)
                .map_err(|e| e.to_string())?;
            name_texture = Some(
                name_surface
                    .as_texture(texture_creator)
                    .map_err(|e| e.to_string())?,
            );
        }

        Ok(Tile {
            person_texture: person_texture,
            background_texture,
            name_texture,
        })
    }

    pub fn draw<T: sdl2::render::RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        dest: Rect,
    ) -> Result<(), String> {
        let background_rect = get_texture_rect(&self.background_texture);
        // Scale+crop the photo to fit within the destination without stretching.
        let mut scaled_background_src = scale_inner_to_outer(background_rect, dest);
        // Centre the scaled source on our image.
        scaled_background_src.center_on(background_rect.center());
        canvas.copy(&self.background_texture, scaled_background_src, dest)?;

        let mut person_dest = None;
        if let Some(person_texture) = &self.person_texture {
            person_dest = Some(Self::draw_person(person_texture, canvas, dest)?);
        }

        if let Some(name_texture) = &self.name_texture {
            let name_dest = dest;
            Self::draw_name(name_texture, canvas, name_dest)?;
        }

        Ok(())
    }

    pub fn draw_person<T: sdl2::render::RenderTarget>(
        person_texture: &Texture,
        canvas: &mut Canvas<T>,
        dest: Rect,
    ) -> Result<Rect, String> {
        /// What ratio of the destination rect should be allocated to the person's photo.
        const PERSON_RATIO: u32 = 4;
        let scaled_dest = Rect::new(
            0,
            0,
            dest.width() / PERSON_RATIO,
            dest.height() / PERSON_RATIO,
        );
        let person_rect = get_texture_rect(person_texture);
        // Scale+crop the destination to fit the photo without stretching.
        let mut person_dest = scale_inner_to_outer(scaled_dest, person_rect);
        // Centre the output in the overall destination rect.
        person_dest.center_on(dest.center());

        canvas.copy(&person_texture, None, person_dest)?;
        Ok(person_dest)
    }
    pub fn draw_name<T: sdl2::render::RenderTarget>(
        name_texture: &Texture,
        canvas: &mut Canvas<T>,
        dest: Rect,
    ) -> Result<(), String> {
        canvas.copy(name_texture, None, dest)?;
        Ok(())
    }
}

pub fn snapshot_to_tiles<'a, T>(
    texture_creator: &'a TextureCreator<T>,
    font: &Font,
    snapshot: &Snapshot,
) -> Vec<Tile<'a>> {
    let mut tiles = vec![];
    for person in &snapshot.people {
        match Tile::new(
            texture_creator,
            font,
            &person,
            snapshot.zones.get(&person.zone_id),
        ) {
            Ok(image) => tiles.push(image),
            Err(e) => log::error!("Failed to render {person:?}: {e}"),
        }
    }
    tiles
}
