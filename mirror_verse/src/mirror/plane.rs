use core::{array, mem, ops::Add};

use super::*;

/// A parallelotope-shaped reflective (hyper)plane
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaneMirror<const D: usize> {
    /// The plane this mirror belongs to.
    plane: Plane<D>,
}

impl<const D: usize> From<Plane<D>> for PlaneMirror<D> {
    fn from(plane: Plane<D>) -> Self {
        Self { plane }
    }
}

struct PlaneRenderData<const D: usize> {
    vertices: gl::VertexBuffer<render::Vertex<D>>,
}

impl<const D: usize> render::RenderData for PlaneRenderData<D> {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        gl::index::IndicesSource::NoIndices {
            primitives: match D {
                0 => unreachable!("dimension must not be zero"),
                1 | 2 => PrimitiveType::LinesList,
                _ => PrimitiveType::TriangleStrip,
            },
        }
    }
}

impl<const D: usize> PlaneMirror<D> {
    pub fn vertices(&self) -> impl Iterator<Item = SVector<f32, D>> + '_ {
        const SHIFT: usize = mem::size_of::<f32>() * 8 - 1;

        let basis = self.plane.basis();
        let v_0 = *self.plane.v_0();

        (0..1 << (D - 1)).map(move |i| {
            basis
                .iter()
                .enumerate()
                // returns `v` with the sign flipped if the `j`th bit in `i` is 1
                .map(|(j, v)| f32::from_bits(i >> j << SHIFT ^ 1f32.to_bits()) * v)
                .fold(v_0, Add::add)
        })
    }
}

use glium::index::PrimitiveType;

impl<const D: usize> Mirror<D> for PlaneMirror<D> {
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<Tangent<D>>) {
        if self
            .plane
            .intersection_coordinates(ray)
            .filter(|v| v.iter().skip(1).all(|mu| mu.abs() < 1.0))
            .is_some()
        {
            list.push(Tangent::Plane(self.plane));
        }
    }
}

impl<const D: usize> JsonType for PlaneMirror<D> {
    fn json_type() -> String {
        "plane".into()
    }
}

impl<const D: usize> JsonDes for PlaneMirror<D> {
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        // see mirror::Plane::from_json for more info on this mirror's json format
        Plane::from_json(json).map(Self::from)
    }
}

impl<const D: usize> JsonSer for PlaneMirror<D> {
    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        self.plane.to_json()
    }
}

impl render::OpenGLRenderable for PlaneMirror<2> {
    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        let vertices: Vec<_> = self.vertices().map(render::Vertex::<2>::from).collect();

        vec![Box::new(PlaneRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        })]
    }
}

impl render::OpenGLRenderable for PlaneMirror<3> {
    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        let vertices: Vec<_> = self.vertices().map(render::Vertex::<3>::from).collect();

        vec![Box::new(PlaneRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        })]
    }
}

impl<const D: usize> Random for PlaneMirror<D> {
    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized,
    {
        // bahaha t'y etais presque
        loop {
            if let Some(plane) = Plane::new(array::from_fn(|_| util::random_vector(rng, 24.0))) {
                break plane;
            }
        }
        .into()
    }
}

#[cfg(test)]
mod tests {

    use core::f32::consts::{FRAC_1_SQRT_2, SQRT_2};

    use super::*;
    use serde_json::json;

