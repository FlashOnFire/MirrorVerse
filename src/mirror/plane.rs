use super::*;

/// A parallelotope-shaped reflective (hyper)plane
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlaneMirror<const D: usize = DEFAULT_DIM> {
    /// The plane this mirror belongs to.
    plane: Plane<D>,
    /// maximum magnitudes (mu_i_max) of the scalars in the linear combination of the
    /// basis vectors of the associated hyperplane.
    ///
    /// Formally, for all vectors `v = sum mu_i * v_i` of
    /// the hyperplane, `v` is in this plane mirror iff for all `i`, `|mu_i| <= |mu_i_max|`
    ///
    /// Note: the first value of this array is irrelevant
    bounds: [f32; D],
}

impl<const D: usize> Mirror<D> for PlaneMirror<D> {
    fn reflect(&self, ray: &Ray<D>) -> Vec<(f32, Plane<D>)> {
        let mut list = vec![];
        self.append_reflections(ray, &mut list);
        list
    }

    fn append_reflections(&self, ray: &Ray<D>, list: &mut Vec<(f32, Plane<D>)>) {
        let mut a = SMatrix::<f32, D, D>::zeros();

        a.column_iter_mut()
            .zip(iter::once(ray.direction.as_ref()).chain(self.plane.basis().iter()))
            .for_each(|(mut i, o)| i.set_column(0, o));

        if a.try_inverse_mut() {
            // a now contains a^-1
            let v = a * (self.plane.v_0() - ray.direction.as_ref());
            if v.iter()
                .zip(&self.bounds)
                .skip(1)
                .all(|(mu, mu_max)| mu.abs() <= mu_max.abs())
            {
                list.push((1., self.plane));
            }
        }
    }

    fn get_type(&self) -> &str {
        "plane"
    }

    fn from_json(json: &serde_json::Value) -> Option<Self>
    where
        Self: Sized,
    {
        /*
        example json:
        {
            "center": [9., 8., 7., ...], (N elements)
            "basis": [ (N - 1 elements)
                [9., 8., 7., ...], (N elements)
                [6., 5., 4., ...],
            ],
            "bounds": [6., 9., ...] (N - 1 elements)
        }
        */

        let mut plane = Plane::new([SVector::zeros(); D]);

        *plane.v_0_mut() = json_array_to_vector(json.get("center")?.as_array()?.as_slice())?;

        let basis = json.get("basis")?.as_array().filter(|l| l.len() == D - 1)?;
        for (vector, value) in plane.basis_mut().iter_mut().zip(basis.iter()) {
            *vector = json_array_to_vector(value.as_array()?.as_slice())?;
        }

        let bounds_array = json
            .get("bounds")?
            .as_array()
            .filter(|l| l.len() == D - 1)?;
        let mut bounds = [0.; D];
        for (vector, value) in bounds.iter_mut().zip(bounds_array.iter()) {
            *vector = value.as_f64()? as f32;
        }

        Some(Self { plane, bounds })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_with_0(mut vec: Vec<f32>) -> Vec<f32> {
        vec.resize(DEFAULT_DIM, 0.0);
        vec
    }

    #[test]
    fn test_2d_horizontal() {
        let mirror = PlaneMirror {
            plane: Plane::new([
                SVector::from_vec(vec![0.0, 0.0]),
                SVector::from_vec(vec![1.0, 0.0]),
            ]),
            bounds: [1.0; 2],
        };
        let ray = Ray {
            origin: Point::from_slice(&[-1.0, 0.0]),
            direction: nalgebra::Unit::new_normalize(SVector::from_vec(vec![1.0, 0.0])),
        };
        let reflections = mirror.reflect(&ray);
        assert_eq!(reflections.len(), 1);
        assert_eq!(reflections[0].1, Plane::new([
            SVector::from_vec(vec![0.0, 1.0]),
            SVector::from_vec(vec![0.0, -1.0]),
        ]));
    }

    #[test]
    fn test_2d_vertical() {
        let mirror = PlaneMirror {
            plane: Plane::new([
                SVector::from_vec(vec![1.0, 0.0]),
                SVector::from_vec(vec![0.0, 1.0]),
            ]),
            bounds: [1.0; 2],
        };
        let ray = Ray {
            origin: Point::from_slice(&[0.2, 0.2]),
            direction: nalgebra::Unit::new_normalize(SVector::from_vec(vec![1.0, 0.0])),
        };
        let reflections = mirror.reflect(&ray);
        assert_eq!(reflections.len(), 1);
        assert_eq!(reflections[0].0, 1.0);
        assert_eq!(reflections[0].1, Plane::new([
            SVector::from_vec(vec![1.0, 0.0]),
            SVector::from_vec(vec![0.0, 1.0]),
        ]));
    }
    #[test]
    fn test_2d_diagonal() {
        let mirror = PlaneMirror {
            plane: Plane::new([
                SVector::from_vec(vec![1.0, 1.0]),
                SVector::from_vec(vec![-1.0, -1.0]),
            ]),
            bounds: [1.0; 2],
        };
        let ray = Ray {
            origin: Point::from_slice(&[1.0, 0.0]),
            direction: nalgebra::Unit::new_normalize(SVector::from_vec(vec![-1.0, 1.0])),
        };
        let reflections = mirror.reflect(&ray);
        assert_eq!(reflections.len(), 1);
        assert_eq!(reflections[0].0, 1.0);
        assert_eq!(reflections[0].1, Plane::new([
            SVector::from_vec(vec![1.0, 1.0]),
            SVector::from_vec(vec![-1.0, -1.0]),
        ]));
    }
}
