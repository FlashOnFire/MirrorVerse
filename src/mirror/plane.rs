use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlaneMirror<const D: usize = DIM> {
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
        let normal = self.plane.basis()[0];
        // TODO test if the ray really touch the plane using bounds

        // the reflection
        //contruct the new hyperplane with the reflected direction as the normal
        let mut return_plane = Plane::new([SVector::zeros(); D]);
        for (i, basis_vector) in self.plane.basis().iter().enumerate() {
            let reflected_basis_vector = basis_vector - 2.0 * basis_vector.dot(&normal) * normal;
            return_plane.basis_mut()[i] = reflected_basis_vector;
        }
        
        vec![(1.0, return_plane)]
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
        vec.resize(DIM, 0.0);
        vec
    }

    #[test]
    fn test_plane_mirror_reflect() {}
}
