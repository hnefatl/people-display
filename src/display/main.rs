use env_logger;
use log;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::video::Window;
use snapshot_manager::{EndpointSnapshots, SnapshotManager};
use std::time::Duration;
use tokio;

use sdl2::render::Canvas;

use lib::env_params::{get_env_variable, get_env_variable_with_default};

mod snapshot_manager;
mod tile;
use tile::snapshot_to_tiles;

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
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let endpoint_uris: Vec<tonic::transport::Uri> = get_env_variable("ENDPOINTS").unwrap();
    let poll_interval: u16 = get_env_variable_with_default("POLL_INTERVAL", 60).unwrap();

    let sdl_context = sdl2::init().expect("failed to init SDL");
    let video_subsystem = sdl_context.video().expect("failed to get video context");
    let window = video_subsystem
        .window("Display", 720, 480)
        .resizable()
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
    let snapshot_manager_handle = tokio::spawn(snapshot_manager.start_loop());

    let mut event_pump = sdl_context.event_pump().unwrap();
    main_loop(&mut canvas, &mut event_pump, snapshot_receiver);

    snapshot_manager_handle.abort();

    Ok(())
}
