use super::*;
use glium_shapes::sphere::SphereBuilder;
use serde_json::json;

#[derive(Clone, Copy)]
pub struct EuclideanSphereMirror<const D: usize> {
    center: SVector<f32, D>,
    radius: f32,
}

impl<const D: usize> Mirror<D> for EuclideanSphereMirror<D> {
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<Tangent<D>>) {
        // TODO: more calculations can be offset to the inside of the if block
        // mental note: Cauchy-Schwarz

        let d = &ray.direction;
        let a = d.norm_squared();

        let v0 = &self.center;
        let v = ray.origin - v0;

        let b = v.dot(d);

        let r = &self.radius;
        let s = v.norm_squared();
        let c = s - r * r;

        let delta = b * b - a * c;

        if delta > f32::EPSILON {
            let root_delta = delta.sqrt();
            let neg_b = -b;

            for t in [(neg_b - root_delta) / a, (neg_b + root_delta) / a] {
                let origin = ray.at(t);
                let normal = Unit::new_normalize(origin - v0);
                list.push(Tangent::Normal { origin, normal });
            }
        }
    }

    fn get_json_type() -> String {
        "sphere".into()
    }

    fn get_json_type_inner(&self) -> String {
        "sphere".into()
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>> {
        /* example json
        {
            "center": [1., 2., 3.],
            "radius": 4.,
        }
         */

        let center = json
            .get("center")
            .and_then(serde_json::Value::as_array)
            .map(Vec::as_slice)
            .and_then(util::json_array_to_vector)
            .ok_or("Failed to parse center")?;

        let radius = json
            .get("radius")
            .and_then(serde_json::Value::as_f64)
            .ok_or("Failed to parse radius")? as f32;

        Ok(Self { center, radius })
    }

    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let center: Vec<f32> = self.center.iter().copied().collect();
        let json = json!({
            "center": center,
            "radius": self.radius,
        });
        Ok(json)
    }

    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        let coords = match D {
            1 => [self.center[0], 0.0, 0.0],
            2 => [self.center[0], self.center[1], 0.0],
            3 => [self.center[0], self.center[1], self.center[2]],
            _ => unreachable!(),
        };

        let scale = match D {
            1 => unimplemented!(),
            2 => [self.radius, self.radius, 0.],
            3 => [self.radius, self.radius, self.radius],
            _ => unreachable!(),
        };

        // The default sphere from the SphereBuilder is a unit-sphere (radius of 1) with its center of mass located at the origin.
        // So we just have to scale it with the sphere radius on each axis and translate it.
        let sphere = SphereBuilder::new()
            .scale(scale[0], scale[1], scale[2])
            .translate(coords[0], coords[1], coords[2])
            .with_divisions(60, 60)
            .build(display)
            .unwrap();

        vec![Box::new(sphere)]
    }

    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized,
    {
        const MAX_RADIUS: f32 = 8.0;
        Self {
            center: util::random_vector(rng, 24.0),
            radius: rng.gen::<f32>() * MAX_RADIUS.abs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_sphere() {
        let mirror = EuclideanSphereMirror::<3>::from_json(&json!({
            "center": [0., 0., 0.],
            "radius": 1.,
        }))
        .expect("json error");

        let mut ray = Ray {
            origin: [-2., 0., 0.].into(),
            direction: Unit::new_normalize([1., 0., 0.].into()),
        };

        let mut intersections = vec![];
        mirror.append_intersecting_points(&ray, &mut intersections);

        assert_eq!(intersections.len(), 2);

        let tangent = &intersections[0];
        let d = tangent.try_intersection_distance(&ray);

        if let Some(t) = d {
            assert!((t - 1.).abs() < f32::EPSILON);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_direction(tangent);

        assert!((ray.origin - SVector::from([-1., 0., 0.])).norm().abs() < f32::EPSILON);
        assert!(
            (ray.direction.into_inner() - SVector::from([-1., 0., 0.]))
                .norm()
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_no_intersection() {
        let mirror = EuclideanSphereMirror::<3>::from_json(&json!({
            "center": [0., 0., 0.],
            "radius": 1.,
        }))
        .expect("json error");

        let ray = Ray {
            origin: [-2., 0., 0.].into(),
            direction: Unit::new_normalize([0., 1., 0.].into()),
        };

        let mut intersections = vec![];
        mirror.append_intersecting_points(&ray, &mut intersections);

        assert_eq!(intersections.len(), 0);
    }

    #[test]
    fn test_angled_ray() {
        let mirror = EuclideanSphereMirror::<3>::from_json(&json!({
            "center": [0., 0., 0.],
            "radius": 1.,
        }))
        .expect("json error");

        let mut ray = Ray {
            origin: [-2., -1., 0.].into(),
            direction: Unit::new_normalize([1., 1., 0.].into()),
        };

        let mut intersections = vec![];
        mirror.append_intersecting_points(&ray, &mut intersections);

        assert_eq!(intersections.len(), 2);

        let tangent = &intersections[0];
        let d = tangent.try_intersection_distance(&ray);

        if let Some(t) = d {
            assert!((t - 1.4142137).abs() < f32::EPSILON);
            ray.advance(t);
        } else {
            panic!("there must be distance");
        }

        ray.reflect_direction(tangent);

        assert!((ray.origin - SVector::from([-1., 0., 0.])).norm().abs() < f32::EPSILON);
        assert!(
            (ray.direction.into_inner() - SVector::from([-0.70710665, 0.70710695, 0.]))
                .norm()
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn test_json() {
        let mirror = EuclideanSphereMirror::<3>::from_json(&json!({
            "center": [0., 0., 0.],
            "radius": 1.,
        }))
        .expect("json error");

        let json = mirror.to_json().expect("json error");

        let mirror2 = EuclideanSphereMirror::<3>::from_json(&json).expect("json error");

        assert_eq!(mirror.center, mirror2.center);
        assert_eq!(mirror.radius, mirror2.radius);
    }
}
