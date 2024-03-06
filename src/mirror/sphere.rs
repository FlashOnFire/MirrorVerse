use super::*;

// TODO: explore the possbility of generalising the definition
// of a sphere to other p-norms (for strictly positive p)

#[derive(Clone, Copy)]
pub struct SphereMirror<const D: usize = DEFAULT_DIM> {
    center: SVector<f32, D>,
    radius: f32,
}

impl<const D: usize> Mirror<D> for SphereMirror<D> {
    fn intersecting_planes(&self, ray: &Ray<D>) -> Vec<(f32, Plane<D>)> {
        // TODO: implement spherical mirror reflection
        vec![]
    }

    fn get_type(&self) -> &str {
        "sphere"
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        /* example json
        {
            "center": [1.0, 2.0, 3.0],
            "radius": 4.0
        }
         */

        let center = json
            .get("center")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .and_then(json_array_to_vector)
            .ok_or_else(|| {
                Box::new(JsonError {
                    message: "Failed to parse center".to_string(),
                }) as Box<dyn std::error::Error>
            })?;

        let radius = json
            .get("radius")
            .and_then(Value::as_f64)
            .ok_or_else(|| {
                Box::new(JsonError {
                    message: "Failed to parse radius".to_string(),
                }) as Box<dyn std::error::Error>
            })
            .unwrap_or_default() as f32;

        Ok(Self { center, radius })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_mirror_from_json() {
        let json = serde_json::json!({
            "center": [1.0, 2.0],
            "radius": 4.0
        });

        let mirror =
            SphereMirror::<DEFAULT_DIM>::from_json(&json).expect("json deserialisation failed");

        assert_eq!(mirror.center, SVector::from([1.0, 2.0]));
        assert_eq!(mirror.radius, 4.0);
    }

    #[test]
    fn test_sphere_mirror_reflect() {}
}
