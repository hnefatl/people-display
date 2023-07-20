use env_logger;
use log;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::Window;
use snapshot_manager::{EndpointSnapshots, Snapshot, SnapshotManager};
use std::time::Duration;
use tokio;

use sdl2::render::{Canvas, Texture, TextureCreator};

use lib::clock_pb;
use lib::env_params::{get_env_variable, get_env_variable_with_default};
mod snapshot_manager;

fn bytes_to_texture<'a, T>(
    texture_creator: &'a TextureCreator<T>,
    bytes: &Vec<u8>,
) -> Result<Texture<'a>, String> {
    texture_creator.load_texture_bytes(bytes)
}

struct Tile<'a> {
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

fn snapshot_to_tiles<'a, T>(
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

fn draw_frame(snapshots: &EndpointSnapshots, canvas: &mut Canvas<Window>) -> Result<(), String> {
    let texture_creator = canvas.texture_creator();
    canvas.clear();
    let (width, height) = canvas.output_size()?;
    // TODO: handle multiple tiles properly
    for snapshot in snapshots {
        let tiles = snapshot_to_tiles(&texture_creator, snapshot);
        let draw_rect = Rect::new(0, 0, width, height);
        tiles[0].draw(canvas, draw_rect)?;
    }

    canvas.present();
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let endpoint_uris: Vec<tonic::transport::Uri> = get_env_variable("ENDPOINTS").unwrap();
    let poll_interval: u16 = get_env_variable_with_default("POLL_INTERVAL", 60).unwrap();

    let sdl_context = sdl2::init().expect("failed to init SDL");
    let video_subsystem = sdl_context.video().expect("failed to get video context");
    let window = video_subsystem
        .window("Display", 800, 600)
        .fullscreen_desktop()
        .build()
        .expect("failed to build window");

    let mut canvas: Canvas<Window> = window
        .into_canvas()
        .build()
        .expect("failed to build window's canvas");

    let (snapshot_manager, snapshot_receiver) =
        SnapshotManager::initialise(Duration::from_secs(poll_interval as u64), &endpoint_uris)
            .await;

    // Start periodically fetching locations in the background.
    tokio::spawn(snapshot_manager.start_loop());

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut latest_snapshots: EndpointSnapshots = vec![];
    loop {
        let mut quit = false;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape) | Some(Keycode::Q),
                    ..
                } => {
                    quit = true;
                }
                _ => {}
            }
        }
        if quit {
            break;
        }

        if let Some(snapshots) = snapshot_receiver.try_iter().last() {
            latest_snapshots = snapshots;
        }
        if let Err(e) = draw_frame(&latest_snapshots, &mut canvas) {
            log::error!("{}", e);
        }
        std::thread::sleep(Duration::from_millis(200)); // 5fps, don't need anything fancy
    }

    Ok(())
}
