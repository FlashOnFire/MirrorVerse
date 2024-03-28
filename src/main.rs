#![allow(warnings)]

extern crate alloc;

use cgmath::perspective;
use glium::glutin::event::DeviceEvent::MouseMotion;
use glium::glutin::event::{ElementState, Event, MouseButton, WindowEvent};
use glium::glutin::event_loop::ControlFlow;
use glium::{implement_vertex, uniform, Program, Surface};
use nalgebra::Point3;
use std::time::{Duration, Instant};

use crate::render::camera::{Camera, CameraController, Projection};
use mirror::{plane::PlaneMirror, Mirror, Ray};

mod mirror;
mod render;

pub const DEFAULT_DIM: usize = 3;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
}
implement_vertex!(Vertex, position);

const VERTEX_SHADER_SRC: &str = r#"
        #version 140

        in vec2 position;
        uniform mat4 perspective;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

const FRAGMENT_SHADER_SRC: &str = r#"
        #version 140

        uniform vec3 color_vec;

        out vec4 color;

        void main() {
            color = vec4(color_vec.xyz, 1.0);
        }
    "#;

const VERTEX_SHADER_SRC_3D: &str = r#"
        #version 140

        in vec3 position;
        uniform mat4 perspective;
        uniform mat4 view;

        void main() {
            gl_Position = perspective * view * vec4(position, 1.0);
        }
    "#;

fn main() {
    let mut events_loop = glium::glutin::event_loop::EventLoop::new();
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1280.0, 720.0))
        .with_title("MirrorVerse");
    let cb = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let mut camera = Camera::new(
        Point3::new(0.0, 0.0, 0.0),
        cgmath::Deg(90.0),
        cgmath::Deg(0.0),
    );

    let mut projection = Projection::new(1280, 720, cgmath::Deg(70.0), 0.1, 100.0);
    let mut camera_controller = CameraController::new(5.0, 0.4);

    let mut program3d =
        Program::from_source(&display, VERTEX_SHADER_SRC_3D, FRAGMENT_SHADER_SRC, None).unwrap();

    let mut last_render_time = Instant::now();

    let mut mouse_pressed = false;

    /// Load the mirror list from the json file
    let json = std::fs::read_to_string("assets/simple_3d.json").unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    let mut mirrors =
        Vec::<PlaneMirror>::from_json(value.get("mirrors").expect("mirrors field expected"))
            .expect("expected data in mirrors field to be well-formed");

    let mut ray = Ray::<DEFAULT_DIM>::from_json(value.get("ray").unwrap()).unwrap();

    let mut rays = vec![ray];
    let reflection_limit = 100;

    for _ in 0..reflection_limit {
        let mut intersections = mirrors.intersecting_points(&ray);
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

    events_loop.run(move |ev, _, control_flow| match ev {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                if physical_size.width > 0 && physical_size.height > 0 {
                    projection.resize(physical_size.width, physical_size.height);
                }

                display.gl_window().resize(physical_size)
            }
            WindowEvent::MouseWheel { delta, .. } => {
                camera_controller.process_scroll(&delta);
            }
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    camera_controller.process_keyboard(keycode, input.state);
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if button == MouseButton::Left {
                    mouse_pressed = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    }
                }
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {
            let now = Instant::now();
            let dt = now - last_render_time;
            last_render_time = now;

            let elapsed_time = dt.as_millis() as u64;

            let wait_millis = match 1000 / 244 >= elapsed_time {
                true => 1000 / 244 - elapsed_time,
                false => 0,
            };
            let new_inst = now + Duration::from_millis(wait_millis);
            *control_flow = ControlFlow::WaitUntil(new_inst);

            update(dt, &mut camera, &mut camera_controller);
            render(
                &display,
                &mut program3d,
                &camera,
                &projection,
                &rays,
                &mirrors,
            );
        }
        Event::MainEventsCleared => display.gl_window().window().request_redraw(),
        Event::DeviceEvent {
            event: MouseMotion { delta, .. },
            ..
        } => {
            if mouse_pressed {
                camera_controller.process_mouse(delta.0, delta.1)
            }
        }
        _ => {}
    });

    /*
    let mut fg = Figure::new();

    render_gnu_plot(&mut fg, &rays, &mirrors);

    fg.show().unwrap();*/
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

fn update(dt: Duration, camera: &mut Camera, camera_controller: &mut CameraController) {
    camera_controller.update_camera(camera, dt);
}

fn render(
    display: &glium::backend::glutin::Display,
    program3d: &mut Program,
    camera: &Camera,
    projection: &Projection,
    rays: &Vec<Ray>,
    mirrors: &Vec<PlaneMirror>,
) {
    let mut target = display.draw();
    target.clear_color_and_depth((1.0, 0.95, 0.7, 1.0), 1.0);
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

    //target.draw(&cuboid, &cuboid, &program3d, &uniform! {perspective: perspective, view: view}, &params).unwrap();

    //target.draw(&circle, &circle, &program3d, &uniform! {perspective: perspective, view: view}, &Default::default()).unwrap();

    let mut ray_vec: Vec<Vertex> = rays
        .iter()
        .map(|r| Vertex {
            position: [r.origin.x, r.origin.y, r.origin.z],
        })
        .collect();

    if let Some(last) = ray_vec.last() {
        let [x, y, z] = last.position;
        let mut direction = rays.last().unwrap().direction;
        ray_vec.push(Vertex {
            position: [
                x + direction.x * 1000.0,
                y + direction.y * 1000.0,
                z + direction.z * 1000.0,
            ],
        });
    }

    let vertex_buffer = glium::VertexBuffer::new(display, &ray_vec).unwrap();
    let indices_linestrip = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);
    let indices_trianglestrip = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    target
        .draw(
            &vertex_buffer,
            &indices_linestrip,
            &program3d,
            &uniform! {perspective: perspective, view: view, color_vec: [0.7f32, 0.3f32, 0.1f32]},
            &params,
        )
        .unwrap();

    for mirror in mirrors {
        let vertices: Vec<Vertex> = mirror
            .get_vertices()
            .iter()
            .map(|vector| Vertex {
                position: [vector.x, vector.y, vector.z],
            })
            .collect();

        for x in &vertices {
            println!("{:?}", x);
        }

        // test : correctly places the 2nd mirror
        //let vertices: Vec<Vertex> = vec![[1.0, 1.0, 1.0], [-1.0, -1.0, 1.0]].iter().map(|v| Vertex { position: *v }).collect();

        let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();
        target.draw(
            &vertex_buffer,
            indices_trianglestrip,
            &program3d,
            &uniform! {perspective: perspective, view: view, color_vec: [0.3f32, 0.3f32, 0.9f32]},
            &params
        ).expect("ooooooo c'est la panique");

        /*
        let rot = Rotation3::rotation_between(&Vector3::new(1.0, 1.0, 1.0), &Vector3::new(mirror.plane.v_0().x, mirror.plane.v_0().y, 1.0)).unwrap();
        CuboidBuilder::new()
            .translate(mirror.plane.v_0().x, mirror.plane.v_0().y, 1.0)
            .scale(mirror.bounds[0], mirror.bounds[1], 1.0)
            .build(display)
            .unwrap();
        target.draw(&cuboid, &cuboid, &program3d, &uniform! {perspective: perspective, view: view}, &params).unwrap();*/
    }

    target.finish().unwrap();

    display.gl_window().window().request_redraw();
}
