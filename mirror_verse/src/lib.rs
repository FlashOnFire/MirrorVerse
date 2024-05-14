extern crate alloc;

// re-export deps for convenience
pub mod mirror;
pub mod render;
pub use glium as gl;
pub use nalgebra;
pub use rand;
pub use serde_json;

use cgmath as cg;
use core::iter;
use gl::glutin::{self, dpi::PhysicalPosition, event, event_loop, window::CursorGrabMode};
use nalgebra::{SMatrix, SVector, Unit};
use serde_json::json;
use std::{error::Error, time};

use render::{
    camera::{Camera, CameraController, Projection}, DrawableSimulation
};

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

    pub fn causes_loop_at(&self, pt: SVector<f32, D>, epsilon: f32) -> Option<usize> {
        self.points.split_last().and_then(|(last_pt, points)| {
            points.windows(2).enumerate().find_map(|(i, window)| {
                // ugly, but `slice::array_windows` is unstable
                let [this_pt, next_pt] = window else {
                    // because window.len() is always 2
                    unreachable!()
                };
                ((last_pt - this_pt).norm() < epsilon && (pt - next_pt).norm() < epsilon)
                    .then_some(i)
            })
        })
    }

    /// Attempts to push a point to the path. If it's on a previously followed path, aborts,
    /// registers the section of the path that loops, and returns `false`
    pub fn try_push_point(&mut self, pt: SVector<f32, D>, epsilon: f32) -> bool {
        let maybe_loop_index = self.causes_loop_at(pt, epsilon);

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

    pub(crate) fn path_vertices(&self, display: &gl::Display) -> (gl::VertexBuffer<render::Vertex<D>>, gl::VertexBuffer<render::Vertex<D>>)
    where
        render::Vertex<D>: gl::Vertex,
    {
        let (non_loop_pts, loop_pts) = self.all_points();

        let non_loop_pts = Vec::from_iter(
            non_loop_pts
                .iter()
                .copied()
                .chain(
                    self
                        .divergence_direction()
                        .map(|dir| non_loop_pts.last().unwrap() + dir.as_ref() * 2000.),
                )
                .map(render::Vertex::from),
        );
        let loop_pts =
            Vec::from_iter(loop_pts.iter().copied().map(render::Vertex::from));

        (
            gl::VertexBuffer::immutable(display, non_loop_pts.as_slice()).unwrap(),
            gl::VertexBuffer::immutable(display, loop_pts.as_slice()).unwrap()
        )
    }
}

pub struct Simulation<T, const D: usize> {
    pub rays: Vec<mirror::Ray<D>>,
    pub mirror: T,
}

impl<const D: usize, T: mirror::Random> Simulation<T, D> {
    pub fn random<U: rand::Rng + ?Sized>(rng: &mut U) -> Self {
        const MIN_NUM_RAYS: usize = 1;
        const MAX_NUM_RAYS: usize = 32;
        let num_rays = rng.gen_range(MIN_NUM_RAYS..MAX_NUM_RAYS);
        Self {
            rays: iter::repeat_with(|| mirror::Ray::random(rng))
                .take(num_rays)
                .collect(),
            mirror: T::random(rng),
        }
    }
}

impl<const D: usize, T: mirror::JsonDes> Simulation<T, D> {
    pub fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        let mirror = T::from_json(json.get("mirror").ok_or("mirror field expected")?)?;

        let rays = util::try_collect(
            json.get("rays")
                .ok_or("rays field not found")?
                .as_array()
                .ok_or("`rays` field must be an array")?
                .iter()
                .map(mirror::Ray::from_json)
                .map(Result::ok),
        )
        .ok_or("failed to deserialize ray")?;

        Ok(Self { mirror, rays })
    }
}

impl<const D: usize, T: mirror::JsonSer> Simulation<T, D> {
    pub fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {

        Ok(serde_json::json!({
            "dim": D,
            "rays": util::try_collect(
                self.rays.iter().map(mirror::Ray::to_json).map(Result::ok)
            ).ok_or("failed to serialize a ray")?,

            "mirror": self.mirror.to_json()?,
        }))
    }
}

impl<const D: usize, T: mirror::Mirror<D>> Simulation<T, D> {
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

                    // TODO: make some of these error messages more useful

