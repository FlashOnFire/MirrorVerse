use super::*;
use glium_shapes::sphere::SphereBuilder;
use rand::random;
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
        // TODO: Maybe support circular shapes in 2 dimensions ?
        assert_eq!(D, 3);
        let coords = self.center.as_slice().get(0..3).unwrap();

        // The default sphere from the SphereBuilder is a unit-sphere (radius of 1) with its center of mass located at the origin.
        // So we just have to scale it with the sphere radius on each axis and translate it.
        let sphere = SphereBuilder::new()
            .scale(self.radius, self.radius, self.radius)
            .translate(coords[0], coords[1], coords[2])
            .with_divisions(
                (self.radius * 200.0) as usize,
                (self.radius * 200.0) as usize,
            )
            .build(display)
            .unwrap();

        vec![Box::new(sphere)]
    }
    fn random() -> Self
    where
        Self: Sized,
    {
        let center = SVector::<f32, D>::from_fn(|_, _| random::<f32>());
        let radius = random::<f32>();
        Self { center, radius }
    }
}

#[cfg(test)]
mod tests {}
