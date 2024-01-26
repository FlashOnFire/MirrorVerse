use nalgebra::{Point, SVector};

use crate::{ray::Ray, DIM};

struct BezierMirror {
    control_points: Vec<Point<f32, DIM>>,
}

impl BezierMirror {
    fn reflect(&self, ray: Ray) -> Option<Ray> {
        Some(Ray { ..ray })
    }
    // Method to calculate a point on the Bezier curve
    fn calculate_point(&self, t: f32) -> Point<f32, DIM> {
        let mut point: Point<f32, DIM> = Point::origin();
        let n = self.control_points.len() - 1; // degree of the curve

        for (i, control_point) in self.control_points.iter().enumerate() {
            let bernstein_polynomial = binomial_coefficient(n, i) as f32
                * t.powi(i as i32)
                * (1.0 - t).powi((n - i) as i32);

            for (j, coordinate) in point.iter_mut().enumerate() {
                *coordinate += bernstein_polynomial * control_point[j];
            }
        }

        point
    }

    fn calculate_tangent(&self, t: f32) -> SVector<f32, DIM> {
        let n = self.control_points.len() - 1; // degree of the curve
        let mut tangent: SVector<f32, DIM> = SVector::zeros();

        for i in 0..n {
            let bernstein_derivative = (n as f32)
                * binomial_coefficient(n - 1, i) as f32
                * t.powi(i as i32)
                * (1.0 - t).powi((n - 1 - i) as i32);

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
