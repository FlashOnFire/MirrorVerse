use nalgebra::{Point, SVector, Unit};

use crate::DIM;

pub struct Ray {
    pub origin: Point<f32, DIM>,
    pub direction: Unit<SVector<f32, DIM>>,
}
