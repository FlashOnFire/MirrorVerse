use super::*;

// TODO: fix bezier mirror implementations

#[derive(PartialEq, Debug)]
pub struct BezierMirror {
    control_points: Vec<Point<f32, 2>>,
}

impl Mirror<2> for BezierMirror {
    fn append_intersecting_points(&self, ray: &Ray<2>, list: &mut Vec<Tangent<2>>) {
        todo!()
    }

    fn get_json_type() -> String {
        "bezier".into()
    }

    fn get_json_type_inner(&self) -> String {
        "bezier".into()
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        /* example json {
            "control_points": [
                [1., 2.],
                [3., 4.],
            ]
        } */

        let mut control_points = vec![];

        for (i, point_json) in json
            .get("control_points")
            .and_then(serde_json::Value::as_array)
            .ok_or("Failed to parse control_points")?
            .iter()
            .enumerate()
        {
            control_points.push(
                point_json
                    .as_array()
                    .map(Vec::as_slice)
                    .and_then(util::json_array_to_float_array)
                    .map(Point::from)
                    .ok_or(f!("Failed to parse {i}th control point"))?,
            );
        }

        Ok(Self { control_points })
    }

    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        todo!()
    }

    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        todo!()
    }

    fn random() -> Self
        where
            Self: Sized {
        todo!()
    }
}

impl BezierMirror {
    // Method to calculate a point on the Bezier curve
    fn calculate_point(&self, t: f32) -> Point<f32, 2> {
        let mut point: Point<f32, 2> = Point::origin();
        let n = self.control_points.len() - 1; // degree of the curve

        for (i, control_point) in self.control_points.iter().enumerate() {
            let bernstein_polynomial = binomial_coefficient(n, i) as f32
                * t.powi(i as i32)
                * (1. - t).powi((n - i) as i32);

            for (j, coordinate) in point.iter_mut().enumerate() {
                *coordinate += bernstein_polynomial * control_point[j];
            }
        }

        point
    }

    fn calculate_tangent(&self, t: f32) -> SVector<f32, 2> {
        let n = self.control_points.len() - 1; // degree of the curve
        let mut tangent: SVector<f32, 2> = SVector::zeros();

        for i in 0..n {
            let bernstein_derivative = (n as f32)
                * binomial_coefficient(n - 1, i) as f32
                * t.powi(i as i32)
                * (1. - t).powi((n - 1 - i) as i32);

            let difference = self.control_points[i + 1] - self.control_points[i];
            tangent += bernstein_derivative * difference;
        }

        tangent.normalize()
    }
}

// Function to calculate binomial coefficients
fn binomial_coefficient(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }

    let mut result = 1;
    for i in 0..k {
        result *= n - i;
        result /= i + 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binomial_coefficient() {
        assert_eq!(binomial_coefficient(0, 0), 1);
        assert_eq!(binomial_coefficient(1, 0), 1);
        assert_eq!(binomial_coefficient(1, 1), 1);
        assert_eq!(binomial_coefficient(2, 0), 1);
        assert_eq!(binomial_coefficient(2, 1), 2);
        assert_eq!(binomial_coefficient(2, 2), 1);
        assert_eq!(binomial_coefficient(3, 0), 1);
        assert_eq!(binomial_coefficient(3, 1), 3);
        assert_eq!(binomial_coefficient(3, 2), 3);
        assert_eq!(binomial_coefficient(3, 3), 1);
        assert_eq!(binomial_coefficient(4, 0), 1);
        assert_eq!(binomial_coefficient(4, 1), 4);
        assert_eq!(binomial_coefficient(4, 2), 6);
        assert_eq!(binomial_coefficient(4, 3), 4);
        assert_eq!(binomial_coefficient(4, 4), 1);
    }

    #[test]
    fn test_calculate_linear_point_2d() {
        let bezier_mirror = BezierMirror {
            control_points: vec![[0., 0.].into(), [1., 1.].into()],
        };
        assert_eq!(bezier_mirror.calculate_point(0.), [0., 0.].into());
        assert_eq!(bezier_mirror.calculate_point(0.5), [0.5, 0.5].into());
        assert_eq!(bezier_mirror.calculate_point(1.), [1., 1.].into());
    }

    #[test]
    fn test_calculate_cubic_point_2d() {
        let bezier_mirror = BezierMirror {
            control_points: vec![[0., 0.].into(), [0.5, 1.].into(), [1., 0.].into()],
        };
        assert_eq!(bezier_mirror.calculate_point(0.), [0., 0.].into());
        assert_eq!(bezier_mirror.calculate_point(0.5), [0.5, 0.5].into());
        assert_eq!(bezier_mirror.calculate_point(1.), [1., 0.].into());
    }

    #[test]
    fn test_calculate_quadratic_point_2d() {
        let bezier_mirror = BezierMirror {
            control_points: vec![
                [0., 0.].into(),
                [0.5, 0.].into(),
                [0.5, 1.].into(),
                [1., 1.].into(),
            ],
        };
        assert_eq!(bezier_mirror.calculate_point(0.), [0., 0.].into());

        assert_eq!(bezier_mirror.calculate_point(0.5), [0.5, 0.5].into());

        assert_eq!(bezier_mirror.calculate_point(1.), [1., 1.].into());
    }

    #[test]
    fn test_calculate_tangent() {
        let bezier_mirror = BezierMirror {
            control_points: vec![[0., 0.].into(), [0.5, 1.].into(), [1., 0.].into()],
        };

        let vector = bezier_mirror.calculate_tangent(1.);
        let axis = SVector::<f32, 2>::from_vec(vec![1., 0., 0.]);
        let dot_product = vector.dot(&axis);
        let reflected_vector = 2. * dot_product * axis - vector;

        assert_eq!(bezier_mirror.calculate_tangent(0.), reflected_vector);
    }

    #[test]
    fn test_from_json() {
        let json = serde_json::json!({
            "control_points": [
                [1., 2.],
                [3., 4.],
            ]
        });
        assert_eq!(
            BezierMirror::from_json(&serde_json::to_value(json).unwrap())
                .expect("json deserialisation failed"),
            BezierMirror {
                control_points: vec![[1., 2.].into(), [3., 4.].into(),],
            }
        );
    }
}
