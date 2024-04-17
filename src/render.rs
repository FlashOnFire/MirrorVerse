use super::*;
use gl::{
    index::{NoIndices, PrimitiveType},
    Blend, Surface, VertexBuffer,
};

pub mod camera;

#[derive(Copy, Clone, Debug)]
pub struct Vertex<const N: usize> {
    position: [f32; N],
}

type Vertex3D = Vertex<3>;
glium::implement_vertex!(Vertex3D, position);

type Vertex2D = Vertex<2>;
glium::implement_vertex!(Vertex2D, position);

impl<const N: usize> From<nalgebra::SVector<f32, N>> for Vertex<N> {
    fn from(v: nalgebra::SVector<f32, N>) -> Self {
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

pub struct DrawableSimulation<T: Copy> {
    ray_path_vertices: Vec<VertexBuffer<T>>,
    mirrors: Vec<(NoIndices, VertexBuffer<T>)>,
}

impl<T: gl::Vertex> DrawableSimulation<T> {
    pub fn new(
        ray_path_vertices: Vec<VertexBuffer<T>>,
        mirrors: Vec<(NoIndices, VertexBuffer<T>)>,
    ) -> Self {
        Self {
            ray_path_vertices,
            mirrors,
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
                test: gl::draw_parameters::DepthTest::Overwrite,
                write: false,
                ..Default::default()
            },
            line_width: Some(3.),
            blend: Blend::alpha_blending(),
            ..Default::default()
        };

        for buffer in self.ray_path_vertices.as_slice() {
            target
                .draw(
                    buffer,
                    NoIndices(PrimitiveType::LineStrip),
                    program3d,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: RAY_COLOR,
                    },
                    &params,
                )
                .unwrap();
        }

        for (indices, buffer) in &self.mirrors {
            target
                .draw(
                    buffer,
                    indices,
                    program3d,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: MIRROR_COLOR,
                    },
                    &params,
                )
                .unwrap();
        }

        target.finish().unwrap();

        display.gl_window().window().request_redraw();
    }
}
