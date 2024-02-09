use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlaneMirror<const D: usize = DIM> {
    /// The plane this mirror belongs to.
    plane: Plane<D>,
    /// Bounds `(mu_i_min, mu_i_max)` for the scalars in the linear combination of the
    /// basis vectors of the associated hyperplane. 
    /// 
    /// Formally, for all vectors `v = sum mu_i * v_i` of
    /// the hyperplane, `v` is in this plane mirror iff for all `i`, `mu_i_min <= mu_i <= mu_i_max`
    /// 
    /// Note: the first value of this array is irrelevant
    scalar_bounds: [(f32, f32) ; D],
}

impl<const D: usize> Mirror<D> for PlaneMirror<D> {
    fn reflect(&self, ray: &Ray<D>) -> Vec<(f32, Plane<D>)> {
        // TODO: implement plane reflection
        vec![]
    }
    fn get_type(&self) -> &str {
        "plane"
    }

    fn from_json(json: &serde_json::Value) -> Option<Self>
    where
        Self: Sized,
    {
        None
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
