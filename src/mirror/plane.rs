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
        /* example json
        {
            "points": [
                [1.0, 2.0, 3.0, ...],
                [4.0, 5.0, 6.0, ...],
                [7.0, 8.0, 9.0, ...],
                ...
            ]
        }
         */

        // TODO: optimize out the allocations
        // TODO: return a Result with clearer errors

        let points: [_; D] = json
            .get("points")?
            .as_array()?
            .iter()
            .filter_map(|point| {
                let point: [_; D] = point
                    .as_array()?
                    .iter()
                    .filter_map(serde_json::Value::as_f64)
                    .map(|val| val as f32)
                    .collect::<Vec<_>>()
                    .try_into()
                    .ok()?;

                Some(Point::from_slice(&point))
            })
            .collect::<Vec<_>>()
            .try_into()
            .ok()?;

        Some(Self { points })
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
    fn test_plane_mirror_from_json() {
        let json = serde_json::json!({
            "points": [
                complete_with_0(vec![1.0, 2.0]),
                complete_with_0(vec![3.0, 4.0]),
            ]
        });

        let mirror = PlaneMirror::from_json(&json).expect("json deserialisation failed");

        assert_eq!(
            mirror.points[0],
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![1.0, 2.0]))
        );
        assert_eq!(
            mirror.points[1],
            Point::<f32, DIM>::from_slice(&complete_with_0(vec![3.0, 4.0]))
        );
    }

    #[test]
    fn test_plane_mirror_reflect() {}
}
