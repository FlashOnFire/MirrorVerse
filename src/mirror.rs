use nalgebra::{Point, SVector, Unit};

pub mod bezier;
pub mod cubic_bezier;
pub mod plane;
pub mod sphere;

use crate::DIM;

/// A light ray
pub struct Ray<const D: usize = DIM> {
    /// current position of the ray
    pub origin: Point<f32, D>,
    /// current direction of the ray
    pub direction: Unit<SVector<f32, D>>,
}

/// An up to N-1-dimensional, affine euclidean space
pub struct Plane<const D: usize = DIM> {
    /// the first element of this array is the plane's "starting point" (i. e. v_0)
    /// the remaining N-1 vectors are a set spanning it's associated subspace
    /// Note that an expression like `[T ; N - 1]``
    /// is locked under `#[feature(const_generic_exprs)]`
    vectors: [SVector<f32, D>; D],
}

impl<const D: usize> Plane<D> {
    /// the plane's starting point
    fn v_0(&self) -> &SVector<f32, D> {
        self.vectors.first().unwrap()
    }
    /// a reference to the stored basis of the plane's associated hyperplane
    fn spanning_set(&self) -> &[SVector<f32, D>] {
        &self.vectors[1..]
    }
    /// a mutable reference to the stored basis of the plane's associated hyperplane
    fn spanning_set_mut(&mut self) -> &mut [SVector<f32, D>] {
        &mut self.vectors[1..]
    }
    /// orthonormalize the plane's spanning set, returns a reference to it if
    /// the size of it's largest free family of vectors is exactly N-1
    fn orthonormalize_spanning_set(&mut self) -> Option<&[SVector<f32, D>]> {
        (SVector::orthonormalize(self.spanning_set_mut()) == D - 1).then_some(self.spanning_set())
    }
}

pub trait Mirror<const D: usize = DIM> {
    fn reflect(&self, ray: Ray<D>) -> Option<(f32, Plane<D>)>;
    fn get_type(&self) -> &str;
    fn from_json(json: &serde_json::Value) -> Option<Self>
    where
        Self: Sized;
}

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

struct CompositeMirror<T: Mirror<D>, const D: usize = DIM> {
    mirrors: Vec<T>,
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for CompositeMirror<T, D> {
    fn reflect(&self, ray: Ray<D>) -> Option<(f32, Plane<D>)> {
        // use the other mirror to reflect the ray
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
        let mirrors = json
            .get("mirrors")?
            .as_array()?
            .iter()
            .filter_map(T::from_json)
            .collect();

        Some(Self { mirrors })
    }
}
