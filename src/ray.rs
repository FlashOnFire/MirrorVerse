use nalgebra::Point;

use crate::DIM;

pub struct Ray {
    pub origin: Point<f32, DIM>,
    pub direction: Point<f32, DIM>,
}
