use super::*;

/// A parallelotope-shaped reflective (hyper)plane
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ParabolioidMirror<const D: usize = DEFAULT_DIM> {
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

impl<const D: usize> ParabolioidMirror<D> {
    fn new(
        directrix_plane: Plane<D>,
        focus: SVector<f32, D>,
        limit_plane: Plane<D>,
        darkness_coef: f32,
    ) -> Self {
        //calculate the equation of the paraboloid
        let k = directrix_plane.orthogonal_projection(focus);
        let p = (focus - k).norm();//distance between the focus and the directrix plane
        let s: SVector<f32, D> = (focus + k) / 2.; // the center of focus to k
        // we now have the basis (s, (k - focus).normalize(), ...) with      j = K to focus
        //now construct the complete basis
        let mut rng = rand::thread_rng();
        let mut basis: [SVector<f32, D>; D] = [SVector::zeros(); D];
        basis[0] = (k - focus).normalize();
        for i in 1..D {
            let mut count: i8 = 0;
            let mut success = false;
            while !success && count < 100 {
                //put some random values in the new vector
                for j in 0..D {
                    basis[i][j] = rng.gen();
                }

                SVector::orthonormalize(&mut basis);
                success = true;
                //check that there is no equal vectors
                for j in 0..i {
                    if (basis[i] - basis[j]).norm() < 1e-6 || (basis[i] + basis[j]).norm() < 1e-6 {
                        success = false;
                        break;
                    }
                }
                count += 1;
            }
        }
        //here we have a basis and the coef so in 2d the equation is y = x^2/(2p)
        println!("basis: {:?}", basis);
        println!("p: {}", p);
        println!("s: {}", s);
        println!("k: {}", k);
        println!("focus: {}", focus);
        println!("directrix_plane: {:?}", directrix_plane);
        println!("limit_plane: {:?}", limit_plane);
        println!("darkness_coef: {}", darkness_coef);
        println!("y = x^2/(2*{})", p);

        //considerring the code above is right, we now have to do a basis change to the canonical basis
        let original_application_matrix = SMatrix::<f32, D, D>::from_fn(|i, j| basis[j][i]);


        Self {
            directrix_plane,
            focus,
            limit_plane,
            darkness_coef,
        }
    }
}

impl<const D: usize> Mirror<D> for ParabolioidMirror<D> {
    fn intersecting_planes(&self, ray: &Ray<D>) -> Vec<(f32, Plane<D>)> {
        let mut list = vec![];
        self.append_intersecting_planes(ray, &mut list);
        list
    }

    fn append_intersecting_planes(&self, ray: &Ray<D>, list: &mut Vec<(f32, Plane<D>)>) {
        panic!("Not implemented yet");
    }

    fn get_type(&self) -> &str {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mirror::Plane;
    use nalgebra::SVector;

    #[test]
    fn test_new() {
        // Plane::new([
        //     SVector::from_vec(vec![0.0, 0.0]),
        //     SVector::from_vec(vec![1.0, 0.0]),
        // ])
        //     .unwrap()
        let directrix_plane = Plane::new([SVector::from_vec(vec![0., 0.]), SVector::from_vec(vec![1., 0.])]).unwrap();
        let focus = SVector::from_vec(vec![0., 1.]);
        let limit_plane = Plane::new([SVector::from_vec(vec![0., 0.]), SVector::from_vec(vec![0., 1.])]).unwrap();
        let darkness_coef = 0.5;
        let mirror = ParabolioidMirror::new(directrix_plane, focus, limit_plane, darkness_coef);
        assert!(false);
        // assert_eq!(mirror.directrix_plane, directrix_plane);
        // assert_eq!(mirror.focus, focus);
        // assert_eq!(mirror.limit_plane, limit_plane);
        // assert_eq!(mirror.darkness_coef, darkness_coef);
    }
}
