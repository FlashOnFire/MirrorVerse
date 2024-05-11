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

pub type Vertex2D = Vertex<2>;
glium::implement_vertex!(Vertex2D, position);

pub type Vertex3D = Vertex<3>;
glium::implement_vertex!(Vertex3D, position);

pub type Vertex4D = Vertex<4>;
glium::implement_vertex!(Vertex4D, position);

impl<const N: usize, const D: usize> From<nalgebra::SVector<f32, D>> for Vertex<N> {
    fn from(v: nalgebra::SVector<f32, D>) -> Self {
        Self {
            position: array::from_fn(|i| if i < D { v[i] } else { 0.0 }),
        }
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

pub struct RayRenderData<const D: usize> {
    // TODO: find another way to draw this, that preserves
    // it's no matter how far away you are from it
    pub origin: Sphere,
    pub non_loop_path: VertexBuffer<Vertex<D>>,
    pub loop_path: VertexBuffer<Vertex<D>>,
}

pub(crate) struct DrawableSimulation<const D: usize> {
    ray_render_data: Vec<RayRenderData<D>>,
    mirror_render_data: Vec<Box<dyn render::RenderData>>,
}

impl<const D: usize> DrawableSimulation<D>
where
    Vertex<D>: gl::Vertex,
{
    pub(crate) fn new(
        ray_render_data: Vec<RayRenderData<D>>,
        mirror_render_data: Vec<Box<dyn RenderData>>,
    ) -> Self {
        Self {
            ray_render_data,
            mirror_render_data,
        }
    }
}

impl DrawableSimulation<3> {
    pub(crate) fn render_3d(
        &self,
        display: &gl::backend::glutin::Display,
        program3d: &gl::Program,
        camera: &Camera,
        projection: &Projection,
    ) {
        const ORIGIN_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const RAY_NON_LOOP_COL: [f32; 4] = [0.7, 0.3, 0.1, 1.0];
        const RAY_LOOP_COL: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const MIRROR_COLOR: [f32; 4] = [0.3, 0.3, 0.9, 0.4];

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

        for ray in &self.ray_render_data {
            let o = &ray.origin;
            target
                .draw(
                    o,
                    o,
                    program3d,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: ORIGIN_COLOR,
                    },
                    &params,
                )
                .unwrap();

            target
                .draw(
                    &ray.non_loop_path,
                    NoIndices(PrimitiveType::LineStrip),
                    program3d,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: RAY_NON_LOOP_COL,
                    },
                    &params,
                )
                .unwrap();

            target
                .draw(
                    &ray.loop_path,
                    NoIndices(PrimitiveType::LineStrip),
                    program3d,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: RAY_LOOP_COL,
                    },
                    &params,
                )
                .unwrap();
        }

        for render_data in &self.mirror_render_data {
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
