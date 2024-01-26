use crate::DIM;
use nalgebra::{Point, SVector, Unit};

pub struct Ray {
    pub origin: Point<f32, DIM>,
    pub direction: Unit<SVector<f32, DIM>>,
}
