use super::*;
use gl::{
    index::{NoIndices, PrimitiveType},
    Blend, Surface, VertexBuffer,
};
use glium_shapes::sphere::Sphere;

pub mod camera;

#[derive(Copy, Clone, Debug)]
pub struct Vertex<const N: usize> {
    pub position: [f32; N],
}

type Vertex3D = Vertex<3>;
glium::implement_vertex!(Vertex3D, position);

type Vertex2D = Vertex<2>;
glium::implement_vertex!(Vertex2D, position);

impl<const N: usize, const D: usize> From<nalgebra::SVector<f32, D>> for Vertex<N> {
    fn from(v: nalgebra::SVector<f32, D>) -> Self {
        assert!(D <= N);
        let mut position = [0.; N];
        for i in 0..D {
            position[i] = v[i];
        }

        Self { position }
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

pub struct DrawableSimulation<const D: usize> {
    origins: Vec<Sphere>,
    ray_path_vertices: Vec<VertexBuffer<Vertex<D>>>,
    mirrors: Vec<Box<dyn render::RenderData>>,
}

impl<const D: usize> DrawableSimulation<D>
where
    Vertex<D>: gl::Vertex,
{
    pub fn new(
        origins: Vec<Sphere>,
        ray_path_vertices: Vec<VertexBuffer<Vertex<D>>>,
        mirrors: Vec<Box<dyn RenderData>>,
    ) -> Self {
        Self {
            origins,
            ray_path_vertices,
            mirrors,
        }
    }

    pub fn render(
        &self,
        display: &gl::backend::glutin::Display,
        program3d: &gl::Program,
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
            line_width: Some(2.),
            blend: Blend::alpha_blending(),
            ..Default::default()
        };

        for sphere in &self.origins {
            target
                .draw(
                    sphere,
                    sphere,
                    program3d,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: ORIGIN_COLOR,
                    },
                    &params,
                )
                .unwrap();
        }

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

        for render_data in &self.mirrors {
            target
                .draw(
                    render_data.vertices(),
                    render_data.indices(),
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

pub trait RenderData {
    fn vertices(&self) -> gl::vertex::VerticesSource;
    fn indices(&self) -> gl::index::IndicesSource;
}

// oh yeah baby
impl<T> RenderData for T
where
    for<'a> &'a T: Into<gl::vertex::VerticesSource<'a>>,
    for<'a> &'a T: Into<gl::index::IndicesSource<'a>>,
{
    fn vertices(&self) -> gl::vertex::VerticesSource {
        self.into()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        self.into()
    }
}
