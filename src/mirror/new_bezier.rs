use nalgebra::Matrix;
use ndarray::{Array, Axis, Dim};

use super::*;

#[derive(PartialEq, Debug)]
pub struct NewBezierMirror {
    pub control_points: Array<Point<f32, DIM>, Dim<[usize; DIM - 1]>>,
}

impl NewBezierMirror {
    /// Calculate the point on the Bezier curve at a given parameter u and v
    // fn calculate_point2(&self, u: f32, v: f32) -> Point<f32, DIM> {
    //     let (n, m) = (self.control_points.len(), self.control_points[0].len());
    //     let basis_u = bernstein(n, u); // n rows
    //     let basis_v = bernstein(m, v); // m cols
    //     let mut point = Point::origin();
    //     for (i, i_element) in basis_u.iter().enumerate() {
    //         for (j, j_element) in basis_v.iter().enumerate() {
    //             let p = self.control_points[i][j];
    //             let tmp_point = i_element * j_element * p;
    //             for k in 0..DIM {
    //                 point[k] += tmp_point[k];
    //             }
    //         }
    //     }
    //     point
    // }

    fn calculate_point(&self, t: [f32; DIM - 1]) -> Point<f32, DIM> {
        // store the size of all the dimensions of the control points
        let mut sizes: [usize; DIM - 1] = [0; DIM - 1];
        for (index, size) in sizes.iter_mut().enumerate() {
            *size = self.control_points.len_of(Axis(index));
        }

        // // store the bernstein polynomials for each dimension
        // let mut bersteins: [Vec<f32>; DIM - 1] = [Vec::new(); DIM - 1];
        // for (index, berstein) in bersteins.iter_mut().enumerate() {
        //     *berstein = bernstein(sizes[index], t[index]);
        // }
        let mut point = Point::origin();
        point = self.calculate_point_recursive(&mut point, /*&bersteins,*/ &sizes, 0, &mut [0; DIM - 1], &t);

        point
    }

    fn calculate_point_recursive(
        &self,
        point: &mut Point<f32, DIM>,
        // bersteins: &[Vec<f32>; DIM - 1],
        sizes: &[usize; DIM - 1],
        dim: usize,
        index: &mut [usize; DIM - 1],
        t: &[f32; DIM - 1]
    ) -> Point<f32, DIM> {
        if dim == DIM - 1 {
            let mut result = Point::<f32, DIM>::from_slice(&[1.0; DIM]);
            for i in 0..DIM - 1 {
                let tmp = bernstein(sizes[i], index[i], t[i]);
                for j in 0..DIM {
                    result[j] *= tmp;
                }
            }
            for i in 0..DIM {
                result[i] *= self.control_points[*index][i];
            }
            return result;
            // let p = self.control_points[*index];
            // // let mut tmp_point = Point::origin();
            // for j in 0..sizes[dim] {
            //     let mut multiplication: Point<f32, DIM> = Point::from_slice(&[1.0; DIM]);
            //     for k in 0..DIM - 1 {
            //         for l in 0..bersteins[k].len() {
            //             multiplication[k] *= bersteins[k][l];
            //         }
            //     }
            //     for k in 0..DIM {
            //         point[k] += multiplication[k];
            //     }
            // }
        } else {
            let mut result = Point::<f32, DIM>::origin();
            for i in 0..sizes[dim] {
                index[dim] = i;
                let tmp = self.calculate_point_recursive(point, /*bersteins,*/ sizes, dim + 1, index, t);
                for j in 0..DIM {
                    result[j] += tmp[j];
                }
            }
            return result;
        }
    }

