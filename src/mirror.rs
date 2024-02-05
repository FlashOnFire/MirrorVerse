use nalgebra::{Point, SVector, Unit};

pub mod bezier;
pub mod cubic_bezier;
pub mod plane;
pub mod sphere;

use crate::DIM;

/// A light ray
pub struct Ray<const D: usize = DIM> {
    /// Current position of the ray
    pub origin: Point<f32, D>,
    /// Current direction of the ray
    pub direction: Unit<SVector<f32, D>>,
}

/// An up to N-1-dimensional, euclidean affine subspace
pub struct Plane<const D: usize = DIM> {
    /// The first element of this array is the plane's "starting point" (i. e. v_0).
    /// The remaining N-1 vectors are a family spanning it's associated subspace.
    ///
    /// Note that an expression like `[T ; N - 1]`
    /// is locked under `#[feature(const_generic_exprs)]`
    vectors: [SVector<f32, D>; D],
}

// Important Note: this implementation is only valid of D >= 2.
impl<const D: usize> Plane<D> {
    /// The plane's starting point
    pub fn v_0(&self) -> &SVector<f32, D> {
        self.vectors.first().unwrap()
    }
    /// A mutable reference to the plane's starting point
    pub fn v_0_mut(&mut self) -> &mut SVector<f32, D> {
        self.vectors.first_mut().unwrap()
    }
    /// A reference to the stored basis of the plane's associated hyperplane
    pub fn spanning_set(&self) -> &[SVector<f32, D>] {
        &self.vectors[1..]
    }
    /// A mutable reference to the stored basis of the plane's associated hyperplane
    pub fn spanning_set_mut(&mut self) -> &mut [SVector<f32, D>] {
        &mut self.vectors[1..]
    }
    /// Orthonormalize the plane's spanning set, Returns
    /// a reference to it's (orthonormalised) largest free family
    pub fn orthonormalize_spanning_set(&mut self) -> &[SVector<f32, D>] {
        let n = SVector::orthonormalize(self.spanning_set_mut());
        &self.spanning_set()[..n]
    }
    /// Project a vector using the orthonormal basis projection formula.
    ///
    /// Assumes `b` is an orthonormal family. If such isn't
    /// the case, the result is unspecified.
    pub fn orthogonal_projection(v: SVector<f32, D>, b: &[SVector<f32, D>]) -> SVector<f32, D> {
        b.iter().map(|e| v.dot(e) * e).sum()
    }
}

pub trait Mirror<const D: usize = DIM> {
    /// Returns a brightness gain and a plane
    /// the laser is expected to:
    ///     - move forward until it intersects the plane
    ///     - adjust it's brightness according to the provided gain value
    ///     - orthognoally reflect it's direction vector with
    ///       repect to the plane's hyperplane/subspace
    ///
    /// Returns None if the laser doesn't interact with the mirror
    fn reflect(&self, ray: Ray<D>) -> Option<(f32, Plane<D>)>;
    /// Returns a string slice, unique to the type
    /// (or inner type if type-erased) and coherent with it's json representation
    // TODO: should this be 'static ?
    fn get_type(&self) -> &str;
    /// Deserialises the mirror's data from the provided json string, returns None in case of error
    // TODO: use Result
    fn from_json(json: &serde_json::Value) -> Option<Self>
    where
        Self: Sized;
}

// Surprisingly doesn't break the orphan rules, because `Box`` is `#[fundamental]``
//
// Note that `T`` is implicitly `Sized``
//
// This impl might not be necessary for the time being
impl<const D: usize, T: Mirror<D>> Mirror<D> for Box<T> {
    fn reflect(&self, ray: Ray<D>) -> Option<(f32, Plane<D>)> {
        self.as_ref().reflect(ray)
    }

    fn get_type(&self) -> &str {
        self.as_ref().get_type()
    }

    fn from_json(json: &serde_json::Value) -> Option<Self>
    where
        Self: Sized,
    {
        T::from_json(json).map(Box::new)
    }
}

impl<const D: usize> Mirror<D> for Box<dyn Mirror<D>> {
    fn reflect(&self, ray: Ray<D>) -> Option<(f32, Plane<D>)> {
        self.as_ref().reflect(ray)
    }

    fn get_type(&self) -> &str {
        self.as_ref().get_type()
    }

    fn from_json(json: &serde_json::Value) -> Option<Self>
    where
        Self: Sized,
    {
        let mirror_type = json.get("type")?.as_str()?;

        match mirror_type {
            "plane" => plane::PlaneMirror::<D>::from_json(json)
                .map(|mirror| Box::new(mirror) as Box<dyn Mirror<D>>),
            "sphere" => {
                sphere::SphereMirror::<D>::from_json(json).map(|mirror| Box::new(mirror) as _)
            }
            _ => None,
        }
    }
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for Vec<T> {
    fn reflect(&self, ray: Ray<D>) -> Option<(f32, Plane<D>)> {
        None
    }

    fn get_type(&self) -> &str {
        "composite"
    }

    fn from_json(json: &serde_json::Value) -> Option<Self>
    where
        Self: Sized,
    {
        /* example json
        {
            "mirrors": [
                {
                    "type": "plane",
                    "points": [
                        [1.0, 2.0, 3.0, ...],
                        [4.0, 5.0, 6.0, ...],
                        [7.0, 8.0, 9.0, ...],
                        ...
                    ]
                },
                {
                    "type": "sphere",
                    "center": [1.0, 2.0, 3.0],
                    "radius": 4.0
                },
                ...
            ]
        }
         */

        // TODO: return a Result with clearer errors

        // fail if the deserialisation of _one_ mirror fails
        Some(
            json.get("mirrors")?
                .as_array()?
                .iter()
                .filter_map(T::from_json)
                .collect(),
        )
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
    fn test_composite_mirror_from_json() {
        let json = serde_json::json!({
            "mirrors": [
                {
                    "type": "plane",
                    "points": [
                        complete_with_0(vec![1.0, 2.0]),
                        complete_with_0(vec![3.0, 4.0]),
                    ]
                },
                {
                    "type": "sphere",
                    "center": complete_with_0(vec![5.0, 6.0]),
                    "radius": 7.0
                },
            ]
        });

        let mirrors =
            Vec::<Box<dyn Mirror<DIM>>>::from_json(&json).expect("json deserialisation failed");

        assert_eq!(mirrors.len(), 2);
        //check the first is a plane mirror
        assert_eq!(mirrors[0].get_type(), "plane");
        assert_eq!(mirrors[1].get_type(), "sphere");
    }
}
