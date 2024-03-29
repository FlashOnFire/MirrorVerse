#![allow(warnings)]

extern crate alloc;

use cgmath as cg;
use core::cmp;
use glium::{
    self as gl,
    glutin::{self, event, event_loop},
};
use nalgebra::Point3;
use std::time;

use mirror::{plane::PlaneMirror, Mirror, Ray};
use render::camera::{Camera, CameraController, Projection};

mod mirror;
mod render;

pub const DEFAULT_DIM: usize = 3;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
}
gl::implement_vertex!(Vertex, position);

impl Vertex {
    pub fn from_vector(v: nalgebra::SVector<f32, 3>) -> Self {
        Self {
            position: [v.x, v.y, v.z],
        }
    }
}

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
    let mut events_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(1280.0, 720.0))
        .with_title("MirrorVerse");
    let cb = glutin::ContextBuilder::new();
    let display = gl::Display::new(wb, cb, &events_loop).unwrap();

    let mut camera = Camera::new(Point3::new(0.0, 0.0, 0.0), cg::Deg(90.0), cg::Deg(0.0));

    let mut projection = Projection::new(1280, 720, cg::Deg(70.0), 0.1, 100.0);
    let mut camera_controller = CameraController::new(5.0, 0.4);

    let mut program3d =
        gl::Program::from_source(&display, VERTEX_SHADER_SRC_3D, FRAGMENT_SHADER_SRC, None)
            .unwrap();

    let mut last_render_time = time::Instant::now();

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
        //sort the intersections by distance using distance_to_point
        intersections.sort_by(|a, b| {
            a.1.distance_to_ray(ray)
                .partial_cmp(&b.1.distance_to_ray(ray))
                .unwrap_or(cmp::Ordering::Equal)
        });
        if let Some((darkness, plane)) = intersections.first() {
            let reflected_ray = ray.reflect(plane, darkness);
            rays.push(reflected_ray);
            ray = reflected_ray;
        } else {
            break;
        }
    }

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
                    mouse_pressed = match state {
                        event::ElementState::Pressed => true,
                        event::ElementState::Released => false,
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

            let wait_millis = match 1000 / 244 >= elapsed_time {
                true => 1000 / 244 - elapsed_time,
                false => 0,
            };
            let new_inst = now + time::Duration::from_millis(wait_millis);
            *control_flow = event_loop::ControlFlow::WaitUntil(new_inst);

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
        event::Event::MainEventsCleared => display.gl_window().window().request_redraw(),
        event::Event::DeviceEvent {
            event: event::DeviceEvent::MouseMotion { delta, .. },
            ..
        } => {
            if mouse_pressed {
                camera_controller.process_mouse(delta.0, delta.1)
            }
        }
        _ => {}
    });
}

fn update(dt: time::Duration, camera: &mut Camera, camera_controller: &mut CameraController) {
    camera_controller.update_camera(camera, dt);
}

fn render(
    display: &gl::backend::glutin::Display,
    program3d: &mut gl::Program,
    camera: &Camera,
    projection: &Projection,
    rays: &Vec<Ray>,
    mirrors: &Vec<PlaneMirror>,
) {
    let mut target = display.draw();

    use gl::Surface;
    target.clear_color_and_depth((1.0, 0.95, 0.7, 1.0), 1.0);

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

    let mat = cg::perspective(cg::Deg(45.0), (16.0) / (9.0), 1000.0, 0.1);
    let perspective: [[f32; 4]; 4] = mat.into();
    let view: [[f32; 4]; 4] = camera.calc_matrix().into();

    let params = gl::DrawParameters {
        depth: gl::Depth {
            test: gl::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        ..Default::default()
    };

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

    let vertex_buffer = gl::VertexBuffer::new(display, &ray_vec).unwrap();
    let indices_linestrip = gl::index::NoIndices(gl::index::PrimitiveType::LineStrip);
    let indices_trianglestrip = gl::index::NoIndices(gl::index::PrimitiveType::TriangleStrip);

    target
        .draw(
            &vertex_buffer,
            &indices_linestrip,
            &program3d,
            &gl::uniform! {perspective: perspective, view: view, color_vec: [0.7f32, 0.3f32, 0.1f32]},
            &params,
        )
        .unwrap();

    for mirror in mirrors {
        let vertices: Vec<_> = mirror
            .get_vertices()
            .iter()
            .copied()
            .map(Vertex::from_vector)
            .collect();

        let vertex_buffer = gl::VertexBuffer::new(display, &vertices).unwrap();
        target.draw(
            &vertex_buffer,
            indices_trianglestrip,
            &program3d,
            &gl::uniform! {perspective: perspective, view: view, color_vec: [0.3f32, 0.3f32, 0.9f32]},
            &params
        ).expect("ooooooo c'est la panique");
    }

    target.finish().unwrap();

    display.gl_window().window().request_redraw();
}
