use crate::snapshot_manager::{EndpointSnapshots, Snapshot};
use lib::clock_pb;
use sdl2::image::LoadTexture;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};

fn bytes_to_texture<'a, T>(
    texture_creator: &'a TextureCreator<T>,
    bytes: &[u8],
) -> Result<Texture<'a>, String> {
    texture_creator.load_texture_bytes(bytes)
}

fn get_texture_rect(texture: &Texture) -> Rect {
    let sdl2::render::TextureQuery { width, height, .. } = texture.query();
    Rect::new(0, 0, width, height)
}

/// Produce a rect with the same aspect ratio as `inner`, entirely contained within `outer`.
/// The intent is to produce a scaled but not stretched image that can be rendered to fill
/// `inner`, potentially cropping out parts of `outer`.
/// The resulting rect has the same top-left corner as `inner`.
fn scale_inner_to_outer(outer: Rect, inner: Rect) -> Rect {
    let image_aspect_ratio = outer.width() as f32 / outer.height() as f32;
    let dest_aspect_ratio = inner.width() as f32 / inner.height() as f32;

    // Make a new dest-sized box aligned with the image's top left corner.
    let mut result = inner;
    // Will we have "black bars" at the top or side, if we scaled the two rects to the same size?
    if image_aspect_ratio > dest_aspect_ratio {
        result.set_height(outer.height());
        // Fix the height, maintaining the original aspect ratio.
        result.set_width((outer.height() as f32 * dest_aspect_ratio) as u32);
    } else {
        result.set_width(outer.width());
        result.set_height((outer.width() as f32 / dest_aspect_ratio) as u32);
    }
    result
}

pub struct Tile<'a> {
    person_texture: Option<Texture<'a>>,
    background_texture: Option<Texture<'a>>,
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
            .map(|b| bytes_to_texture(texture_creator, b))
            .transpose()?;
        let zone_texture = zone
            .and_then(|z| z.photo_data.as_ref())
            .map(|b| bytes_to_texture(texture_creator, b))
            .transpose()?;

        if zone_texture.is_none() {
            log::trace!(
                "Using blank zone texture for {}, no zone photo data provided.",
                &person.id
            );
        }

        // Mark all textures for smooth scaling, otherwise everything gets pixelated.
        for texture in [&person_texture, &zone_texture].into_iter().flatten() {
            unsafe {
                sdl2::sys::SDL_SetTextureScaleMode(
                    texture.raw(),
                    sdl2::sys::SDL_ScaleMode::SDL_ScaleModeBest,
                );
            }
        }

        Ok(Tile {
            person_texture,
            background_texture: zone_texture,
        })
    }

    pub fn draw<T: sdl2::render::RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        dest: Rect,
    ) -> Result<(), String> {
        Self::draw_background(&self.background_texture, canvas, dest)?;

        if let Some(person_texture) = &self.person_texture {
            Self::draw_person(person_texture, canvas, dest)?;
        }

        Ok(())
    }

    pub fn draw_background<T: sdl2::render::RenderTarget>(
        background_texture: &Option<Texture>,
        canvas: &mut Canvas<T>,
        dest: Rect,
    ) -> Result<(), String> {
        match background_texture {
            Some(texture) => {
                let background_rect = get_texture_rect(texture);
                // Scale+crop the photo to fit within the destination without stretching.
                let mut scaled_background_src = scale_inner_to_outer(background_rect, dest);
                // Centre the scaled source on our image.
                scaled_background_src.center_on(background_rect.center());
                canvas.copy(texture, scaled_background_src, dest)?;
            }
            None => {
                canvas.set_draw_color(sdl2::pixels::Color::BLACK);
                canvas.fill_rect(dest)?;
            }
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

        canvas.copy(person_texture, None, person_dest)?;
        Ok(person_dest)
    }
}

pub fn snapshot_to_tiles<'a, T>(
    texture_creator: &'a TextureCreator<T>,
    snapshot: &Snapshot,
) -> Vec<Tile<'a>> {
    let mut tiles = vec![];
    let mut sorted_people = snapshot.people.clone();
    sorted_people.sort_by_key(|p| p.id.clone());

    for person in sorted_people {
        match Tile::new(
            texture_creator,
            &person,
            person
                .zone_id
                .as_ref()
                .and_then(|id| snapshot.zones.get(id)),
        ) {
            Ok(image) => tiles.push(image),
            Err(e) => log::error!("Failed to render {person:?}: {e}"),
        }
    }
    tiles
}

pub fn snapshots_to_tiles<'a, T>(
    texture_creator: &'a TextureCreator<T>,
    snapshots: &EndpointSnapshots,
) -> Vec<Tile<'a>> {
    snapshots
        .iter()
        .flat_map(|s| snapshot_to_tiles(texture_creator, s))
        .collect()
}
