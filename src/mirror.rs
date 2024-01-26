use nalgebra::Point;

use crate::{ray::Ray, DIM};

trait Mirror {
    fn reflect(&self, ray: Ray) -> Vec<Ray>;
}

struct CompositeMirror {
    mirrors: Vec<Box<dyn Mirror>>,
}

impl Mirror for CompositeMirror {
    fn reflect(&self, ray: Ray) -> Vec<Ray> {
        // use the other mirror to reflect the ray
        vec![]
    }
}

#[derive(Clone, Copy)]
struct PlaneMirror {
    points: [Point<f32, DIM>; DIM],
}

impl Mirror for PlaneMirror {
    fn reflect(&self, ray: Ray) -> Vec<Ray> {
        vec![Ray { ..ray }]
    }
}

#[derive(Clone, Copy)]
struct SphereMirror {
    center: Point<f32, DIM>,
    radius: f32,
}

impl Mirror for SphereMirror {
    fn reflect(&self, ray: Ray) -> Vec<Ray> {
        vec![Ray { ..ray }]
    }
}