    // fn calculate_tangent(&self, u: f32, v: f32) -> SVector<f32, DIM> {
    //     let (n, m) = (self.control_points.len(), self.control_points[0].len());
    //     let basis_u = bernstein(n, u); // n rows
    //     let basis_v = bernstein(m, v); // m cols
    //     let mut tangent = SVector::zeros();
    //     for i in 0..n {
    //         for j in 0..m {
    //             let p = self.control_points[i][j];
    //             let du = basis_u[i] * basis_v[j];
    //             let dv = basis_u[i] * basis_v[j];
    //             let dp = if i > 0 {
    //                 basis_u[i - 1] * basis_v[j] * (p - self.control_points[i - 1][j])
    //             } else {
    //                 SVector::zeros()
    //             } + if i < n - 1 {
    //                 basis_u[i + 1] * basis_v[j] * (self.control_points[i + 1][j] - p)
    //             } else {
    //                 SVector::zeros()
    //             } + if j > 0 {
    //                 basis_u[i] * basis_v[j - 1] * (p - self.control_points[i][j - 1])
    //             } else {
    //                 SVector::zeros()
    //             } + if j < m - 1 {
    //                 basis_u[i] * basis_v[j + 1] * (self.control_points[i][j + 1] - p)
    //             } else {
    //                 SVector::zeros()
    //             };
    //             tangent += du * dp + dv * dp;
    //         }
    //     }
    //     tangent.normalize()
    // }
}

