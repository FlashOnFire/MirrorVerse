use core::ops::Deref;

use super::*;
use gl::{
    glutin::dpi::PhysicalSize,
    index::{NoIndices, PrimitiveType},
    Blend, Surface, VertexBuffer,
};

pub(crate) mod camera;

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

impl<const D: usize> From<nalgebra::SVector<f32, D>> for Vertex<D> {
    fn from(v: nalgebra::SVector<f32, D>) -> Self {
        Self {
            position: v.into(),
        }
    }
}

pub(crate) const FRAGMENT_SHADER_SRC: &str = r#"
    #version 140

    uniform vec4 color_vec;

    out vec4 color;

    void main() {
        color = color_vec;
    }
"#;

pub(crate) const VERTEX_SHADER_SRC_3D: &str = r#"
    #version 140

    in vec3 position;
    uniform mat4 perspective;
    uniform mat4 view;

    void main() {
        gl_Position = perspective * view * vec4(position, 1.0);
    }
"#;

pub(crate) const VERTEX_SHADER_SRC_2D: &str = r#"
    #version 140

    in vec2 position;
    uniform mat4 perspective;
    uniform mat4 view;

    void main() {
        gl_Position = perspective * view * vec4(position, 0.0, 1.0);
    }
"#;

pub(crate) struct RayRenderData<const D: usize> {
    // TODO: find another way to draw this, that preserves
    // it's size no matter how far away you are from it
    pub origin: Box<dyn RenderData>,
    pub non_loop_path: VertexBuffer<Vertex<D>>,
    pub loop_path: VertexBuffer<Vertex<D>>,
}

pub(crate) struct DrawableSimulation<const D: usize> {
    ray_render_data: Vec<RayRenderData<D>>,
    mirror_render_data: Vec<Box<dyn render::RenderData>>,
    program: gl::Program,
}

impl<const D: usize> DrawableSimulation<D>
where
    Vertex<D>: gl::Vertex,
{
    pub(crate) fn new(
        ray_render_data: Vec<RayRenderData<D>>,
        mirror_render_data: Vec<Box<dyn RenderData>>,
        program: gl::Program,
    ) -> Self {
        Self {
            ray_render_data,
            mirror_render_data,
            program,
        }
    }
}

impl<const D: usize> DrawableSimulation<D>
where
    Vertex<D>: gl::Vertex,
{
    pub(crate) fn run(self, display: gl::Display, events_loop: glutin::event_loop::EventLoop<()>) {
        const DEFAULT_CAMERA_POS: cg::Point3<f32> = cg::Point3::new(0., 0., 5.);
        const DEFAULT_CAMERA_YAW: cg::Deg<f32> = cg::Deg(-90.);
        const DEFAULT_CAMERA_PITCH: cg::Deg<f32> = cg::Deg(0.);

        let mut camera = Camera::new(DEFAULT_CAMERA_POS, DEFAULT_CAMERA_YAW, DEFAULT_CAMERA_PITCH);

        const DEFAULT_PROJECCTION_POV: cg::Deg<f32> = cg::Deg(85.);
        const NEAR_PLANE: f32 = 0.0001;
        const FAR_PLANE: f32 = 10000.;

        let PhysicalSize { width, height } = display.gl_window().window().inner_size();

        let mut projection = Projection::new(
            width,
            height,
            DEFAULT_PROJECCTION_POV,
            NEAR_PLANE,
            FAR_PLANE,
        );

        const SPEED: f32 = 5.;
        const MOUSE_SENSITIVITY: f32 = 4.0;

        let mut camera_controller = CameraController::new(SPEED, MOUSE_SENSITIVITY);

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
                self.render_3d(&display, &camera, &projection);
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

    pub(crate) fn render_3d(
        &self,
        display: &gl::Display,
        camera: &Camera,
        projection: &Projection,
    ) {
        const ORIGIN_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const RAY_NON_LOOP_COL: [f32; 4] = [0.7, 0.3, 0.1, 1.0];
        const RAY_LOOP_COL: [f32; 4] = [1.0, 0.0, 1.0, 1.0];
        let mirror_color = if D >= 3 {
            [0.3f32, 0.3, 0.9, 0.4]
        } else {
            [0.15, 0.15, 0.5, 1.0]
        };

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
            line_width: Some(2.0),
            blend: Blend::alpha_blending(),
            ..Default::default()
        };

        for ray in &self.ray_render_data {

            target
                .draw(
                    &ray.non_loop_path,
                    NoIndices(PrimitiveType::LineStrip),
                    &self.program,
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
                    &self.program,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: RAY_LOOP_COL,
                    },
                    &params,
                )
                .unwrap();
                
            let o = &ray.origin;
            target
                .draw(
                    o.vertices(),
                    o.indices(),
                    &self.program,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: ORIGIN_COLOR,
                    },
                    &params,
                )
                .unwrap();
        }

        for render_data in self.mirror_render_data.iter().map(Box::as_ref) {
            target
                .draw(
                    render_data.vertices(),
                    render_data.indices(),
                    &self.program,
                    &gl::uniform! {
                        perspective: perspective,
                        view: view,
                        color_vec: mirror_color,
                    },
                    &params,
                )
                .unwrap();
        }

        target.finish().unwrap();

        display.gl_window().window().request_redraw();
    }
}

// Again, could have been an associated constant, but `#[feature(generic_const_exprs)]` screws us over
pub trait RenderData {
    fn vertices(&self) -> gl::vertex::VerticesSource;
    fn indices(&self) -> gl::index::IndicesSource;
}

// glium_shapes 3d convenience blanket impl
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

pub trait OpenGLRenderable {
    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>>;
}

impl<T: OpenGLRenderable> OpenGLRenderable for [T] {
    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        self.iter().flat_map(|a| a.render_data(display)).collect()
    }
}

impl<T: Deref> OpenGLRenderable for T
where
    T::Target: OpenGLRenderable,
{
    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        self.deref().render_data(display)
    }
}

pub(crate) struct Circle {
    pub vertices: gl::VertexBuffer<render::Vertex2D>,
}

impl Circle {
    pub fn new(center: [f32 ; 2], radius: f32, display: &gl::Display) -> Self {
        const NUM_POINTS: usize = 360;

        use core::f32::consts::TAU;

        let c = SVector::from(center);



        let points: Vec<Vertex2D> = (0..NUM_POINTS)
            .map(|i| {
                let pos: [f32 ; 2] = (i as f32 / NUM_POINTS as f32 * TAU).sin_cos().into();
                (SVector::from(pos) * radius + c).into()
            })
            .collect();

        let vertices = gl::VertexBuffer::immutable(display, points.as_slice()).unwrap();

        Self { vertices }
    }
}

impl render::RenderData for Circle {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        gl::index::IndicesSource::NoIndices {
            primitives: gl::index::PrimitiveType::LineLoop
        }
    }
}

pub(crate) struct FilledCircle(Circle);

impl From<Circle> for FilledCircle {
    fn from(value: Circle) -> Self {
        Self(value)
    }
}

impl render::RenderData for FilledCircle {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        self.0.vertices()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        gl::index::IndicesSource::NoIndices {
            primitives: gl::index::PrimitiveType::TriangleFan
        }
    }
}