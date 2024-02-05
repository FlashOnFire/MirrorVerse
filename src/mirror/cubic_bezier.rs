use nalgebra::{Point, SMatrix, SVector, Unit};

use crate::{mirror::Mirror, ray::Ray, DIM};

pub struct CubicBezierMirror {
    control_points: Vec<Point<f32, DIM>>,
}

impl Mirror for CubicBezierMirror {
    fn reflect(&self, ray: Ray) -> Vec<(f32, Unit<SMatrix<f32, DIM, DIM>>)> {
        // use the other mirror to reflect the ray
        vec![]
    }
    fn get_type(&self) -> String {
        "cubicBezier".to_string()
    }
}

impl CubicBezierMirror {
    pub fn new(control_points: Vec<Point<f32, DIM>>) -> Self {
        Self { control_points }
    }

    pub fn calculate_point(&self, t: f32) -> Point<f32, DIM> {
        // P(t) = (1 - t)^3 * P0 + 3t(1-t)^2 * P1 + 3t^2 (1-t) * P2 + t^3 * P3
        let t2 = t * t;
        let t3 = t2 * t;
        let one_minus_t = 1.0 - t;
        let one_minus_t2 = one_minus_t * one_minus_t;
        let one_minus_t3 = one_minus_t2 * one_minus_t;

        let mut result = Point::origin();

        for i in 0..DIM {
            let p0 = &self.control_points[0][i];
            let p1 = &self.control_points[1][i];
            let p2 = &self.control_points[2][i];
            let p3 = &self.control_points[3][i];

            let x = one_minus_t3 * p0
                + 3.0 * one_minus_t2 * t * p1
                + 3.0 * one_minus_t * t2 * p2
                + t3 * p3;

            result[i] = x;
        }

        result
    }

    pub fn calculate_tangent(&self, t: f32) -> SVector<f32, DIM> {
        // dP(t) / dt =  3(1-t)^2 * (P1-P0) + 6(1-t) * t * (P2 -P1) + 3t^2 * (P3-P2)
        let t2 = t * t;
        let one_minus_t = 1.0 - t;
        let one_minus_t2 = one_minus_t * one_minus_t;

        let mut result = SVector::<f32, DIM>::zeros();

        for i in 0..DIM {
            let p0 = &self.control_points[0][i];
            let p1 = &self.control_points[1][i];
            let p2 = &self.control_points[2][i];
            let p3 = &self.control_points[3][i];

            let x = 3.0 * one_minus_t2 * (p1 - p0)
                + 6.0 * one_minus_t * t * (p2 - p1)
                + 3.0 * t2 * (p3 - p2);

            result[i] = x;
        }

        //divise all the components by the pgcd
        result.normalize()
    }

    fn from_json(json: &serde_json::Value) -> Self {
        /* example json
        {
            "control_points": [
                [1.0, 2.0, 3.0, ...],
                [4.0, 5.0, 6.0, ...],
                [7.0, 8.0, 9.0, ...],
                ...
            ]
        }
         */
        let control_points = json["control_points"]
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

        Self { control_points }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn complete_with_0(mut vec: Vec<f32>) -> Vec<f32> {
        vec.resize(DIM, 0.0);
        vec
    }

    #[test]
    fn test_calculate_linear_point_2d() {
        let bezier_mirror = CubicBezierMirror {
            control_points: vec![
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0])),
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0])),
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 1.0])),
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 1.0])),
            ],
        };
        assert_eq!(
            bezier_mirror.calculate_point(0.0),
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0]))
        );
        assert_eq!(
            bezier_mirror.calculate_point(0.5),
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 0.5]))
        );
        assert_eq!(
            bezier_mirror.calculate_point(1.0),
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 1.0]))
        );
    }

    #[test]
    fn test_ease_in_out_point_2d() {
        let bezier_mirror = CubicBezierMirror {
            control_points: vec![
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0])),
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 0.0])),
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 1.0])),
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 0.0])),
            ],
        };
        // calculate position
        assert_eq!(
            bezier_mirror.calculate_point(0.0),
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0]))
        );
        assert_eq!(
            bezier_mirror.calculate_point(0.5),
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.875, 0.375]))
        );
        assert_eq!(
            bezier_mirror.calculate_point(1.0),
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 0.0]))
        );
        // calculate tangent

        assert_eq!(
            bezier_mirror.calculate_tangent(0.0),
            SVector::<f32, DIM>::from_vec(complete_with_0(vec![1.0, 0.0]))
        );
        assert_eq!(
            bezier_mirror.calculate_tangent(0.5)[0],
            bezier_mirror.calculate_tangent(0.5)[1]
        );
        assert_eq!(
            bezier_mirror.calculate_tangent(1.0),
            SVector::<f32, DIM>::from_vec(complete_with_0(vec![0.0, -1.0]))
        );
    }

    #[test]
    fn generate_point_in_csv() {
        let bezier_mirror = CubicBezierMirror {
            control_points: vec![
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0])),
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 0.0])),
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 1.0])),
                Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 1.0])),
            ],
        };

        let mut file = std::fs::File::create("points.csv").unwrap();
        for i in 0..100 {
            let t = i as f32 / 100.0;
            let point = bezier_mirror.calculate_point(t);
            writeln!(file, "{},{}", point[0], point[1]).unwrap();
        }
    }

    #[test]
    fn test_from_json() {
        let json = serde_json::json!({
            "control_points": [
                complete_with_0(vec![0.0, 0.0]),
                complete_with_0(vec![1.0, 0.0]),
                complete_with_0(vec![1.0, 1.0]),
            ]
        });

        let bezier_mirror = CubicBezierMirror::from_json(&json);

        assert_eq!(bezier_mirror.control_points.len(), 3);
        assert_eq!(
            bezier_mirror.control_points[0],
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0]))
        );
        assert_eq!(
            bezier_mirror.control_points[1],
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 0.0]))
        );
        assert_eq!(
            bezier_mirror.control_points[2],
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 1.0]))
        );
    }
}
