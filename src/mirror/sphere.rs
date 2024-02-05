use super::*;

#[derive(Clone, Copy)]
pub struct SphereMirror<const D: usize = DIM> {
    center: Point<f32, D>,
    radius: f32,
}

impl<const D: usize> Mirror<D> for SphereMirror<D> {
    fn reflect(&self, ray: Ray<D>) -> Vec<(f32, Unit<SMatrix<f32, D, D>>)> {
        vec![]
    }
    fn get_type(&self) -> String {
        "sphere".to_string()
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

        let radius = json["radius"].as_f64().unwrap() as f32;

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
        vec.resize(DIM, 0.0);
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
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 2.0]))
        );
        assert_eq!(mirror.radius, 4.0);
    }

    #[test]
    fn test_sphere_mirror_reflect() {}

    #[test]
    fn test_composite_mirror_from_json() {
        let json = serde_json::json!({
            "mirrors": [
                {
                    "type": "plane",
                    "points": [
                        complete_with_0(vec![1.0, 2.0]),
                        complete_with_0(vec![3.0, 4.0]),
                    ]
                },
                {
                    "type": "sphere",
                    "center": complete_with_0(vec![5.0, 6.0]),
                    "radius": 7.0
                },
            ]
        });

        let mirror = CompositeMirror::<Box<dyn Mirror<DIM>>, DIM>::from_json(&json)
            .expect("json deserialisation failed");

        assert_eq!(mirror.mirrors.len(), 2);
        //check the first is a plane mirror
        assert_eq!(mirror.mirrors[0].get_type(), "plane");
        assert_eq!(mirror.mirrors[1].get_type(), "sphere");
    }
}