                    if let Some((distance, tangent)) = intersections_scratch
                        .iter()
                        .filter_map(|tangent| {
                            let d = tangent
                                .try_intersection_distance(&ray)
                                .expect("a mirror returned a plane parallel to the ray: aborting");
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
}

impl<T: mirror::Mirror<3>> Simulation<T, 3> {
    fn ray_render_data(
        &self,
        reflection_limit: usize,
        display: &gl::Display,
    ) -> Vec<render::RayRenderData<3>> {
        self.get_ray_paths(reflection_limit)
            .into_iter()
            .map(|ray_path| {
                // we'll change this to a square or circle that's doesn't get scaled by the projection matrix
                // use Sphere for 3D, and Circle for 2D
                let [x, y, z] = ray_path.all_points_raw().first().copied().unwrap().into();

                let (non_loop_path, loop_path) = ray_path.path_vertices(display);

                render::RayRenderData {
                    origin: Box::new(glium_shapes::sphere::SphereBuilder::new()
                        .scale(0.1, 0.1, 0.1)
                        .translate(x, y, z)
                        .with_divisions(60, 60)
                        .build(display)
                        .unwrap()),
                    non_loop_path,
                    loop_path,
                }
            })
            .collect()
    }
}

impl<T: mirror::Mirror<2>> Simulation<T, 2> {
    fn ray_render_data(
        &self,
        reflection_limit: usize,
        display: &gl::Display,
    ) -> Vec<render::RayRenderData<2>> {
        self.get_ray_paths(reflection_limit)
            .into_iter()
            .map(|ray_path| {
                // we'll change this to a square or circle that's doesn't get scaled by the projection matrix
                // use Sphere for 3D, and Circle for 2D
                let center = ray_path.all_points_raw().first().copied().unwrap().into();

                let (non_loop_path, loop_path) = ray_path.path_vertices(display);

                render::RayRenderData {
                    origin: Box::new(render::FilledCircle::from(render::Circle::new(center, 0.1, display))),
                    non_loop_path,
                    loop_path,
                }
            })
            .collect()
    }
}

impl<T: mirror::Mirror<2> + render::OpenGLRenderable> Simulation<T, 2> {
    fn to_drawable(&self, reflection_limit: usize, display: &gl::Display) -> DrawableSimulation<2> {
        let program = gl::Program::from_source(
            display,
            render::VERTEX_SHADER_SRC_2D,
            render::FRAGMENT_SHADER_SRC,
            None,
        )
        .unwrap();

        DrawableSimulation::new(
            self.ray_render_data(reflection_limit, display),
            self.mirror_render_data(display),
            program,
        )
    }

    pub fn run_opengl_3d(&self, reflection_limit: usize) {
        let events_loop = glutin::event_loop::EventLoop::new();

        const DEFAULT_WIDTH: u32 = 1280;
        const DEFAULT_HEIGHT: u32 = 720;

        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            .with_title("MirrorVerse");

        let cb = glutin::ContextBuilder::new().with_vsync(true);

        let display = gl::Display::new(wb, cb, &events_loop).unwrap();

        let drawable_simulation = self.to_drawable(reflection_limit, &display);

        drawable_simulation.run(display, events_loop);
    }
}

impl<const D: usize, T: render::OpenGLRenderable> Simulation<T, D> {
    pub fn mirror_render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        self.mirror.render_data(display)
    }
}

impl<T: mirror::Mirror<3> + render::OpenGLRenderable> Simulation<T, 3> {
    fn to_drawable(&self, reflection_limit: usize, display: &gl::Display) -> DrawableSimulation<3> {
        let program = gl::Program::from_source(
            display,
            render::VERTEX_SHADER_SRC_3D,
            render::FRAGMENT_SHADER_SRC,
            None,
        )
        .unwrap();

        DrawableSimulation::new(
            self.ray_render_data(reflection_limit, display),
            self.mirror_render_data(display),
            program,
        )
    }

    pub fn run_opengl_3d(&self, reflection_limit: usize) {
        let events_loop = glutin::event_loop::EventLoop::new();

        const DEFAULT_WIDTH: u32 = 1280;
        const DEFAULT_HEIGHT: u32 = 720;

        let wb = glutin::window::WindowBuilder::new()
            .with_inner_size(glutin::dpi::LogicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
            .with_title("MirrorVerse");

        let cb = glutin::ContextBuilder::new().with_vsync(true);

        let display = gl::Display::new(wb, cb, &events_loop).unwrap();

        let drawable_simulation = self.to_drawable(reflection_limit, &display);

        drawable_simulation.run(display, events_loop);
    }
}

pub mod util {
    use super::*;

    pub fn random_vector<T: rand::Rng + ?Sized, const D: usize>(
        rng: &mut T,
        max_coord_mag: f32,
    ) -> SVector<f32, D> {
        // the rng generates floats in 0.0..1.0, scale and translate the range accordingly

        SVector::<f32, D>::from_fn(|_, _| (rng.gen::<f32>() - 0.5) * (max_coord_mag.abs() * 2.0))
    }

    /// This is essentially `try_into` then `try_map` but the latter is nightly-only
    pub fn json_array_to_float_array<const D: usize>(
        json_array: &[serde_json::Value],
    ) -> Option<[f32; D]> {
        let array: &[serde_json::Value; D] = json_array.try_into().ok()?;

        let mut center_coords_array = [0.; D];
        for (coord, value) in center_coords_array.iter_mut().zip(array) {
            *coord = value.as_f64()? as f32;
        }
        Some(center_coords_array)
    }

    /// This is essentially `try_into` then `try_map` but the latter is nightly-only
    pub fn json_array_to_vector<const D: usize>(
        json_array: &[serde_json::Value],
    ) -> Option<SVector<f32, D>> {
        json_array_to_float_array(json_array).map(SVector::from)
    }

    /// This is essentially [`Iterator::try_collect`]
    /// for `Vec<T>` but without having to use nightly
    pub fn try_collect<T>(i: impl Iterator<Item = Option<T>>) -> Option<Vec<T>> {
        let mut vec = vec![];
        for item in i {
            vec.push(item?);
        }

        Some(vec)
    }
}
