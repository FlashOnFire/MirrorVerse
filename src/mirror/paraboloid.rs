use nalgebra::{Point2, Vector2};

use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ParaboloidMirror<const D: usize = DEFAULT_DIM> {
    /// The plane this mirror belongs to.
    directrix_plane: Plane<D>,
    /// The focus the parabola is centered on
    focus: SVector<f32, D>,
    /// The limit of the parabola
    limit_plane: Plane<D>,
    /// Coefficient describing the darkness of the mirror which will be applied to the brightness
    darkness_coef: f32,
    // /// the coefficient of the parabola (distance between the home and the guide plane)
    // magic_coef: f32,
}

impl<const D: usize> ParaboloidMirror<D> {
    fn is_point_on_parabola(&self, point: &SVector<f32, D>) -> bool {
        let dist_to_directrix =
            (self.directrix_plane.orthogonal_point_projection(*point) - *point).norm();
        let dist_to_focus = (self.focus - *point).norm();
        let distance_ok = dist_to_directrix.powi(2) - 2. * dist_to_focus < f32::EPSILON;
        //check if the point is on the right side of the limit plane
        let point_projection_on_limit_plane = self.limit_plane.orthogonal_projection(*point);
        let focus_projection_on_limit_plane = self.limit_plane.orthogonal_projection(self.focus);
        //check if the two vector are in the same direction
        let same_direction = (point_projection_on_limit_plane - focus_projection_on_limit_plane)
            .dot(&(point - focus_projection_on_limit_plane))
            > f32::EPSILON;
        distance_ok && same_direction
    }
}

impl ParaboloidMirror<2> {
    fn get_tangent(&self, point: &SVector<f32, 2>) -> Option<Plane<2>> {
        if !self.is_point_on_parabola(point) {
            return None;
        }
        //calculate the line to the directrix
        let point_to_directrix_direction =
            self.directrix_plane.orthogonal_point_projection(*point) - *point;
        //calculate the line to the focus
        let point_to_focus_direction = self.focus - *point;

        //calculate the tangent
        let direction = point_to_directrix_direction + point_to_focus_direction;

        Some(Plane::new([*point, direction]).unwrap())
    }

    // fn new(
    //     directrix_plane: Plane<D>,
    //     focus: SVector<f32, D>,
    //     limit_plane: Plane<D>,
    //     darkness_coef: f32,
    // ) -> Self {
    //     //ax+by+c=0
    //     //F=(f_1,f_2)
    //     //(ax+by+c)^2/(a^2+b^2)=(x-f_1)^2+(y-f_2)^2
    //
    //     /*
    //
    //     //calculate the equation of the paraboloid
    //     let k = directrix_plane.orthogonal_projection(focus);
    //     let p = (focus - k).norm(); //distance between the focus and the directrix plane
    //     let s: SVector<f32, D> = (focus + k) / 2.; // the center of focus to k
    //                                                // we now have the basis (s, (k - focus).normalize(), ...) with      j = K to focus
    //                                                //now construct the complete basis
    //     let mut rng = rand::thread_rng();
    //     let mut basis: [SVector<f32, D>; D] = [SVector::zeros(); D];
    //     basis[0] = (k - focus).normalize();
    //     for i in 1..D {
    //         let mut count: i8 = 0;
    //         let mut success = false;
    //         while !success && count < 100 {
    //             //put some random values in the new vector
    //             for j in 0..D {
    //                 basis[i][j] = rng.gen();
    //             }
    //
    //             SVector::orthonormalize(&mut basis);
    //             success = true;
    //             //check that there is no equal vectors
    //             for j in 0..i {
    //                 if (basis[i] - basis[j]).norm() < 1e-6 || (basis[i] + basis[j]).norm() < 1e-6 {
    //                     success = false;
    //                     break;
    //                 }
    //             }
    //             count += 1;
    //         }
    //     }
    //     //here we have a basis and the coef so in 2d the equation is y = x^2/(2p)
    //     println!("basis: {:?}", basis);
    //     println!("p: {}", p);
    //     println!("s: {}", s);
    //     println!("k: {}", k);
    //     println!("focus: {}", focus);
    //     println!("directrix_plane: {:?}", directrix_plane);
    //     println!("limit_plane: {:?}", limit_plane);
    //     println!("darkness_coef: {}", darkness_coef);
    //     println!("y = x^2/(2*{})", p);
    //
    //     //considerring the code above is right, we now have to do a basis change to the canonical basis
    //     let original_application_matrix = SMatrix::<f32, D, D>::from_fn(|i, j| basis[j][i]);
    //
    //     // create the transformation matrix to the new basis
    //     let mut transformation_matrix = SMatrix::<f32, D, D>::zeros();
    //
    //     */
    //
    //     Self {
    //         directrix_plane,
    //         focus,
    //         limit_plane,
    //         darkness_coef,
    //     }
    // }
}