    #[test]
    fn test_left_basic_2d() {
        let mirror = PlaneMirror::<2>::from_json(&json!({
            "center": [0., 0.],
            "basis": [
                [0., 1.],
            ],
            "bounds": [1.],
        }))
        .expect("json monke");

        let mut ray = Ray {
            origin: [-1., 0.].into(),
            direction: Unit::new_normalize([1., 0.].into()),
        };

        let mut intersections = vec![];
        mirror.append_intersecting_points(&ray, &mut intersections);

        let [tangent] = intersections.as_slice() else {
            panic!("there must be an intersection");
        };

        let d = tangent.try_intersection_distance(&ray);

        if let Some(t) = d {
            assert!((t - 1.).abs() < f32::EPSILON);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_direction(tangent);

        assert!((ray.origin - SVector::from([0., 0.])).norm().abs() < f32::EPSILON);
        assert!(
            (ray.direction.into_inner() - SVector::from([-1., 0.]))
                .norm()
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_right_basic_2d() {
        let mirror = PlaneMirror::<2>::from_json(&json!({
            "center": [0., 0.],
            "basis": [
                [0., 1.],
            ],
            "bounds": [1.],
        }))
        .expect("json monke");

        let mut ray = Ray {
            origin: [1., 0.].into(),
            direction: Unit::new_normalize([-1., 0.].into()),
        };

        let mut intersections = vec![];

        mirror.append_intersecting_points(&ray, &mut intersections);

        let [tangent] = intersections.as_slice() else {
            panic!("there must be an intersection");
        };

        let d = tangent.try_intersection_distance(&ray);

        if let Some(t) = d {
            assert!((t - 1.).abs() < f32::EPSILON);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_direction(&tangent);

        assert!((ray.origin - SVector::from([0., 0.])).norm().abs() < f32::EPSILON);
        assert!(
            (ray.direction.into_inner() - SVector::from([1., 0.]))
                .norm()
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_diagonal_2d() {
        let mirror = PlaneMirror::<2>::from_json(&json!({
            "center": [0., 0.],
            "basis": [
                [FRAC_1_SQRT_2, FRAC_1_SQRT_2],
            ],
            "bounds": [1.],
        }))
        .expect("json monke");

        let mut ray = Ray {
            origin: [-1., 1.].into(),
            direction: Unit::new_normalize([1., -1.].into()),
        };

        let mut intersections = vec![];
        mirror.append_intersecting_points(&ray, &mut intersections);

        let [tangent] = intersections.as_slice() else {
            panic!("there must be an intersection");
        };

        let d = tangent.try_intersection_distance(&ray);

        if let Some(t) = d {
            assert!((t - SQRT_2).abs() < f32::EPSILON * 2.);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_direction(&tangent);

        assert!((ray.origin - SVector::from([0., 0.])).norm().abs() < f32::EPSILON);
        assert!(
            (ray.direction.into_inner() - SVector::from([-FRAC_1_SQRT_2, FRAC_1_SQRT_2]))
                .norm()
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_multiple_intersections_2d() {
        let m1 = PlaneMirror::<2>::from_json(&json!({
            "center": [10., 0.],
            "basis": [
                [0., 1.],
            ],
            "bounds": [1.],
        }))
        .expect("json monke");

        let m2 = PlaneMirror::<2>::from_json(&json!({
            "center": [-1., 0.],
            "basis": [
                [0., 1.],
            ],
            "bounds": [1.],
        }))
        .expect("json monke");

        let mut ray = Ray {
            origin: [0., 0.5].into(),
            direction: Unit::new_normalize([1., 0.].into()),
        };

        let mut pts = vec![];
        m1.append_intersecting_points(&ray, &mut pts);
        m2.append_intersecting_points(&ray, &mut pts);

        let [t1, t2] = pts.as_slice() else {
            panic!("there must be an intersection");
        };

        let d1 = t1.try_intersection_distance(&ray);
        let d2 = t2.try_intersection_distance(&ray);

        if let Some(t) = d1 {
            assert!((t - 10.).abs() < f32::EPSILON * 2.);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        if let Some(t) = d2 {
            assert!((t - -1.).abs() < f32::EPSILON * 2.);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_direction(&t1);

        assert!((ray.origin - SVector::from([10., 0.5])).norm().abs() < f32::EPSILON);
        assert!(
            (ray.direction.into_inner() - SVector::from([-1., 0.]))
                .norm()
                .abs()
                < f32::EPSILON
        );
    }
}
