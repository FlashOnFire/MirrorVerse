pub mod camera;

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

        uniform vec3 color_vec;

        out vec4 color;

        void main() {
            color = vec4(color_vec.xyz, 1.0);
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
