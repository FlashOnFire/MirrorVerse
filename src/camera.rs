use nalgebra::{Matrix4, MatrixView3, Perspective3, Point3, SMatrix, Transform3, Vector3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: SMatrix<f32, 4, 4> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub(crate) eye: Point3<f32>,
    pub(crate) target: Point3<f32>,
    pub(crate) up: Vector3<f32>,
    pub(crate) aspect: f32,
    pub(crate) fovy: f32,
    pub(crate) znear: f32,
    pub(crate) zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> SMatrix<f32, 4, 4> {
        let proj = Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar);
        let view = Matrix4::look_at_rh(&self.eye, &self.target, &self.up);

        return OPENGL_TO_WGPU_MATRIX * proj.as_matrix() * view;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    pub(crate) fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

