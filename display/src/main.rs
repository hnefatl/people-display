#![feature(never_type)]
#![feature(exhaustive_patterns)]

use snapshot_manager::{EndpointSnapshots, SnapshotManager};
use std::time::Duration;

use sfml::{
    graphics::{BlendMode, Color, RenderStates, RenderTarget, RenderWindow, Transform},
    window::{ContextSettings, Event, Key, Style, VideoMode},
};

use lib::env_params::get_env_variable;

mod config;
mod snapshot_manager;
mod tile;
use tile::{snapshots_to_tiles, Tile};

fn render_frame(
    snapshots: &EndpointSnapshots,
    target: &mut dyn RenderTarget,
) -> Result<(), String> {
    target.clear(Color::BLACK);

    let tiles = snapshots_to_tiles(snapshots);
    render_tiles(&tiles, target)?;

    Ok(())
}

fn render_tiles(tiles: &Vec<Tile>, target: &mut dyn RenderTarget) -> Result<(), String> {
    // Make a square grid of tiles. `grid_size` is how many rows/columns we have.
    let grid_size = ((tiles.len() as f32).sqrt().ceil() as u32).max(1);

    let num_columns = grid_size;
    // Sneaky trick: with e.g. 2 tiles, we don't want 2 rows and columns, it wastes half the screen.
    // Instead, pick the number of columns according to a square, but only use as many rows as
    // necessary to fit all the tiles with that many columns.
    let num_rows = ((tiles.len() as f32 / num_columns as f32).ceil() as u32).max(1);

    let (tile_width, tile_height) = (target.size().x / num_columns, target.size().y / num_rows);
    for (i, tile) in tiles.iter().enumerate() {
        let row = i as u32 / num_columns;
        let column = i as u32 % num_columns;

        // The transform is abused here: it's from screenspace e.g. (0, 0, 800, 480) to a
        // destination rectangle for the tile to be rendered within. It's easier to do nice
        // image scaling under this model since we need to clip images to fit within their
        // rect rather than just smooth-scaling them.
        let mut transform = Transform::IDENTITY;
        // Reverse order :/ First scale, then translate.
        transform.translate((column * tile_width) as f32, (row * tile_height) as f32);
        transform.scale(1.0 / num_columns as f32, 1.0 / num_rows as f32);

        target.draw_with_renderstates(
            tile,
            &RenderStates::new(BlendMode::ALPHA, transform, None, None),
        );
    }

    Ok(())
}

fn main_loop(
    window: &mut RenderWindow,
    snapshot_receiver: std::sync::mpsc::Receiver<EndpointSnapshots>,
) {
    let mut latest_snapshots: EndpointSnapshots = vec![];
    while window.is_open() {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed | Event::KeyPressed { code: Key::Q, .. } => {
                    window.close();
                    break;
                }
                _ => {}
            }
        }

        if let Some(snapshots) = snapshot_receiver.try_iter().last() {
            latest_snapshots = snapshots;
        }
        if let Err(e) = render_frame(&latest_snapshots, window) {
            log::error!("{}", e);
        }
        window.display();
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let config: config::Config = get_env_variable("CONFIG").unwrap();

    let window_settings = ContextSettings {
        antialiasing_level: 2,
        ..Default::default()
    };

    let mut window = RenderWindow::new(
        VideoMode::desktop_mode(),
        "Display",
        Style::FULLSCREEN,
        &window_settings,
    );
    // 5fps, don't need anything fancy
    window.set_framerate_limit(5);
    window.set_mouse_cursor_visible(false);
    window.set_vertical_sync_enabled(true);
    window.set_active(true);

    let (snapshot_manager, snapshot_receiver) = SnapshotManager::initialise(config).await;

    // Start periodically fetching locations in the background.
    let snapshot_manager_handle = tokio::spawn(snapshot_manager.start_loop());

    main_loop(&mut window, snapshot_receiver);

    snapshot_manager_handle.abort();

    Ok(())
}
