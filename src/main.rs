#![allow(warnings)]

extern crate alloc;

use std::time::{Duration, Instant};
use cgmath::perspective;
use glium::glutin::event::{ElementState, Event, MouseButton, WindowEvent};
use glium::glutin::event::DeviceEvent::MouseMotion;
use glium::glutin::event_loop::ControlFlow;
use glium::{implement_vertex, Program, Surface, uniform};
use gnuplot::Figure;
use nalgebra::Point3;
use render::state::State;

use mirror::{plane::PlaneMirror, Mirror, Ray};
use render::gnuplot::render_gnu_plot;
use crate::render::camera::{Camera, CameraController, Projection};

mod mirror;
mod render;

pub const DEFAULT_DIM: usize = 2;

const vertex_shader_src: &str = r#"
        #version 140

        in vec2 position;
        uniform mat4 perspective;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

const fragment_shader_src: &str = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(0.7, 0.3, 0.1, 1.0);
        }
    "#;

const vertex_shader_src_3d: &str = r#"
        #version 140

        in vec3 position;
        uniform mat4 perspective;
        uniform mat4 view;

        void main() {
            gl_Position = perspective * view * vec4(position, 1.0);
        }
    "#;

fn main() {
    render2();

    return;

    /// Load the mirror list from the json file
    let json = std::fs::read_to_string("assets/simple.json").unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    let mut mirrors =
        Vec::<PlaneMirror>::from_json(value.get("mirrors").expect("mirrors field expected"))
            .expect("expected data in mirrors field to be well-formed");

    let mut ray = Ray::<DEFAULT_DIM>::from_json(value.get("ray").unwrap()).unwrap();

    let mut rays = vec![ray];
    let reflection_limit = 100;

    for _ in 0..reflection_limit {
        let mut intersections = mirrors.intersecting_planes(&ray);
        println!("{:?}", ray);
        //sort the intersections by distance using distance_to_point
        intersections.sort_by(|a, b| {
            a.1.distance_to_ray(ray)
                .partial_cmp(&b.1.distance_to_ray(ray))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some((darkness, plane)) = intersections.first() {
            let reflected_ray = ray.reflect(plane, darkness);
            rays.push(reflected_ray);
            ray = reflected_ray;
        } else {
            break;
        }
    }
    println!("{:?}", rays);

    let mut fg = Figure::new();

    render_gnu_plot(&mut fg, &rays, &mirrors);

    fg.show().unwrap();
    // run the wgpu
    // run().block_on();
}

async fn run() {
    /*env_logger::init();

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
                    event: DeviceEvent::MouseMotion { delta, },
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
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    state.window().request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();*/
}

fn render2() {
    let mut events_loop = glium::glutin::event_loop::EventLoop::new();
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1280.0, 720.0))
        .with_title("MirrorVerse");
    let cb = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let mut camera = Camera::new(
        Point3::new(0.0, 0.0, 0.0),
        cgmath::Deg(0.0),
        cgmath::Deg(0.0),
    );

    let mut projection = Projection::new(1280, 720, cgmath::Deg(70.0), 0.1, 100.0);
    let mut camera_controller = CameraController::new(10.0, 0.4);

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }
    implement_vertex!(Vertex, position);
    let shape = vec![
        Vertex { position: [-0.5, -0.5] },
        Vertex { position: [0.0, 0.5] },
        Vertex { position: [0.5, -0.25] },
    ];
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();
    let mut program3d = glium::Program::from_source(&display, vertex_shader_src_3d, fragment_shader_src, None).unwrap();

    let mut last_render_time = Instant::now();

    let mut mouse_pressed = false;

    events_loop.run(move |ev, _, control_flow| {
        match ev {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested =>
                    *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    if physical_size.width > 0 && physical_size.height > 0 {
                        projection.resize(physical_size.width, physical_size.height);
                    }

                    display.gl_window().resize(physical_size)
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    camera_controller.process_scroll(&delta);
                }
                WindowEvent::KeyboardInput {
                    input,
                    ..
                } => {
                    if let Some(keycode) = input.virtual_keycode {
                        camera_controller.process_keyboard(keycode, input.state);
                    }
                }
                WindowEvent::MouseInput {
                    button,
                    state,
                    ..
                } => {
                    if (button == MouseButton::Left) {
                        mouse_pressed = match state {
                            ElementState::Pressed => true,
                            ElementState::Released => false
                        }
                    }
                }
                _ => {}
            }
            Event::RedrawRequested(_) => {
                println!("oui");
                let now = Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                let elapsed_time = dt.as_millis() as u64;

                let wait_millis = match (1000 / 244 >= elapsed_time) {
                    true => 1000 / 244 - elapsed_time,
                    false => 0
                };
                let new_inst = now + std::time::Duration::from_millis(wait_millis);
                *control_flow = ControlFlow::WaitUntil(new_inst);

                update(dt, &mut camera, &mut camera_controller);
                render(&display, &mut program3d, &camera, &projection);
            }
            Event::MainEventsCleared => display.gl_window().window().request_redraw(),
            Event::DeviceEvent {
                event: MouseMotion {
                    delta,
                    ..
                }, ..
            } => if mouse_pressed {
                camera_controller.process_mouse(delta.0, delta.1)
            }
            _ => {}
        }
    });
}

fn update(dt: Duration, camera: &mut Camera, camera_controller: &mut CameraController) {
    camera_controller.update_camera(camera, dt);
}

fn render(display: &glium::backend::glutin::Display, program3d: &mut Program, camera: &Camera, projection: &Projection) {
    let mut target = display.draw();
    target.clear_color_and_depth((1.0, 1.0, 1.0, 1.0), 1.0);
    //target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,
    //            &Default::default()).unwrap();


    let cuboid = glium_shapes::cuboid::CuboidBuilder::new()
        .translate(0.0, 0.0, 0.0)
        .scale(0.5, 0.5, 0.5)
        .build(display)
        .expect("Failed to build cuboid shape");

    let circle = glium_shapes::sphere::SphereBuilder::new()
        .translate(0.0, 0.0, 3.0)
        .scale(0.2, 0.2, 0.2)
        .with_divisions(100, 100)
        .build(display)
        .expect("failed to build sphere shape");

    let (width, height) = target.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;

    let mat = perspective(cgmath::Deg(45.0), (16.0) / (9.0), 1000.0, 0.1);
    let perspective: [[f32; 4]; 4] = mat.into();


    let view: [[f32; 4]; 4] = camera.calc_matrix().into();


    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        ..Default::default()
    };

    target.draw(&cuboid, &cuboid, &program3d, &uniform! {perspective: perspective, view: view}, &params).unwrap();

    target.draw(&circle, &circle, &program3d, &uniform! {perspective: perspective, view: view}, &Default::default()).unwrap();

    target.finish().unwrap();

    display.gl_window().window().request_redraw();
}