fn bernstein(n: usize, i: usize, t: f32) -> f32 {
    let expo = t.powi(i as i32) * (1.0 - t).powi((n - i) as i32);
    let binom = binomial_coefficient(n, i) as f32;
    expo * binom
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
    use ndarray::ArrayBase;

    use super::*;
    use std::io::Write;
    use crate::mirror::bezier::BezierMirror;

    fn complete_with_0<const N: usize, const O: usize>(arr: [f32; N]) -> [f32; O] {
        let mut result = [0.0; O];
        result.copy_from_slice(&arr[..N]);
        result
    }

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
        let bezier_mirror = NewBezierMirror {
            control_points: Array::from_shape_vec(
                4,
                vec![
                    Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([0.0, 0.0])),
                    Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([1.0, 1.0])),
                    Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([0.0, 1.0])),
                    Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([1.0, 0.0])),
                    // Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([1.0, 0.0])),
                    // Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([1.0, 0.0])),
                ],
            )
            .unwrap(),
        };

        assert_eq!(
            bezier_mirror.calculate_point(complete_with_0([0.0])),
            Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([0.0, 0.0]))
        );

        assert_eq!(
            bezier_mirror.calculate_point(complete_with_0([1.0])),
            Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([0.0, 0.0]))
        );

        assert_eq!(
            bezier_mirror.calculate_point(complete_with_0([0.5])),
            Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([0.5, 0.5]))
        );

        // assert_eq!(
        //     bezier_mirror.calculate_point(0.0, 0.0),
        //     Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0]))
        // );

        // assert_eq!(
        //     bezier_mirror.calculate_point(0.5, 0.5),
        //     Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 0.5]))
        // );

        // assert_eq!(
        //     bezier_mirror.calculate_point(1.0, 1.0),
        //     Point::from_slice(&complete_with_0(vec![1.0, 0.0]))
        // );
    }

    #[test]
    fn test_calculate_linear_3d() {
        // let bezier_mirror = NewBezierMirror {
        //     control_points: vec![
        //         vec![
        //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0, 0.0])),
        //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 1.0, 1.0])),
        //         ],
        //         vec![
        //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 1.0, 0.0])),
        //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 0.0, 1.0])),
        //         ],
        //     ],
        // };

        // assert_eq!(
        //     bezier_mirror.calculate_point(0.0, 0.0),
        //     Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0, 0.0]))
        // );

        // assert_eq!(
        //     bezier_mirror.calculate_point(0.5, 0.5),
        //     Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 0.5, 0.5]))
        // );

        // assert_eq!(
        //     bezier_mirror.calculate_point(1.0, 1.0),
        //     Point::from_slice(&complete_with_0(vec![1.0, 0.0, 1.0]))
        // );
    }

    #[test]
    fn generate_point_in_csv() {
        //simple function to visualize the bezier curve to check that I dont do shit
        let bezier_mirror = NewBezierMirror {
            control_points: Array::from_shape_vec(
                3,
                vec![
                    Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([0.0, 0.0])),
                    Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([1.0, 1.0])),
                    Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([1.0, 0.0])),
                    //Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([0.0, 1.0])),
                    // Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([1.0, 0.0])),
                    // Point::<f32, DIM>::from_slice(&complete_with_0::<2, DIM>([1.0, 0.0])),
                ],
            )
                .unwrap(),
        };

        let mut file = std::fs::File::create("points.csv").unwrap();
        for i in 0..100 {
            let t = i as f32 / 100.0;
            let point = bezier_mirror.calculate_point([t]);
            writeln!(file, "{},{}", point[0], point[1]).unwrap();
            println!("{} : {}", t, point);
        }
    }

    // #[test]
    // fn test_calculate_cubic_point_2d() {
    //     let bezier_mirror = BezierMirror {
    //         control_points: vec![
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0])),
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 1.0])),
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 0.0])),
    //         ],
    //     };
    //     assert_eq!(
    //         bezier_mirror.calculate_point(0.0),
    //         Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0]))
    //     );
    //     assert_eq!(
    //         bezier_mirror.calculate_point(0.5),
    //         Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 0.5]))
    //     );
    //     assert_eq!(
    //         bezier_mirror.calculate_point(1.0),
    //         Point::from_slice(&complete_with_0(vec![1.0, 0.0]))
    //     );
    // }

    // #[test]
    // fn test_calculate_quadratic_point_2d() {
    //     let bezier_mirror = BezierMirror {
    //         control_points: vec![
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0])),
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 0.0])),
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 1.0])),
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 1.0])),
    //         ],
    //     };
    //     assert_eq!(
    //         bezier_mirror.calculate_point(0.0),
    //         Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0]))
    //     );

    //     assert_eq!(
    //         bezier_mirror.calculate_point(0.5),
    //         Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 0.5]))
    //     );

    //     assert_eq!(
    //         bezier_mirror.calculate_point(1.0),
    //         Point::from_slice(&complete_with_0(vec![1.0, 1.0]))
    //     );
    // }

    // #[test]
    // fn generate_point_in_csv() {
    //     //simple function to visualize the bezier curve to check that I dont do shit
    //     let bezier_mirror = BezierMirror {
    //         control_points: vec![
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0])),
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 1.0])),
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 1.0])),
    //         ],
    //     };

    //     let mut file = std::fs::File::create("points.csv").unwrap();
    //     for i in 0..100 {
    //         let t = i as f32 / 100.0;
    //         let point = bezier_mirror.calculate_point(t);
    //         writeln!(file, "{},{}", point[0], point[1]).unwrap();
    //         println!("{} : {}", t, point);
    //     }
    // }

    // #[test]
    // fn test_calculate_tangent() {
    //     let bezier_mirror = BezierMirror {
    //         control_points: vec![
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.0, 0.0])),
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![0.5, 1.0])),
    //             Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 0.0])),
    //         ],
    //     };

    //     let vector = bezier_mirror.calculate_tangent(1.0);
    //     let axis = SVector::<f32, DIM>::from_vec(complete_with_0(vec![1.0, 0.0]));
    //     let dot_product = vector.dot(&axis);
    //     let reflected_vector = 2.0 * dot_product * axis - vector;

    //     assert_eq!(bezier_mirror.calculate_tangent(0.0), reflected_vector);
    // }

    // #[test]
    // fn test_from_json() {
    //     let json = serde_json::json!({
    //         "control_points": [
    //             complete_with_0(vec![1.0, 2.0, 3.0]),
    //             complete_with_0(vec![4.0, 5.0, 6.0]),
    //                 complete_with_0(vec![7.0, 8.0, 9.0]),
    //         ]
    //     });
    //     assert_eq!(
    //         BezierMirror::from_json(&serde_json::to_value(json).unwrap())
    //             .expect("json deserialisation failed"),
    //         BezierMirror {
    //             control_points: vec![
    //                 Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 2.0, 3.0])),
    //                 Point::<f32, DIM>::from_slice(&complete_with_0(vec![4.0, 5.0, 6.0])),
    //                 Point::<f32, DIM>::from_slice(&complete_with_0(vec![7.0, 8.0, 9.0])),
    //             ],
    //         }
    //     );
    // }
}
