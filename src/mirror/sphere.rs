use super::*;

// TODO: explore the possbility of generalising the definition
// of a sphere to other p-norms (for strictly positive p)

#[derive(Clone, Copy)]
pub struct SphereMirror<const D: usize = DEFAULT_DIM> {
    center: Point<f32, D>,
    radius: f32,
}

impl<const D: usize> Mirror<D> for SphereMirror<D> {
    fn reflect(&self, ray: &Ray<D>) -> Vec<(f32, Plane<D>)> {
        // TODO: implement spherical mirror reflection
        vec![]
    }

    fn get_type(&self) -> &str {
        "sphere"
    }

    fn from_json(json: &serde_json::Value) -> Option<Self>
    where
        Self: Sized,
    {
        /* example json
        {
            "center": [1.0, 2.0, 3.0],
            "radius": 4.0
        }
         */

        // TODO: optimize out the allocations
        // TODO: return a Result with clearer errors

        let center: [_; D] = json
            .get("center")?
            .as_array()?
            .iter()
            .filter_map(serde_json::Value::as_f64)
            .map(|val| val as f32)
            .collect::<Vec<_>>()
            .try_into()
            .ok()?;

        let radius = json.get("radius")?.as_f64()? as f32;

        Some(Self {
            center: Point::from_slice(&center),
            radius,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_with_0(mut vec: Vec<f32>) -> Vec<f32> {
        vec.resize(DEFAULT_DIM, 0.0);
        vec
    }

    #[test]
    fn test_sphere_mirror_from_json() {
        println!("oucou");
        let json = serde_json::json!({
            "center": complete_with_0(vec![1.0, 2.0]),
            "radius": 4.0
        });

        let mirror = SphereMirror::from_json(&json).expect("json deserialisation failed");

        assert_eq!(
            mirror.center,
            Point::<f32, DEFAULT_DIM>::from_slice(&complete_with_0(vec![1.0, 2.0]))
        );
        assert_eq!(mirror.radius, 4.0);
    }

    #[test]
    fn test_sphere_mirror_reflect() {}
}
