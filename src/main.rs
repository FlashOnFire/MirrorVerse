extern crate alloc;
mod mirror;
mod render;

use cgmath as cg;
use glium::{
    self as gl,
    glutin::{self, dpi::PhysicalPosition, event, event_loop, window::CursorGrabMode},
};
use nalgebra::{SVector, Unit};
use std::{error::Error, fs::File, time};

use render::{
    camera::{Camera, CameraController, Projection},
    DrawableSimulation,
};

use mirror::{util, Mirror, Ray};

const DEFAULT_DIM: usize = 3;

const TARGET_FPS: u64 = 288;

const DEFAULT_WIDTH: u32 = 1280;
const DEFAULT_HEIGHT: u32 = 720;

const NEAR_PLANE: f32 = 0.1;
const FAR_PLANE: f32 = 2000.;

const SPEED: f32 = 5.;
const MOVEMENT_SENSITIVITY: f32 = 3.0;
const MOUSE_SENSITIVITY: f32 = 4.0;

const DEFAULT_CAMERA_POS: cg::Point3<f32> = cg::Point3::new(0., 0., 0.);
const DEFAULT_CAMERA_YAW: cg::Deg<f32> = cg::Deg(0.);
const DEFAULT_CAMERA_PITCH: cg::Deg<f32> = cg::Deg(0.);
const PROJECTION_FOV: cg::Deg<f32> = cg::Deg(85.);

const RAY_COLOR: [f32; 4] = [0.7, 0.3, 0.1, 6.0];
const MIRROR_COLOR: [f32; 4] = [0.3, 0.3, 0.9, 0.4];

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RayPath<const D: usize> {
    points: Vec<SVector<f32, D>>,
    final_direction: Option<Unit<SVector<f32, D>>>,
}

impl<const D: usize> RayPath<D> {
    pub fn points(&self) -> &[SVector<f32, D>] {
        self.points.as_slice()
    }

    pub fn final_direction(&self) -> Option<&Unit<SVector<f32, D>>> {
        self.final_direction.as_ref()
    }

    pub fn push_point(&mut self, pt: SVector<f32, D>) {
        self.points.push(pt);
    }

    pub fn set_final_direction(&mut self, dir: Unit<SVector<f32, D>>) -> bool {
        let first_time = self.final_direction.is_none();
        self.final_direction = Some(dir);
        first_time
    }
}

pub struct Simulation<T, const D: usize> {
    pub rays: Vec<Ray<D>>,
    pub mirror: T,
}

impl<const D: usize, T: Mirror<D>> Simulation<T, D> {
    pub fn get_ray_paths(&self, reflection_limit: usize) -> Vec<RayPath<D>> {
        let mut intersections = vec![];
        let mut ray_paths = vec![RayPath::default(); self.rays.len()];

        // TODO: clean this up

        for (ray, ray_path) in self.rays.iter().zip(ray_paths.iter_mut()) {
            let mut ray = *ray;

            for _n in 0..reflection_limit {
                ray_path.push_point(ray.origin);

                self.mirror
                    .append_intersecting_points(&ray, &mut intersections);

                if let Some((distance, tangent)) = intersections
                    .iter()
                    .filter_map(|tangent| {
                        let dist = tangent
                            .try_intersection_distance(&ray)
                            .expect("the ray must intersect with the plane");
                        (dist > f32::EPSILON * 16.0).then_some((dist, tangent))
                    })
                    .min_by(|(d1, _), (d2, _)| {
                        d1.partial_cmp(d2)
                            .expect("NaN found in intersection distances: aborting")
                    })
                {
                    ray.advance(distance);
                    ray.reflect_direction(tangent);
                } else {
                    ray_path.set_final_direction(ray.direction);
                    break;
                }

                intersections.clear()
            }

            // if we were capped by the reflection limit, our last position wasn't saved
            if ray_path.final_direction().is_none() {
                ray_path.push_point(ray.origin)
            }
        }

        ray_paths
    }

    pub fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        let mirror = T::from_json(json.get("mirror").ok_or("mirror field expected")?)?;

        let rays = util::try_collect(
            json.get("rays")
                .ok_or("rays field not found")?
                .as_array()
                .ok_or("`rays` field must be an array")?
                .iter()
                .map(Ray::from_json)
                .map(Result::ok),
        )
        .ok_or("failed to deserialize a ray")?;

        Ok(Self { mirror, rays })
    }

    pub fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        todo!()
    }
}

impl<const D: usize, T: mirror::Mirror<D>> Simulation<T, D>
where
    render::Vertex<D>: gl::Vertex,
{
    fn into_drawable(
        &self,
        reflection_limit: usize,
        display: &gl::Display,
    ) -> DrawableSimulation<render::Vertex<D>> {
        let mut vertex_scratch = vec![];

        DrawableSimulation::new(
            self.get_ray_paths(reflection_limit)
                .into_iter()
                .map(|ray_path| {
                    vertex_scratch.extend(
                        ray_path
                            .points()
                            .iter()
                            .copied()
                            .chain(ray_path.final_direction().map(|dir| {
                                ray_path.points().last().unwrap() + dir.as_ref() * 2000.
                            }))
                            .map(render::Vertex::from),
                    );

                    let vertex_buf = gl::VertexBuffer::new(display, &vertex_scratch).unwrap();

                    vertex_scratch.clear();

                    vertex_buf
                })
                .collect(),
            self.mirror.render_data(display),
        )
    }

    fn run(&self, reflection_limit: usize) {
        let events_loop = glutin::event_loop::EventLoop::new();

        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            .with_title("MirrorVerse");

        let cb = glutin::ContextBuilder::new().with_vsync(true);

        let display = gl::Display::new(wb, cb, &events_loop).unwrap();

        let drawable_simulation = self.into_drawable(reflection_limit, &display);

        let mut camera = Camera::new(DEFAULT_CAMERA_POS, DEFAULT_CAMERA_YAW, DEFAULT_CAMERA_PITCH);

        let mut projection = Projection::new(
            DEFAULT_WIDTH,
            DEFAULT_HEIGHT,
            PROJECTION_FOV,
            NEAR_PLANE,
            FAR_PLANE,
        );

        let mut camera_controller =
            CameraController::new(SPEED, MOVEMENT_SENSITIVITY, MOUSE_SENSITIVITY);

        let mut program3d = gl::Program::from_source(
            &display,
            render::VERTEX_SHADER_SRC_3D,
            render::FRAGMENT_SHADER_SRC,
            None,
        )
        .unwrap();

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
                    camera_controller.set_scoll(&delta);
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

                camera_controller.update_camera(&mut camera, dt);
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
                    camera_controller.set_mouse_delta(delta.0, delta.1)
                }
            }
            _ => (),
        });
    }
}

fn main() {
    // Load the mirror list from the json file
    let file_path = std::env::args()
        .nth(1)
        .expect("Please provide a file path as a command-line argument.");

    let simulation = Simulation::<Box<dyn Mirror<DEFAULT_DIM>>, DEFAULT_DIM>::from_json(
        &serde_json::from_reader(File::open(file_path).unwrap()).unwrap(),
    )
    .unwrap();

    simulation.run(500);
}
