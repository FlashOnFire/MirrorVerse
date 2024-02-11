mod mirror;
mod render;

use pollster::FutureExt;
use render::state::State;
use std::sync::Arc;
use std::time::Instant;
use winit::event::*;
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

pub const DIM: usize = 2;

fn main() {
    run().block_on();
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
