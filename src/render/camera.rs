use core::{f32::consts::FRAC_PI_2, time::Duration};

use cgmath::{Matrix4, Rad, Vector3};
use glium::glutin::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseScrollDelta, VirtualKeyCode},
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: nalgebra::Matrix4<f32> = nalgebra::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform {
    view_pos: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn new() -> Self {
        Self {
            view_pos: [0.0; 4],
            view_proj: nalgebra::Matrix4::identity().into(),
        }
    }

    pub(crate) fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_pos = camera.position.to_homogeneous().into();

        let a = camera.calc_matrix();

        let cam = nalgebra::Matrix4::new(
            a.x.x, a.y.x, a.z.x, a.w.x, a.x.y, a.y.y, a.z.y, a.w.y, a.x.z, a.y.z, a.z.z, a.w.z,
            a.x.w, a.y.w, a.z.w, a.w.w,
        );

        self.view_proj = (projection.calc_matrix() * cam).into()
    }
}

pub struct Camera {
    pub position: nalgebra::Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<V: Into<nalgebra::Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        let y = nalgebra::Vector3::y();
        let target =
            nalgebra::Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize();

        let ret = Matrix4::look_to_rh(
            cgmath::Point3::new(self.position.x, self.position.y, self.position.z),
            Vector3::new(target.x, target.y, target.z),
            Vector3::new(y.x, y.y, y.z),
        );

        let a =
            nalgebra::Isometry3::look_at_rh(&self.position, &target.into(), &y).to_homogeneous();

        let ret2 = Matrix4::new(
            a.m11, a.m21, a.m31, a.m41, a.m12, a.m22, a.m32, a.m42, a.m13, a.m23, a.m33, a.m43,
            a.m14, a.m24, a.m34, a.m44,
        );
        ret
    }
}

#[derive(Debug)]
pub struct Projection {
    aspect: f32,
    fov_y: Rad<f32>,
    z_near: f32,
    z_far: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(
        width: u32,
        height: u32,
        fov_y: F,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fov_y: fov_y.into(),
            z_near,
            z_far,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> nalgebra::Matrix4<f32> {
        let b = nalgebra::Perspective3::new(self.fov_y.0, self.aspect, self.z_near, self.z_far);
        OPENGL_TO_WGPU_MATRIX * b.as_matrix()
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backwards: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backwards: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };

        match key {
            VirtualKeyCode::Z => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S => {
                self.amount_backwards = amount;
                true
            }
            VirtualKeyCode::Q => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = nalgebra::Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = nalgebra::Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward =
            nalgebra::Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();

        camera.position +=
            scrollward * (self.amount_forward - self.amount_backwards) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}
