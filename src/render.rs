use glium::{Blend, Surface, VertexBuffer};
use glium::index::{NoIndices, PrimitiveType};
use crate::{mirror, MIRROR_COLOR, RAY_COLOR, render};
use crate::render::camera::{Camera, Projection};

use glium as gl;
use crate::mirror::plane::PlaneMirror;

pub mod camera;

const INDICES_LINESTRIP: NoIndices = NoIndices(PrimitiveType::LineStrip);
const INDICES_TRIANGLE_STRIP: NoIndices = NoIndices(PrimitiveType::TriangleStrip);

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
}
glium::implement_vertex!(Vertex, position);

impl From<nalgebra::SVector<f32, 3>> for Vertex {
    fn from(v: nalgebra::SVector<f32, 3>) -> Self {
        Self { position: v.into() }
    }
}

pub const FRAGMENT_SHADER_SRC: &str = r#"
        #version 140

        uniform vec4 color_vec;

        out vec4 color;

        void main() {
            color = color_vec;
        }
    "#;

pub const VERTEX_SHADER_SRC_3D: &str = r#"
        #version 140

        in vec3 position;
        uniform mat4 perspective;
        uniform mat4 view;

        void main() {
            gl_Position = perspective * view * vec4(position, 1.0);
        }
    "#;

pub struct DrawableSimulation {
    rays: Vec<VertexBuffer<Vertex>>,
    mirrors: Vec<VertexBuffer<Vertex>>,
}

impl DrawableSimulation {
    pub fn new(sim: mirror::Simulation<Vec<PlaneMirror>, 3>, reflection_limit: usize, display: &gl::backend::glutin::Display) -> Self {
        let mut rays_vertex_buffers: Vec<VertexBuffer<Vertex>> = vec![];

        for ray_path in sim.get_ray_paths(reflection_limit) {
            let mut ray_path_vertices_vectors: Vec<_> = ray_path.points().to_vec();

            // Add another point far away to render the last line
            if let Some(dir) = ray_path.final_direction() {
                ray_path_vertices_vectors.push(ray_path.points().last().unwrap() + dir.as_ref() * 2000.);
            }

            let ray_path_vertices: Vec<_> = ray_path_vertices_vectors.iter()
                .copied()
                .map(render::Vertex::from)
                .collect();


            rays_vertex_buffers.push(gl::VertexBuffer::new(display, &ray_path_vertices).unwrap());
        }

        let mut mirrors_vertex_buffers: Vec<VertexBuffer<Vertex>> = vec![];

        for mirror in sim.mirror {
            let vertices: Vec<_> = mirror.vertices().map(render::Vertex::from).collect();
            mirrors_vertex_buffers.push(gl::VertexBuffer::new(display, &vertices).unwrap());
        }

        Self {
            rays: rays_vertex_buffers,
            mirrors: mirrors_vertex_buffers,
        }
    }


    pub fn render(
        &self,
        display: &gl::backend::glutin::Display,
        program3d: &mut gl::Program,
        camera: &Camera,
        projection: &Projection,
    ) {
        let mut target = display.draw();

        target.clear_color_and_depth((1., 0.95, 0.7, 1.), 1.0);

        let perspective = projection.get_matrix();
        let view = camera.calc_matrix();

        let params = gl::DrawParameters {
            depth: gl::Depth {
                test: gl::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            line_width: Some(3.),
            blend: Blend::alpha_blending(),
            ..Default::default()
        };

        for buffer in self.rays.as_slice() {
            target.draw(
                buffer,
                INDICES_LINESTRIP,
                program3d,
                &gl::uniform! {
                    perspective: perspective,
                    view: view,
                    color_vec: RAY_COLOR,
                },
                &params,
            ).unwrap();
        }

        for buffer in self.mirrors.as_slice() {
            target.draw(
                buffer,
                INDICES_TRIANGLE_STRIP,
                program3d,
                &gl::uniform! {
                    perspective: perspective,
                    view: view,
                    color_vec: MIRROR_COLOR,
                },
                &params,
            ).unwrap();
        }

        target.finish().unwrap();

        display.gl_window().window().request_redraw();
    }
}