impl<const D: usize> Mirror<D> for ParaboloidMirror<D> {
    fn intersecting_points(&self, ray: &Ray<D>) -> Vec<(f32, Tangent<D>)> {
        let mut list = vec![];
        self.append_intersecting_points(ray, &mut list);
        list
    }

    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<(f32, Tangent<D>)>) {
        // Define the focus and directrix
        let focus = Point2::new(self.focus[0], self.focus[1]); // Focus of the parabola
        let directrix_point =
            Point2::new(self.directrix_plane.v_0()[0], self.directrix_plane.v_0()[1]); // A point on the directrix line
        let directrix_vector = Vector2::new(
            self.directrix_plane.basis()[0][0],
            self.directrix_plane.basis()[0][1],
        ); // Direction vector of the directrix line

        // Define the line
        let line_point = Point2::new(ray.origin[0], ray.origin[1]); // A point on the line
        let line_direction = Unit::new_normalize(Vector2::new(ray.direction[0], ray.direction[1])); // Direction vector of the line

        let func = |t: f32| -> f32 {
            //x and y of the line
            let x = line_point[0] + t * line_direction[0];
            let y = line_point[1] + t * line_direction[1];
            let dx = x - directrix_point[0];
            let dy = y - directrix_point[1];
            let numerator = (x - focus[0]).powi(2) + (y - focus[1]).powi(2);
            let denominator = directrix_vector[1].powi(2) + directrix_vector[0].powi(2);
            numerator - (dx * directrix_vector[1] - dy * directrix_vector[0]).powi(2) / denominator
        };

        // Solve the equation
        let t0 = 1.0; // Initial guess for the first root
        let solution = newton_raphson(t0, func).unwrap(); // You need to implement the Newton-Raphson method
        let mut intersection_points = [Point2::new(0.0, 0.0); 2];
        intersection_points[0] = line_point + solution * line_direction.into_inner();

        //calculate the t1 by adding the distance beetween the ray and the focus or substract if if we are on the right side

        let ray_to_focus = focus - line_point;
        let mut t1: f32;
        if ray_to_focus.dot(&line_direction) > 0. {
            t1 = solution + ray_to_focus.norm();
        } else {
            t1 = solution - ray_to_focus.norm();
        }

        let solution = newton_raphson(t1, func).unwrap(); // You need to implement the Newton-Raphson method
        intersection_points[1] = line_point + solution * line_direction.into_inner();

        for intersection_point in intersection_points.iter() {
            if self.is_point_on_parabola(&SVector::from_vec(vec![
                intersection_point[0],
                intersection_point[1],
            ])) {
                list.push((
                    0.0,
                    Tangent::new(
                        SVector::from_vec(vec![intersection_point[0], intersection_point[1]]),
                        Unit::new_normalize(SVector::from_vec(vec![1., 1.])), //TODO with the new method of momo aucun soucis on utilise la tangent
                                                                              //self.get_tangent(&SVector::from_vec(vec![intersection_point[0], intersection_point[1]])).unwrap(),
                    ),
                ));
            }
        }
    }

    fn get_type(&self) -> &'static str {
        "parabola"
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        /*
        example json:

        */

        return Err("Not implemented yet".into());
    }
}

