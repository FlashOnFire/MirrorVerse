use nalgebra::{Point, SMatrix, Unit};

use crate::{ray::Ray, DIM};

pub trait Mirror {
    fn reflect(&self, ray: Ray) -> Vec<(f32, Unit<SMatrix<f32, DIM, DIM>>)>;
}

struct CompositeMirror {
    mirrors: Vec<Box<dyn Mirror>>,
}

impl Mirror for CompositeMirror {
    fn reflect(&self, ray: Ray) -> Vec<(f32, Unit<SMatrix<f32, DIM, DIM>>)> {
        // use the other mirror to reflect the ray
        vec![]
    }
}

impl CompositeMirror {
    fn from_json(json: &serde_json::Value) -> Self {
        /* example json
        {
            "mirrors": [
                {
                    "type": "plane",
                    "points": [
                        [1.0, 2.0, 3.0, ...],
                        [4.0, 5.0, 6.0, ...],
                        [7.0, 8.0, 9.0, ...],
                        ...
                    ]
                },
                {
                    "type": "sphere",
                    "center": [1.0, 2.0, 3.0],
                    "radius": 4.0
                },
                ...
            ]
        }
         */
        let mirrors = json["mirrors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|mirror| {
                let mirror_type = mirror["type"].as_str().unwrap();

                match mirror_type {
                    "plane" => Box::new(PlaneMirror::from_json(mirror)) as Box<dyn Mirror>,
                    "sphere" => Box::new(SphereMirror::from_json(mirror)) as Box<dyn Mirror>,
                    _ => panic!("Unknown mirror type: {}", mirror_type),
                }
            })
            .collect::<Vec<_>>();

        Self { mirrors }
    }
}

#[derive(Clone, Copy)]
struct PlaneMirror {
    points: [Point<f32, DIM>; DIM],
}

impl Mirror for PlaneMirror {
    fn reflect(&self, ray: Ray) -> Vec<(f32, Unit<SMatrix<f32, DIM, DIM>>)> {
        vec![]
    }
}

impl PlaneMirror {
    fn from_json(json: &serde_json::Value) -> Self {
        /* example json
        {
            "points": [
                [1.0, 2.0, 3.0, ...],
                [4.0, 5.0, 6.0, ...],
                [7.0, 8.0, 9.0, ...],
                ...
            ]
        }
         */
        let points = json["points"]
            .as_array()
            .unwrap()
            .iter()
            .map(|point| {
                let point = point
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_f64().unwrap() as f32)
                    .collect::<Vec<_>>();

                Point::from_slice(&point)
            })
            .collect::<Vec<_>>();

        let mut mirror_points = [Point::origin(); DIM];
        for (i, point) in points.iter().enumerate() {
            mirror_points[i] = *point;
        }

        Self {
            points: mirror_points,
        }
    }
}

#[derive(Clone, Copy)]
struct SphereMirror {
    center: Point<f32, DIM>,
    radius: f32,
}

impl Mirror for SphereMirror {
    fn reflect(&self, ray: Ray) -> Vec<(f32, Unit<SMatrix<f32, DIM, DIM>>)> {
        vec![]
    }
}

impl SphereMirror {
    fn from_json(json: &serde_json::Value) -> Self {
        /* example json
        {
            "center": [1.0, 2.0, 3.0],
            "radius": 4.0
        }
         */
        let center = json["center"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_f64().unwrap() as f32)
            .collect::<Vec<_>>();

        let radius = json["radius"].as_f64().unwrap() as f32;

        Self {
            center: Point::from_slice(&center),
            radius,
        }
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
    fn test_plane_mirror_from_json() {
        let json = serde_json::json!({
            "points": [
                complete_with_0(vec![1.0, 2.0]),
                complete_with_0(vec![3.0, 4.0]),
            ]
        });

        let mirror = PlaneMirror::from_json(&json);

        assert_eq!(
            mirror.points[0],
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 2.0]))
        );
        assert_eq!(
            mirror.points[1],
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![3.0, 4.0]))
        );
    }

    #[test]
    fn test_plane_mirror_reflect() {}

    #[test]
    fn test_sphere_mirror_from_json() {
        println!("oucou");
        let json = serde_json::json!({
            "center": complete_with_0(vec![1.0, 2.0]),
            "radius": 4.0
        });

        let mirror = SphereMirror::from_json(&json);

        assert_eq!(
            mirror.center,
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 2.0]))
        );
        assert_eq!(mirror.radius, 4.0);
    }

    #[test]
    fn test_sphere_mirror_reflect() {}
}
