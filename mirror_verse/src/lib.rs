extern crate alloc;

// re-export deps for convenience
pub mod mirror;
pub mod render;
pub use glium as gl;
pub use nalgebra;
pub use rand;
pub use serde_json;

use cgmath as cg;
use core::{array, iter};
use gl::glutin::{self, dpi::PhysicalPosition, event, event_loop, window::CursorGrabMode};
use glium_shapes::sphere::SphereBuilder;
use nalgebra::{Point, SMatrix, SVector, Unit};
use std::{error::Error, time};

use render::{
    camera::{Camera, CameraController, Projection},
    DrawableSimulation, RayRenderData, RenderData,
};

use mirror::{util, Mirror, Ray};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RayPath<const D: usize> {
    points: Vec<SVector<f32, D>>,
    loop_start: Option<usize>,
    divergence_direction: Option<Unit<SVector<f32, D>>>,
}

impl<const D: usize> RayPath<D> {
    pub fn all_points_raw(&self) -> &[SVector<f32, D>] {
        self.points.as_slice()
    }

    /// returns a pair (non_loop_points, loop_points)
    pub fn all_points(&self) -> (&[SVector<f32, D>], &[SVector<f32, D>]) {
        self.points
            .split_at(self.loop_start.unwrap_or(self.points.len()))
    }

    // name bikeshedding welcome

    pub fn non_loop_points(&self) -> &[SVector<f32, D>] {
        &self.points[..self.loop_start.unwrap_or(self.points.len())]
    }

    pub fn loop_points(&self) -> &[SVector<f32, D>] {
        self.loop_start
            .map(|index| &self.points[index..])
            .unwrap_or_default()
    }

    pub fn divergence_direction(&self) -> Option<&Unit<SVector<f32, D>>> {
        self.divergence_direction.as_ref()
    }

    pub fn push_point(&mut self, pt: SVector<f32, D>) {
        self.points.push(pt);
    }

    /// Attempts to push a point to the path. If it's on a previously followed path, aborts,
    /// registers the section of the path that loops, and returns `false`
    pub fn try_push_point(&mut self, pt: SVector<f32, D>, epsilon: f32) -> bool {
        let maybe_loop_index = self.points.split_last().and_then(|(last_pt, points)| {
            points.windows(2).enumerate().find_map(|(i, window)| {
                // ugly, but `slice::array_windows` is unstable
                let [this_pt, next_pt] = window else {
                    // because window.len() is always 2
                    unreachable!()
                };
                ((last_pt - this_pt).norm() < epsilon && (pt - next_pt).norm() < epsilon)
                    .then_some(i)
            })
        });

        if let Some(loop_index) = maybe_loop_index {
            self.loop_start = Some(loop_index);
        } else {
            self.push_point(pt);
        }

        maybe_loop_index.is_none()
    }

    pub fn set_divergence_direction(&mut self, dir: Unit<SVector<f32, D>>) -> bool {
        let first_time = self.divergence_direction.is_none();
        self.divergence_direction = Some(dir);
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
        const MAX_NUM_RAYS: usize = 32;
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
                        ray_path.set_divergence_direction(ray.direction);
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

            "mirror": self.mirror.to_json()?,
        }))
    }

    fn ray_render_data(
        &self,
        reflaction_limit: usize,
        display: &gl::Display,
    ) -> Vec<RayRenderData<3>> {
        self.get_ray_paths(reflaction_limit)
            .into_iter()
            .map(|ray_path| {
                // we'll change the dimension logic in the future
                let v = ray_path.all_points_raw().first().unwrap();
                let [x, y, z] = array::from_fn(|i| if i < D { v[i] } else { 0.0 });

                let (non_loop_pts, loop_pts) = ray_path.all_points();

                let non_loop_points = Vec::from_iter(
                    non_loop_pts
                        .iter()
                        .copied()
                        .chain(
                            ray_path
                                .divergence_direction()
                                .map(|dir| non_loop_pts.last().unwrap() + dir.as_ref() * 2000.),
                        )
                        .map(render::Vertex::<3>::from),
                );
                let loop_points =
                    Vec::from_iter(loop_pts.iter().copied().map(render::Vertex::<3>::from));

                RayRenderData {
                    origin: SphereBuilder::new()
                        .scale(0.1, 0.1, 0.1)
                        .translate(x, y, z)
                        .with_divisions(60, 60)
                        .build(display)
                        .unwrap(),
                    non_loop_path: gl::VertexBuffer::immutable(display, non_loop_points.as_slice())
                        .unwrap(),
                    loop_path: gl::VertexBuffer::immutable(display, loop_points.as_slice())
                        .unwrap(),
                }
            })
            .collect()
    }

    pub fn mirror_render_data(&self, display: &gl::Display) -> Vec<Box<dyn RenderData<3>>> {
        self.mirror.render_data(display)
    }

    fn to_drawable(&self, reflection_limit: usize, display: &gl::Display) -> DrawableSimulation<3> {
        DrawableSimulation::<3>::new(
            self.ray_render_data(reflection_limit, display),
            self.mirror_render_data(display),
        )
    }

    pub fn run_opengl(&self, reflection_limit: usize) {
        let events_loop = glutin::event_loop::EventLoop::new();

        const DEFAULT_WIDTH: u32 = 1280;
        const DEFAULT_HEIGHT: u32 = 720;

        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            .with_title("MirrorVerse");

        let cb = glutin::ContextBuilder::new().with_vsync(true);

        let display = gl::Display::new(wb, cb, &events_loop).unwrap();

        let drawable_simulation = self.to_drawable(reflection_limit, &display);

        const DEFAULT_CAMERA_POS: cg::Point3<f32> = cg::Point3::new(0., 0., 5.);
        const DEFAULT_CAMERA_YAW: cg::Deg<f32> = cg::Deg(-90.);
        const DEFAULT_CAMERA_PITCH: cg::Deg<f32> = cg::Deg(0.);

        let mut camera = Camera::new(DEFAULT_CAMERA_POS, DEFAULT_CAMERA_YAW, DEFAULT_CAMERA_PITCH);

        const DEFAULT_PROJECCTION_POV: cg::Deg<f32> = cg::Deg(85.);
        const NEAR_PLANE: f32 = 0.0001;
        const FAR_PLANE: f32 = 10000.;

        let mut projection = Projection::new(
            DEFAULT_WIDTH,
            DEFAULT_HEIGHT,
            DEFAULT_PROJECCTION_POV,
            NEAR_PLANE,
            FAR_PLANE,
        );

        const SPEED: f32 = 5.;
        const MOUSE_SENSITIVITY: f32 = 4.0;

        let mut camera_controller = CameraController::new(SPEED, MOUSE_SENSITIVITY);

        let program3d = gl::Program::from_source(
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
                drawable_simulation.render_3d(&display, &program3d, &camera, &projection);
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
