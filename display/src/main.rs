#![feature(never_type)]
#![feature(exhaustive_patterns)]

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::Window;
use snapshot_manager::{EndpointSnapshots, SnapshotManager};
use std::time::Duration;

use sdl2::render::Canvas;

use lib::env_params::get_env_variable;

mod config;
mod snapshot_manager;
mod tile;
use tile::{snapshots_to_tiles, Tile};

fn draw_frame(snapshots: &EndpointSnapshots, canvas: &mut Canvas<Window>) -> Result<(), String> {
    let texture_creator = canvas.texture_creator();
    canvas.set_draw_color(sdl2::pixels::Color::BLACK);
    canvas.clear();

    let tiles = snapshots_to_tiles(&texture_creator, snapshots);
    draw_tiles(&tiles, canvas)?;

    canvas.present();
    Ok(())
}

fn draw_tiles(tiles: &Vec<Tile>, canvas: &mut Canvas<Window>) -> Result<(), String> {
    // Make a square grid of tiles. `grid_size` is how many rows/columns we have.
    let grid_size = ((tiles.len() as f32).sqrt().ceil() as u32).max(1);

    let num_columns = grid_size;
    // Sneaky trick: with e.g. 2 tiles, we don't want 2 rows and columns, it wastes half the screen.
    // Instead, pick the number of columns according to a square, but only use as many rows as
    // necessary to fit all the tiles with that many columns.
    let num_rows = ((tiles.len() as f32 / num_columns as f32).ceil() as u32).max(1);

    let (output_width, output_height) = canvas.output_size()?;
    let (tile_width, tile_height) = (output_width / num_columns, output_height / num_rows);
    for (i, tile) in tiles.iter().enumerate() {
        let row = i as u32 / num_columns;
        let column = i as u32 % num_columns;

        let draw_rect = Rect::new(
            (column * tile_width) as i32,
            (row * tile_height) as i32,
            tile_width,
            tile_height,
        );

        tile.draw(canvas, draw_rect)?;
    }

    Ok(())
}

fn main_loop(
    canvas: &mut Canvas<Window>,
    event_pump: &mut sdl2::EventPump,
    snapshot_receiver: std::sync::mpsc::Receiver<EndpointSnapshots>,
) {
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
        if let Err(e) = draw_frame(&latest_snapshots, canvas) {
            log::error!("{}", e);
        }
        std::thread::sleep(Duration::from_millis(200)); // 5fps, don't need anything fancy

        if let Err(e) = canvas
            .window_mut()
            .set_fullscreen(sdl2::video::FullscreenType::Desktop)
        {
            log::error!("Failed to make fullscreen: {e}")
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let config: config::Config = get_env_variable("CONFIG").unwrap();

    let sdl_context = sdl2::init().expect("failed to init SDL");
    let video_subsystem = sdl_context.video().expect("failed to get video context");
    let window = video_subsystem
        .window("Display", 720, 480)
        .resizable()
        .fullscreen_desktop()
        .build()
        .expect("failed to build window");
    sdl_context.mouse().show_cursor(false);

    // Try to use smooth texture scaling.
    if !sdl2::hint::set("SDL_HINT_RENDER_SCALE_QUALITY", "linear") {
        log::error!("Failed to set render scale quality hint.");
    }

    let mut canvas: Canvas<Window> = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("failed to build window's canvas");

    let (snapshot_manager, snapshot_receiver) = SnapshotManager::initialise(config).await;

    // Start periodically fetching locations in the background.
    let snapshot_manager_handle = tokio::spawn(snapshot_manager.start_loop());

    let mut event_pump = sdl_context.event_pump().unwrap();
    main_loop(&mut canvas, &mut event_pump, snapshot_receiver);

    snapshot_manager_handle.abort();

    Ok(())
}
