mod mirror;
mod render;
extern crate alloc;

use crate::mirror::{Mirror, Ray};
use alloc::sync::Arc;
use pollster::FutureExt;
use render::state::State;
use std::time::Instant;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

pub const DEFAULT_DIM: usize = 2;

fn main() {
    /// Load the mirror list from the json file
    let json = std::fs::read_to_string("assets/simple.json").unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    let mirrors_data = value
        .get("mirrors")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            Box::new(mirror::JsonError {
                message: "Invalid mirror list".to_string(),
            })
        })
        .unwrap();

    let mut mirrors = Vec::with_capacity(mirrors_data.len());
    for mirrorData in mirrors_data {
        mirrors.push(Box::<dyn Mirror<DEFAULT_DIM>>::from_json(mirrorData).unwrap());
    }

    let mut ray = Ray::<DEFAULT_DIM>::from_json(value.get("ray").unwrap()).unwrap();

    let mut rays = vec![ray];
    let reflection_limit = 100;

    for _ in 0..reflection_limit {
        let mut intersections = mirrors.intersecting_planes(&ray);
        println!("{:?}", ray);
        intersections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        if let Some((darkness, plane)) = intersections.first() {
            let reflected_ray = ray.reflect(plane, darkness);
            rays.push(reflected_ray);
            ray = reflected_ray;
        } else {
            break;
        }
    }
    println!("{:?}", rays);

    // run the wgpu
    // run().block_on();
}

async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("MirrorVerse")
            .build(&event_loop)
            .unwrap(),
    );

    let mut state = State::new(window.clone()).await;
    let mut last_render_time = Instant::now();

    event_loop
        .run(|event, target| {
            #[allow(clippy::single_match)]
            #[allow(clippy::collapsible_match)]
            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion{ delta, },
                    .. // We're not using device_id currently
                } => if state.mouse_pressed {
                    state.camera_controller.process_mouse(delta.0, delta.1)
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() && !state.input(event) => match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(keycode),
                                ..
                            },
                        ..
                    } => match keycode {
                        KeyCode::Escape => {
                            target.exit();
                        }
                        _ => {}
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        let now = Instant::now();
                        let dt = now - last_render_time;
                        last_render_time = now;

                        state.update(dt);
                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                target.exit();
                            }
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    },
                    _ => {}
                },
                Event::AboutToWait => {
                    state.window().request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
