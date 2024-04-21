use core::{mem, ops::Add};

use super::*;

/// A parallelotope-shaped reflective (hyper)plane
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlaneMirror<const D: usize = DEFAULT_DIM> {
    /// The plane this mirror belongs to.
    plane: Plane<D>,
    /// maximum magnitudes (mu_i_max) of the scalars in the linear combination of the
    /// basis vectors of the associated hyperplane.
    ///
    /// Formally, for all vectors `v = sum mu_i * v_i` of
    /// the hyperplane, `v` is in this plane mirror iff for all `i`, `|mu_i| <= |mu_i_max|`
    ///
    /// Note: the first value of this array is irrelevant
    bounds: [f32; D],
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
            primitives: index::PrimitiveType::TriangleStrip,
        }
    }
}

impl<const D: usize> PlaneMirror<D> {
    pub fn vector_bounds(&self) -> &[f32] {
        &self.bounds[1..]
    }

    pub fn vertices(&self) -> impl Iterator<Item = SVector<f32, D>> + '_ {
        const SHIFT: usize = mem::size_of::<f32>() * 8 - 1;

        let basis = self.plane.basis();
        let v_0 = *self.plane.v_0();
        let bounds = self.vector_bounds();

        (0..1 << (D - 1)).map(move |i| {
            bounds
                .iter()
                .zip(basis)
                .enumerate()
                // returns `mu * v` with the sign flipped if the `j`th bit in `i` is 1
                .map(|(j, (mu, v))| f32::from_bits(i >> j << SHIFT ^ mu.to_bits()) * v)
                .fold(v_0, Add::add)
        })
    }
}

use gl::index;

impl<const D: usize> Mirror<D> for PlaneMirror<D>
where
    render::Vertex<D>: gl::Vertex,
{
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<Tangent<D>>) {
        if self
            .plane
            .intersection_coordinates(ray)
            .filter(|v| {
                v.iter()
                    .skip(1)
                    .zip(self.vector_bounds())
                    .all(|(mu, mu_max)| mu.abs() <= mu_max.abs())
            })
            .is_some()
        {
            list.push(Tangent::Plane(self.plane));
        }
    }

    fn get_json_type() -> String {
        "plane".into()
    }

    fn get_json_type_inner(&self) -> String {
        "plane".into()
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        /*
        example json:
        {
            "center": [9., 8., 7., ...], (N elements)
            "basis": [ (N - 1 elements)
                [9., 8., 7., ...], (N elements)
                [6., 5., 4., ...],
            ],
            "bounds": [6., 9., ...] (N - 1 elements)
            "darkness": 0.5,
        }
        */

        let mut vectors = [SVector::zeros(); D];

        let (v_0, basis) = vectors.split_first_mut().unwrap();

        *v_0 = json
            .get("center")
            .and_then(serde_json::Value::as_array)
            .map(Vec::as_slice)
            .and_then(util::json_array_to_vector)
            .ok_or("Failed to parse center")?;

        let basis_json = json
            .get("basis")
            .and_then(serde_json::Value::as_array)
            .filter(|l| l.len() == D - 1)
            .ok_or("Failed to parse basis")?;

        for (value, vector) in basis_json.iter().zip(basis) {
            *vector = value
                .as_array()
                .map(Vec::as_slice)
                .and_then(util::json_array_to_vector)
                .ok_or("Failed to parse basis vector")?;
        }

        let bounds_json = json
            .get("bounds")
            .and_then(serde_json::Value::as_array)
            .filter(|l| l.len() == D - 1)
            .ok_or("Failed to parse bounds")?;

        let mut bounds = [0.; D];
        for (i, o) in bounds[1..].iter_mut().zip(bounds_json.iter()) {
            *i = o.as_f64().ok_or("Failed to parse bound")? as f32;
        }

        let plane = Plane::new(vectors).ok_or("Failed to create plane")?;

        Ok(Self { plane, bounds })
    }

    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        todo!()
    }

    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        let vertices: Vec<_> = self.vertices().map(render::Vertex::from).collect();

        vec![Box::new(PlaneRenderData {
            vertices: gl::VertexBuffer::new(display, vertices.as_slice()).unwrap(),
        })]
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
