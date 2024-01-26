use nalgebra::{Point, SVector, Unit};

use crate::DIM;

pub struct Ray {
    pub origin: Unit<SVector<f32, DIM>>,
    pub direction: Point<f32, DIM>,
}