fn newton_raphson<F>(guess: f32, f: F) -> Option<f32>
where
    F: Fn(f32) -> f32,
{
    let mut x = guess;
    let mut dx;

    for _ in 0..1000 {
        // Maximum 1000 iterations
        dx = f(x) / (f(x + 0.01) - f(x)) * 0.01; // Numerical derivative
        if dx.abs() < f32::EPSILON {
            // Convergence criterion
            return Some(x);
        }
        x -= dx;
    }

    None // Did not converge
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_1_SQRT_2;

    use nalgebra::SVector;

    use crate::mirror::Plane;

    use super::*;

    #[test]
    fn test_intersection() {
        let directrix_plane =
            Plane::new([SVector::from([0., 0.]), SVector::from([1., 0.])]).unwrap();
        let focus = SVector::from([0., 1.]);
        let limit_plane = Plane::new([SVector::from([0., 2.]), SVector::from([1., 0.])]).unwrap();
        let darkness_coef = 0.5;
        let mirror = ParaboloidMirror {
            directrix_plane,
            focus,
            limit_plane,
            darkness_coef,
        };

        let ray = Ray {
            origin: SVector::from([-10., 1.]),
            direction: Unit::new_normalize(SVector::from([1., 0.])),
            brightness: 1.0,
        };
        let mut list = vec![];
        mirror.append_intersecting_points(&ray, &mut list);
        println!("{:?}", list);

        assert_eq!(list.len(), 2);
        assert_eq!(list[0].1.origin, SVector::<f32, 2>::from([-1., 1.]));
        assert_eq!(list[1].1.origin, SVector::<f32, 2>::from([1., 1.]));
    }

    #[test]
    fn test_intersection_2() {
        let directrix_plane =
            Plane::new([SVector::from([0., 0.]), SVector::from([0., 1.])]).unwrap();
        let focus = SVector::from([-1., 0.]);
        let limit_plane = Plane::new([SVector::from([0., 0.]), SVector::from([0., 1.])]).unwrap();
        let darkness_coef = 0.5;
        let mirror = ParaboloidMirror {
            directrix_plane,
            focus,
            limit_plane,
            darkness_coef,
        };

        let ray = Ray {
            origin: SVector::from([-1., -10.]),
            direction: Unit::new_normalize(SVector::from([0., 1.])),
            brightness: 1.0,
        };
        let mut list = vec![];
        mirror.append_intersecting_points(&ray, &mut list);
        println!("{:?}", list);

        assert_eq!(list.len(), 2);
        assert_eq!(list[0].1.origin, SVector::<f32, 2>::from([-1., -1.]));
        assert_eq!(list[1].1.origin, SVector::<f32, 2>::from([-1., 1.]));
    }

    #[test]
    fn test_intersection_3() {
        let directrix_plane =
            Plane::new([SVector::from([0., 0.]), SVector::from([1., 1.])]).unwrap();
        let focus = SVector::from([-1., 1.]);
        let limit_plane = Plane::new([SVector::from([0., 0.]), SVector::from([0., 1.])]).unwrap();
        let darkness_coef = 0.5;
        let mirror = ParaboloidMirror {
            directrix_plane,
            focus,
            limit_plane,
            darkness_coef,
        };

        let ray = Ray {
            origin: SVector::from([-4., -2.]),
            direction: Unit::new_normalize(SVector::from([1., 1.])),
            brightness: 1.0,
        };
        let mut list = vec![];
        mirror.append_intersecting_points(&ray, &mut list);
        println!("{:?}", list);

        assert_eq!(list.len(), 2);
        assert!((list[0].1.origin - SVector::<f32, 2>::from([-2., 0.])).norm() < f32::EPSILON);
        assert!((list[1].1.origin - SVector::<f32, 2>::from([0., 2.])).norm() < f32::EPSILON);
    }

    #[test]
    fn test_tangent_1() {
        let directrix_plane = Plane::new([
            SVector::from_vec(vec![1., 0.]),
            SVector::from_vec(vec![0., 1.]),
        ])
        .unwrap();
        let focus = SVector::from_vec(vec![0., 0.]);
        let limit_plane = Plane::new([
            SVector::from_vec(vec![0., 0.]),
            SVector::from_vec(vec![0., 1.]),
        ])
        .unwrap();
        let darkness_coef = 0.5;
        let mirror = ParaboloidMirror {
            directrix_plane,
            focus,
            limit_plane,
            darkness_coef,
        };

        let point = SVector::from_vec(vec![0., -1.]);
        let tangent = mirror.get_tangent(&point).unwrap();
        assert_eq!(*tangent.v_0(), point);
        assert_eq!(
            tangent.basis()[0],
            SVector::<f32, 2>::from_vec(vec![FRAC_1_SQRT_2, FRAC_1_SQRT_2])
        );
    }

    #[test]
    fn test_tangent_2() {
        let directrix_plane = Plane::new([
            SVector::from_vec(vec![1., 0.]),
            SVector::from_vec(vec![0., 1.]),
        ])
        .unwrap();
        let focus = SVector::from([0., 0.]);
        let limit_plane = Plane::new([SVector::from([0., 0.]), SVector::from([0., 1.])]).unwrap();
        let darkness_coef = 0.5;
        let mirror = ParaboloidMirror {
            directrix_plane,
            focus,
            limit_plane,
            darkness_coef,
        };

        let point = SVector::from([0., 1.]);
        let tangent = mirror.get_tangent(&point).unwrap();
        assert_eq!(*tangent.v_0(), point);
        assert_eq!(
            tangent.basis()[0],
            SVector::<f32, 2>::from([FRAC_1_SQRT_2, -FRAC_1_SQRT_2])
        );
    }

    #[test]
    fn test_tangent_3() {
        let directrix_plane =
            Plane::new([SVector::from([0., -2.]), SVector::from([1., 1.])]).unwrap();
        let focus = SVector::from([0., 0.]);
        let limit_plane = Plane::new([SVector::from([0., 0.]), SVector::from([0., 1.])]).unwrap();
        let darkness_coef = 0.5;
        let mirror = ParaboloidMirror {
            directrix_plane,
            focus,
            limit_plane,
            darkness_coef,
        };

        let point = SVector::from([1., 1.]);
        let tangent = mirror.get_tangent(&point).unwrap();
        assert_eq!(*tangent.v_0(), point);
        assert!(
            tangent.basis()[0] - SVector::<f32, 2>::from([0., -1.])
                < SVector::<f32, 2>::from([f32::EPSILON, f32::EPSILON])
        );
    }

    #[test]
    fn test_tangent_4() {
        let directrix_plane =
            Plane::new([SVector::from([0., -2.]), SVector::from([1., 1.])]).unwrap();
        let focus = SVector::from([0., 0.]);
        let limit_plane = Plane::new([SVector::from([0., 0.]), SVector::from([0., 1.])]).unwrap();
        let darkness_coef = 0.5;
        let mirror = ParaboloidMirror {
            directrix_plane,
            focus,
            limit_plane,
            darkness_coef,
        };

        let point = SVector::from([-1., -1.]);
        let tangent = mirror.get_tangent(&point).unwrap();
        assert_eq!(*tangent.v_0(), point);
        assert!(
            tangent.basis()[0] - SVector::<f32, 2>::from([1., 1.])
                < SVector::<f32, 2>::from([f32::EPSILON, f32::EPSILON])
        );
    }

    #[test]
    fn test_point_on_parabola_1() {
        let directrix_plane =
            Plane::new([SVector::from([0., -2.]), SVector::from([1., 1.])]).unwrap();
        let focus = SVector::from([0., 0.]);
        let limit_plane = Plane::new([SVector::from([0., 0.]), SVector::from([0., 1.])]).unwrap();
        let darkness_coef = 0.5;
        let mirror = ParaboloidMirror {
            directrix_plane,
            focus,
            limit_plane,
            darkness_coef,
        };

        assert!(mirror.is_point_on_parabola(&SVector::from([-1., -1.])));
        assert!(mirror.is_point_on_parabola(&SVector::from([1., 1.])));
        assert!(!mirror.is_point_on_parabola(&SVector::from([-2., -1.])));
        assert!(!mirror.is_point_on_parabola(&SVector::from([1., 2.])));
        assert!(mirror.is_point_on_parabola(&SVector::from([2., 1.])));
    }
}
