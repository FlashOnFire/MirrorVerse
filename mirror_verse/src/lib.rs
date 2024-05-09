extern crate alloc;
pub mod mirror;
// re-export deps for convenience
pub use rand;
pub use serde_json;
mod render;

use cgmath as cg;
use core::iter;
use glium::{
    self as gl,
    glutin::{self, dpi::PhysicalPosition, event, event_loop, window::CursorGrabMode},
};
use glium_shapes::sphere::SphereBuilder;
use nalgebra::{SVector, Unit};
use std::{error::Error, time};

use render::{
    camera::{Camera, CameraController, Projection},
    DrawableSimulation,
};

use mirror::{util, Mirror, Ray};

const DEFAULT_WIDTH: u32 = 1280;
const DEFAULT_HEIGHT: u32 = 720;

const NEAR_PLANE: f32 = 0.0001;
const FAR_PLANE: f32 = 10000.;

const SPEED: f32 = 5.;
const MOUSE_SENSITIVITY: f32 = 4.0;

const DEFAULT_CAMERA_POS: cg::Point3<f32> = cg::Point3::new(0., 0., 5.);
const DEFAULT_CAMERA_YAW: cg::Deg<f32> = cg::Deg(-90.);
const DEFAULT_CAMERA_PITCH: cg::Deg<f32> = cg::Deg(0.);
const PROJECTION_FOV: cg::Deg<f32> = cg::Deg(85.);

const ORIGIN_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const RAY_COLOR: [f32; 4] = [0.7, 0.3, 0.1, 0.6];
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

    /// Attempts to push a point to the path. Aborts and
    /// returns `false` if it's on a previously followed path
    pub fn try_push_point(&mut self, pt: SVector<f32, D>, epsilon: f32) -> bool {
        let no_dupes = !self
            .points
            .split_last()
            .map(|(last_pt, points)| {
                points.windows(2).any(|window| {
                    // ugly, but `slice::array_windows` is unstable
                    let [this_pt, next_pt] = window else {
                        // because window.len() is always 2
                        unreachable!()
                    };
                    (last_pt - this_pt).norm() < epsilon && (pt - next_pt).norm() < epsilon
                })
            })
            .unwrap_or(false);

        if no_dupes {
            self.push_point(pt)
        }

        no_dupes
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
    pub fn random<U: rand::Rng + ?Sized>(rng: &mut U) -> Self {
        const MIN_NUM_RAYS: usize = 1;
        const MAX_NUM_RAYS: usize = 16;
        let num_rays = rng.gen_range(MIN_NUM_RAYS..MAX_NUM_RAYS);
        Self {
            rays: iter::repeat_with(|| Ray::random(rng))
                .take(num_rays)
                .collect(),
            mirror: T::random(rng),
        }
    }

    pub fn get_ray_paths(&self, reflection_limit: usize) -> Vec<RayPath<D>> {
        let mut intersections_scratch = vec![];
        self.rays
            .iter()
            .map(|ray| {
                let mut ray = *ray;
                let mut ray_path = RayPath::default();
                ray_path.push_point(ray.origin);

                for _n in 0..reflection_limit {
                    intersections_scratch.clear();
                    self.mirror
                        .append_intersecting_points(&ray, &mut intersections_scratch);

                    if let Some((distance, tangent)) = intersections_scratch
                        .iter()
                        .filter_map(|tangent| {
                            let d = tangent
                                .try_intersection_distance(&ray)
                                .expect("the ray must intersect with the plane");
                            (d > f32::EPSILON * 16.0).then_some((d, tangent))
                        })
                        .min_by(|(d1, _), (d2, _)| {
                            d1.partial_cmp(d2)
                                .expect("NaN found in intersection distances: aborting")
                        })
                    {
                        ray.advance(distance);
                        if !ray_path.try_push_point(ray.origin, f32::EPSILON * 16.0) {
                            break;
                        }
                        ray.reflect_direction(tangent);
                    } else {
                        ray_path.set_final_direction(ray.direction);
                        break;
                    }
                }
                ray_path
            })
            .collect()
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
        .ok_or("failed to deserialize ray")?;

        Ok(Self { mirror, rays })
    }

    pub fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        Ok(serde_json::json!({

            "rays": util::try_collect(
                self.rays.iter().map(Ray::to_json).map(Result::ok)
            ).ok_or("failed to serialize a ray")?,

            "mirror": self.mirror.to_json()?
        }))
    }

    // trop de racisme ici
    fn to_drawable(&self, reflection_limit: usize, display: &gl::Display) -> DrawableSimulation<3> {
        assert!(D <= 3);

        // hein?
        // We first check if there is at least one ray to display its origin
        let origins = if self.rays.is_empty() {
            vec![]
        } else {
            // if it's the case, we loop through all of them and build a small sphere around the origin of the ray
            Vec::from_iter(
                self.rays
                    .iter()
                    .map(|r| r.origin)
                    .map(|v| {
                        if D == 1 {
                            [*v.get(0).unwrap(), 0., 0.]
                        } else if D == 2 {
                            [*v.get(0).unwrap(), *v.get(1).unwrap(), 0.]
                        } else {
                            [*v.get(0).unwrap(), *v.get(1).unwrap(), *v.get(2).unwrap()]
                        }
                    })
                    .map(|v| {
                        SphereBuilder::new()
                            .scale(0.05, 0.05, 0.05)
                            .translate(v[0], v[1], v[2])
                            .with_divisions(20, 20)
                            .build(display)
                            .unwrap()
                    }),
            )
        };

        // Then we calculate the path of each ray
        let ray_paths = self
            .get_ray_paths(reflection_limit)
            .into_iter()
            .map(|ray_path| {
                gl::VertexBuffer::new(
                    display,
                    &Vec::from_iter(
                        ray_path
                            .points()
                            .iter()
                            .copied()
                            .chain(ray_path.final_direction().map(|dir| {
                                ray_path.points().last().unwrap() + dir.as_ref() * 2000.
                            }))
                            .map(render::Vertex::<3>::from),
                    ),
                )
                .unwrap()
            })
            .collect();

        // finally we build the render data of each mirror
        let mirrors = self.mirror.render_data(display);

        DrawableSimulation::<3>::new(origins, ray_paths, mirrors)
    }

    pub fn run_opengl(&self, reflection_limit: usize) {
        let events_loop = glutin::event_loop::EventLoop::new();

        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            .with_title("MirrorVerse");

        let cb = glutin::ContextBuilder::new().with_vsync(true);

        let display = gl::Display::new(wb, cb, &events_loop).unwrap();

        let drawable_simulation = self.to_drawable(reflection_limit, &display);

        let mut camera = Camera::new(DEFAULT_CAMERA_POS, DEFAULT_CAMERA_YAW, DEFAULT_CAMERA_PITCH);

        let mut projection = Projection::new(
            DEFAULT_WIDTH,
            DEFAULT_HEIGHT,
            PROJECTION_FOV,
            NEAR_PLANE,
            FAR_PLANE,
        );

        let mut camera_controller = CameraController::new(SPEED, MOUSE_SENSITIVITY);

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
                    camera_controller.set_scroll(&delta);
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
