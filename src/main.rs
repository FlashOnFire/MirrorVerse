extern crate alloc;
extern crate core;

use cgmath as cg;
use glium::glutin::dpi::PhysicalPosition;
use glium::glutin::window::CursorGrabMode;
use glium::{
    self as gl,
    glutin::{self, event, event_loop},
};
use std::{fs::File, time};

use crate::render::DrawableSimulation;
use render::camera::{Camera, CameraController, Projection};

mod mirror;
mod render;

pub const DEFAULT_DIM: usize = 3;

pub const DEFAULT_WIDTH: u32 = 1280;
pub const DEFAULT_HEIGHT: u32 = 720;
pub const NEAR_PLANE: f32 = 0.1;
pub const FAR_PLANE: f32 = 2000.;

pub const SPEED: f32 = 5.;
pub const MOVEMENT_SENSITIVITY: f32 = 3.0;
pub const MOUSE_SENSITIVITY: f32 = 4.0;

pub const DEFAULT_CAMERA_POS: cg::Point3<f32> = cg::Point3::new(0., 0., 0.);
pub const DEFAULT_CAMERA_YAW: cg::Deg<f32> = cg::Deg(0.);
pub const DEFAULT_CAMERA_PITCH: cg::Deg<f32> = cg::Deg(0.);

pub const PROJECTION_FOV: cg::Deg<f32> = cg::Deg(85.);

pub const RAY_COLOR: [f32; 4] = [0.7, 0.3, 0.1, 1.0];
pub const MIRROR_COLOR: [f32; 4] = [0.3, 0.3, 0.9, 0.7];

fn main() {
    // Load the mirror list from the json file
    let file_path = std::env::args()
        .nth(1)
        .expect("Please provide a file path as a command-line argument.");

    let simulation = mirror::Simulation::<Vec<mirror::plane::PlaneMirror>>::from_json(
        &serde_json::from_reader(File::open(file_path).unwrap()).unwrap(),
    )
    .unwrap();

    let events_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
        .with_title("MirrorVerse");
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = gl::Display::new(wb, cb, &events_loop).unwrap();

    let mut camera = Camera::new(DEFAULT_CAMERA_POS, DEFAULT_CAMERA_YAW, DEFAULT_CAMERA_PITCH);

    let mut projection = Projection::new(
        DEFAULT_WIDTH,
        DEFAULT_HEIGHT,
        PROJECTION_FOV,
        NEAR_PLANE,
        FAR_PLANE,
    );
    let mut camera_controller = CameraController::new(5., MOVEMENT_SENSITIVITY, MOUSE_SENSITIVITY);

    let mut program3d = gl::Program::from_source(
        &display,
        render::VERTEX_SHADER_SRC_3D,
        render::FRAGMENT_SHADER_SRC,
        None,
    )
    .unwrap();

    let drawable_simulation = DrawableSimulation::new(simulation, 300, &display);

    let mut last_render_time = time::Instant::now();
    let mut mouse_pressed = false;

    events_loop.run(move |ev, _, control_flow| match ev {
        event::Event::WindowEvent { event, .. } => match event {
            event::WindowEvent::CloseRequested => *control_flow = event_loop::ControlFlow::Exit,
            event::WindowEvent::Resized(physical_size) => {
                if physical_size.width > 0 && physical_size.height > 0 {
                    projection.resize(physical_size.width, physical_size.height);
                }

                display.gl_window().resize(physical_size)
            }
            event::WindowEvent::MouseWheel { delta, .. } => {
                camera_controller.process_scroll(&delta);
            }
            event::WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    camera_controller.process_keyboard(keycode, input.state);
                }
            }
            event::WindowEvent::MouseInput { button, state, .. } => {
                if button == event::MouseButton::Left {
                    match state {
                        event::ElementState::Pressed => {
                            mouse_pressed = true;
                            display
                                .gl_window()
                                .window()
                                .set_cursor_grab(CursorGrabMode::Locked)
                                .or_else(|_| {
                                    display
                                        .gl_window()
                                        .window()
                                        .set_cursor_grab(CursorGrabMode::Confined)
                                })
                                .unwrap();

                            display.gl_window().window().set_cursor_visible(false);
                        }
                        event::ElementState::Released => {
                            mouse_pressed = false;
                            display
                                .gl_window()
                                .window()
                                .set_cursor_grab(CursorGrabMode::None)
                                .unwrap();
                            display.gl_window().window().set_cursor_visible(true);
                        }
                    }
                }
            }
            _ => {}
        },
        event::Event::RedrawRequested(_) => {
            let now = time::Instant::now();
            let dt = now - last_render_time;
            last_render_time = now;

            let elapsed_time = dt.as_millis() as u64;

            let wait_millis = if 1000 / 244 >= elapsed_time {
                1000 / 244 - elapsed_time
            } else {
                0
            };
            let new_inst = now + time::Duration::from_millis(wait_millis);
            *control_flow = event_loop::ControlFlow::WaitUntil(new_inst);

            update(dt, &mut camera, &mut camera_controller);
            drawable_simulation.render(&display, &mut program3d, &camera, &projection);
        }
        event::Event::MainEventsCleared => display.gl_window().window().request_redraw(),
        event::Event::DeviceEvent {
            event: event::DeviceEvent::MouseMotion { delta, .. },
            ..
        } => {
            if mouse_pressed {
                let inner_window_size = display.gl_window().window().inner_size();

                display
                    .gl_window()
                    .window()
                    .set_cursor_position(PhysicalPosition {
                        x: inner_window_size.width / 2,
                        y: inner_window_size.height / 2,
                    })
                    .unwrap();
                camera_controller.process_mouse(delta.0, delta.1)
            }
        }
        _ => (),
    });
}

fn update(dt: time::Duration, camera: &mut Camera, camera_controller: &mut CameraController) {
    camera_controller.update_camera(camera, dt);
}
