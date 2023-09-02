use crate::snapshot_manager::{EndpointSnapshots, Snapshot};
use lib::clock_pb;
use sfml::{
    graphics::{Drawable, Rect, RenderTarget, Sprite, Texture, Transformable},
    system::Vector2,
    ResourceLoadError, SfBox,
};
use thiserror;

type UIntRect = sfml::graphics::Rect<u32>;

fn new_texture(bytes: &[u8]) -> Result<SfBox<Texture>, Error> {
    let mut texture = Texture::new().ok_or(Error::TextureCreation)?;
    texture.load_from_memory(bytes, sfml::graphics::Rect::default())?;
    Ok(texture)
}

/// Get a new source texture rectangle for a given original `src` texture box, such that the new source ensures no
/// black bars when rendered into a `clip`-sized space.
/// In other words, produce a rectangle with the same aspect ratio as `clip` which fits maximally into `src`.
fn clip_to_aspect_ratio(src: UIntRect, clip: UIntRect) -> UIntRect {
    let src_aspect_ratio = src.width as f32 / src.width as f32;
    let clip_aspect_ratio = clip.width as f32 / clip.height as f32;

    let (new_src_width, new_src_height);
    // Would we have "black bars" at the top or side, if we scaled `src` to fit entirely within `clip`?
    if src_aspect_ratio > clip_aspect_ratio {
        // Black bars would be at the top, so make the new source rectangle as tall as possible and cut off the sides.
        new_src_height = src.height;
        new_src_width = (src.height as f32 * clip_aspect_ratio) as u32;
    } else {
        // Black bars would be at the side, so make the new source rectangle wide to fit, and cut off the top/bottom.
        new_src_width = src.width;
        new_src_height = (src.height as f32 / clip_aspect_ratio) as u32;
    }

    // Shift the top-left corner of the src rectangle to centre the image.
    let x = (src.width - new_src_width) / 2;
    let y = (src.height - new_src_height) / 2;

    Rect::new(x, y, new_src_width, new_src_height)
}

/// Draw `texture` to `target` within `dest_rect`, clipping the texture to fit entirely within the destination without scaling.
fn draw_texture_absolute(target: &mut dyn RenderTarget, texture: &Texture, dest_rect: UIntRect) {
    let texture_rect = UIntRect::new(0, 0, texture.size().x, texture.size().y);
    // Scale+crop the photo to fit within the destination without stretching.
    let scaled_background_src = clip_to_aspect_ratio(texture_rect, dest_rect);

    let mut sprite = Sprite::with_texture_and_rect(&texture, scaled_background_src.as_other());

    // Set the absolute position of the sprite in screenspace
    sprite.set_position(dest_rect.position().as_other());
    // Scale the sprite to fit exactly within the screenspace rect. Thanks to the clipping above,
    // we won't spill outside that space or have black bars.
    sprite.set_scale(Vector2::new(
        dest_rect.width as f32 / scaled_background_src.width as f32,
        dest_rect.height as f32 / scaled_background_src.height as f32,
    ));
    target.draw_sprite(&sprite, &Default::default());
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create new texture")]
    TextureCreation,
    #[error("Failed to load texture: {0}")]
    ResourceLoad(#[from] ResourceLoadError),
}

pub struct Tile {
    person_texture: Option<SfBox<Texture>>,
    background_texture: Option<SfBox<Texture>>,
}
impl Tile {
    pub fn new(person: &clock_pb::Person, zone: Option<&clock_pb::Zone>) -> Result<Self, Error> {
        let person_texture = person
            .photo_data
            .as_ref()
            .map(|b| new_texture(b))
            .transpose()?;
        let zone_texture = zone
            .and_then(|z| z.photo_data.as_ref())
            .map(|b| new_texture(b))
            .transpose()?;

        if zone_texture.is_none() {
            log::trace!(
                "Using blank zone texture for {}, no zone photo data provided.",
                &person.id
            );
        }

        Ok(Tile {
            person_texture,
            background_texture: zone_texture,
        })
    }
}
impl Drawable for Tile {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn RenderTarget,
        rs: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let screen_rect = UIntRect::new(0, 0, target.size().x, target.size().y);
        // Get the desired destination rectangle from the screen+transform.
        let dest_rect = rs
            .transform
            .transform_rect(screen_rect.as_other())
            .as_other();

        // Draw the background
        if let Some(texture) = &self.background_texture {
            draw_texture_absolute(target, texture, dest_rect);
        }

        // Draw the person
        if let Some(texture) = &self.person_texture {
            /// What ratio of the destination rect should be allocated to the person's photo.
            const PERSON_RATIO: u32 = 4;
            let (scaled_width, scaled_height) = (
                dest_rect.width / PERSON_RATIO,
                dest_rect.height / PERSON_RATIO,
            );
            let scaled_dest = UIntRect::new(
                dest_rect.left + (dest_rect.width - scaled_width) / 2,
                dest_rect.top + (dest_rect.height - scaled_height) / 2,
                scaled_width,
                scaled_height,
            );
            draw_texture_absolute(target, texture, scaled_dest);
        }
    }
}

pub fn snapshot_to_tiles(snapshot: &Snapshot) -> Vec<Tile> {
    let mut tiles = vec![];
    let mut sorted_people = snapshot.people.clone();
    sorted_people.sort_by_key(|p| p.id.clone());

    for person in sorted_people {
        match Tile::new(
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

pub fn snapshots_to_tiles(snapshots: &EndpointSnapshots) -> Vec<Tile> {
    snapshots
        .iter()
        .flat_map(|s| snapshot_to_tiles(s))
        .collect()
}
