use super::*;

// TODO: fix bezier mirror implementations

pub struct CubicBezierMirror {
    control_points: Vec<Point<f32, 2>>,
}

impl JsonSerialisable for CubicBezierMirror {
    fn get_type(&self) -> &'static str {
        "cubic_bezier"
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        /* example json
        {
            "control_points": [
                [1., 2., 3., ...],
                [4., 5., 6., ...],
                [7., 8., 9., ...],
                ...
            ]
        }
         */

        let mut control_points = vec![];

        for (i, point_json) in json
            .get("control_points")
            .and_then(Value::as_array)
            .ok_or("Failed to parse control_points")?
            .iter()
            .enumerate()
        {
            control_points.push(
                point_json
                    .as_array()
                    .map(Vec::as_slice)
                    .and_then(json_array_to_float_array)
                    .map(Point::from)
                    .ok_or(f!("Failed to parse {i}th control point"))?,
            );
        }

        Ok(Self { control_points })
    }
}

impl Mirror for CubicBezierMirror {
    fn intersecting_points(&self, ray: &Ray) -> Vec<Tangent> {
        vec![]
    }
}

impl CubicBezierMirror {
    pub fn new(control_points: Vec<Point<f32, 2>>) -> Self {
        Self { control_points }
    }

    pub fn calculate_point(&self, t: f32) -> Point<f32, 2> {
        // P(t) = (1 - t)^3 * P0 + 3t(1-t)^2 * P1 + 3t^2 (1-t) * P2 + t^3 * P3
        let t2 = t * t;
        let t3 = t2 * t;
        let one_minus_t = 1. - t;
        let one_minus_t2 = one_minus_t * one_minus_t;
        let one_minus_t3 = one_minus_t2 * one_minus_t;

        let mut result = Point::origin();

        for i in 0..2 {
            let p0 = &self.control_points[0][i];
            let p1 = &self.control_points[1][i];
            let p2 = &self.control_points[2][i];
            let p3 = &self.control_points[3][i];

            let x = one_minus_t3 * p0
                + 3. * one_minus_t2 * t * p1
                + 3. * one_minus_t * t2 * p2
                + t3 * p3;

            result[i] = x;
        }

        result
    }

    pub fn calculate_tangent(&self, t: f32) -> SVector<f32, 2> {
        // dP(t) / dt =  3(1-t)^2 * (P1-P0) + 6(1-t) * t * (P2 -P1) + 3t^2 * (P3-P2)
        let t2 = t * t;
        let one_minus_t = 1. - t;
        let one_minus_t2 = one_minus_t * one_minus_t;

        let mut result = SVector::<f32, 2>::zeros();

        for i in 0..2 {
            let p0 = &self.control_points[0][i];
            let p1 = &self.control_points[1][i];
            let p2 = &self.control_points[2][i];
            let p3 = &self.control_points[3][i];

            let x = 3. * one_minus_t2 * (p1 - p0)
                + 6. * one_minus_t * t * (p2 - p1)
                + 3. * t2 * (p3 - p2);

            result[i] = x;
        }

        result.normalize()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_calculate_linear_point_2d() {
        let bezier_mirror = CubicBezierMirror {
            control_points: vec![
                [0., 0.].into(),
                [0., 0.].into(),
                [1., 1.].into(),
                [1., 1.].into(),
            ],
        };
        assert_eq!(bezier_mirror.calculate_point(0.), [0., 0.].into());
        assert_eq!(bezier_mirror.calculate_point(0.5), [0.5, 0.5].into());
        assert_eq!(bezier_mirror.calculate_point(1.), [1., 1.].into());
    }

    #[test]
    fn test_ease_in_out_point_2d() {
        let bezier_mirror = CubicBezierMirror {
            control_points: vec![
                [0., 0.].into(),
                [1., 0.].into(),
                [0., 1.].into(),
                [1., 1.].into(),
            ],
        };
        // calculate position
        assert_eq!(bezier_mirror.calculate_point(0.), [0., 0.].into());
        assert_eq!(bezier_mirror.calculate_point(0.5), [0.5, 0.5].into());
        assert_eq!(bezier_mirror.calculate_point(1.), [1., 1.].into());
        // calculate tangent

        assert_eq!(
            bezier_mirror.calculate_tangent(0.),
            SVector::from([1., 0.])
        );
        assert_eq!(
            bezier_mirror.calculate_tangent(0.5),
            SVector::from([0., 1.])
        );
        assert_eq!(
            bezier_mirror.calculate_tangent(1.),
            SVector::from([1., 0.])
        );
    }

    // #[test]
    fn test_ease_in_out_point_3d() {
        let bezier_mirror = CubicBezierMirror {
            control_points: vec![
                [0., 0.].into(),
                [1., 1.].into(),
                [0., 0.].into(),
                [1., 1.].into(),
            ],
        };
        // calculate position
        assert_eq!(bezier_mirror.calculate_point(0.), [0., 0.].into());
        assert_eq!(bezier_mirror.calculate_point(0.5), [0.5, 0.5].into());
        assert_eq!(bezier_mirror.calculate_point(1.), [1., 1.].into());
        // calculate tangent

        assert_eq!(
            bezier_mirror.calculate_tangent(0.)[0],
            bezier_mirror.calculate_tangent(0.)[1]
        );
        assert_ne!(bezier_mirror.calculate_tangent(0.)[0], 0.);
        assert_eq!(
            bezier_mirror.calculate_tangent(0.5),
            SVector::from([0., 0., 1.])
        );
        assert_eq!(
            bezier_mirror.calculate_tangent(1.)[0],
            bezier_mirror.calculate_tangent(1.)[1]
        );
        assert_ne!(bezier_mirror.calculate_tangent(1.)[0], 0.);
    }

    #[test]
    fn test_from_json() {
        let json = serde_json::json!({
            "control_points": [
                [0., 0., 0.],
                [1., 0., 0.],
                [1., 1., 0.],
            ]
        });

        let bezier_mirror =
            CubicBezierMirror::from_json(&json).expect("json deserialisation failed");

        assert_eq!(bezier_mirror.control_points.len(), 3);
        assert_eq!(bezier_mirror.control_points[0], [0., 0.].into());
        assert_eq!(bezier_mirror.control_points[1], [1., 0.].into());
        assert_eq!(bezier_mirror.control_points[2], [1., 1.].into());
    }
}
