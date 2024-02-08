use nalgebra::{Matrix4, Perspective3, Point3, SMatrix, Vector3};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

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

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(keycode),
                    state,
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    KeyCode::KeyW | KeyCode::ArrowUp => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA | KeyCode::ArrowLeft => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS | KeyCode::ArrowDown => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD | KeyCode::ArrowRight => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false
                }
            }
            _ => false
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(&camera.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }

        println!("{}", camera.eye);
    }
}
