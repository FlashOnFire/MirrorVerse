use core::iter;
use nalgebra::{ArrayStorage, Point, SMatrix, SVector, Unit};
use serde_json::Value;
use std::fmt;

pub mod bezier;
pub mod cubic_bezier;
pub mod plane;
pub mod sphere;

use crate::DEFAULT_DIM;

/// A light ray
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray<const D: usize = DEFAULT_DIM> {
    /// Current position of the ray
    pub origin: SVector<f32, D>,
    /// Current direction of the ray
    pub direction: Unit<SVector<f32, D>>,
    /// Current brightness of the ray (0.0 to 1.0)
    pub brightness: f32,
}

impl<const D: usize> Ray<D> {
    /// Create a new ray with a given origin and direction
    pub fn from_json(json: &Value) -> Result<Self, Box<dyn std::error::Error>> {
        /*
        example json:
                 {
            "origin": [9., 8., 7., ...], (N elements)
            "direction": [9., 8., 7., ...], (N elements)
            "brightness": 0.5
        }
        */
        let origin = json
            .get("origin")
            .and_then(Value::as_array)
            .ok_or_else(|| Box::<dyn std::error::Error>::from("Missing ray origin"))?;
        let direction = json
            .get("direction")
            .and_then(Value::as_array)
            .ok_or_else(|| Box::<dyn std::error::Error>::from("Missing ray direction"))?;
        let brightness = json
            .get("brightness")
            .ok_or_else(|| Box::<dyn std::error::Error>::from("Missing ray brightness"))?;

        let origin = json_array_to_vector::<D>(origin)
            .ok_or_else(|| Box::<dyn std::error::Error>::from("Invalid ray origin"))?;

        let direction = json_array_to_vector::<D>(direction)
            .ok_or_else(|| Box::<dyn std::error::Error>::from("Invalid ray direction"))?;

        let direction = Unit::try_new(direction, 1e-6).ok_or_else(|| {
            Box::<dyn std::error::Error>::from("Unable to normalize ray direction")
        })?;

        let brightness = brightness.as_f64().ok_or_else(|| {
            Box::<dyn std::error::Error>::from("Invalid ray brightness (not a number)")
        })? as f32;

        Ok(Self {
            origin,
            direction,
            brightness,
        })
    }
}

/// An up to N-1-dimensional, euclidean affine subspace
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane<const D: usize = DEFAULT_DIM> {
    /// The first element of this array is the plane's "starting point" (i. e. v_0).
    /// The remaining N-1 vectors are a family spanning it's associated subspace.
    ///
    /// Note that an expression like `[T ; N - 1]`
    /// is locked under `#[feature(const_generic_exprs)]`
    vectors: [SVector<f32, D>; D],
}

// Important Note: this implementation is only valid for D >= 1.
// but it is impossible to write something akin to `where D >= 1`
// to statically restrict the value without `#[feature(const_generic_exprs)]`
impl<const D: usize> Plane<D> {
    /// `vectors` must respect the layout/specification of the `vectors` field
    pub fn new(mut vectors: [SVector<f32, D>; D]) -> Option<Self> {
        (SVector::orthonormalize(&mut vectors[1..]) == D - 1).then_some(Self { vectors })
    }
    /// The plane's starting point
    pub fn v_0(&self) -> &SVector<f32, D> {
        self.vectors.first().unwrap()
    }
    /// A reference to the stored basis of the plane's associated hyperplane.
    ///
    /// The returned slice is garanteed to be of length D - 1.
    pub fn basis(&self) -> &[SVector<f32, D>] {
        &self.vectors[1..]
    }
    /// Project a vector using the orthonormal basis projection formula.
    ///
    /// Assumes `b` is an orthonormal (thus, free) family. If such isn't
    /// the case, the result is unspecified.
    pub fn orthogonal_projection(&self, v: SVector<f32, D>) -> SVector<f32, D> {
        self.basis().iter().map(|e| v.dot(e) * e).sum()
    }
}

pub trait Mirror<const D: usize = DEFAULT_DIM> {
    /// Returns a set of brightness gains and planes, in no particular order.
    ///
    /// The laser is expected to "bounce" off the closest plane.
    ///
    /// Here, "bounce" refers to the process of:
    ///     - Moving forward until it intersects the plane
    ///     - Adjusting it's brightness according to the provided gain value
    ///     - Then, orthognoally reflecting it's direction vector with
    ///       respect to the subspace defining the plane's "orientation"
    ///
    /// Returns an empty list if the vector doesn't intersect with the mirror.
    fn intersecting_planes(&self, ray: &Ray<D>) -> Vec<(f32, Plane<D>)>;
    /// An optimised version of `Self::reflect` that potentially saves
    /// an allocation by writing into another `Vec`. Override this if needed.
    ///
    /// It is a logic error for this function to remove/reorder elements in `list`
    fn append_intersecting_planes(&self, ray: &Ray<D>, list: &mut Vec<(f32, Plane<D>)>) {
        list.append(&mut self.intersecting_planes(ray))
    }
    /// Returns a string slice, unique to the type
    /// (or inner type if type-erased) and coherent with it's json representation
    // TODO: should this be 'static ?
    fn get_type(&self) -> &str;
    /// Deserialises the mirror's data from the provided json string, returns `None` in case of error
    // TODO: use Result and an enum for clearer error handling
    fn from_json(json: &Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

// Note that `T` is implicitly `Sized`
//
// This impl might not be necessary for the time being
impl<const D: usize, T: Mirror<D>> Mirror<D> for Box<T> {
    fn intersecting_planes(&self, ray: &Ray<D>) -> Vec<(f32, Plane<D>)> {
        self.as_ref().intersecting_planes(ray)
    }

    fn get_type(&self) -> &str {
        self.as_ref().get_type()
    }

    fn from_json(json: &Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        T::from_json(json).map(Box::new)
    }
}

impl<const D: usize> Mirror<D> for Box<dyn Mirror<D>> {
    fn intersecting_planes(&self, ray: &Ray<D>) -> Vec<(f32, Plane<D>)> {
        self.as_ref().intersecting_planes(ray)
    }

    fn get_type(&self) -> &str {
        "dynamic"
    }

    fn from_json(json: &Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        /*
        example json
        {
            "type": "....",
            "mirror": <json value whose structure depends on "type">,
        }
         */

        let mirror_type = json
            .get("type")
            .ok_or_else(|| Box::<dyn std::error::Error>::from("Missing mirror type"))?
            .as_str()
            .ok_or_else(|| Box::<dyn std::error::Error>::from("Invalid mirror type"))?;
        let mirror = json
            .get("mirror")
            .ok_or_else(|| Box::<dyn std::error::Error>::from("Missing mirror data"))?;

        

        match mirror_type {
            "plane" => plane::PlaneMirror::<D>::from_json(mirror)
                .map(|mirror| Box::new(mirror) as Box<dyn Mirror<D>>),
            "sphere" => {
                sphere::SphereMirror::<D>::from_json(mirror).map(|mirror| Box::new(mirror) as _)
            }
            _ => Err(Box::<dyn std::error::Error>::from("Invalid mirror type")),
        }
    }
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for Vec<T> {
    fn intersecting_planes(&self, ray: &Ray<D>) -> Vec<(f32, Plane<D>)> {
        let mut list = vec![];
        self.append_intersecting_planes(ray, &mut list);
        list
    }

    fn append_intersecting_planes(&self, ray: &Ray<D>, list: &mut Vec<(f32, Plane<D>)>) {
        self.iter()
            .for_each(|mirror| mirror.append_intersecting_planes(ray, list));
    }

    fn get_type(&self) -> &str {
        "composite"
    }

    fn from_json(json: &Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        /* example json
        [
            ... list of json values whose structure depends on `T`
        ]
         */
        if let Some(json) = json.as_array() {
            let mut mirrors = vec![];
            for mirror in json {
                mirrors.push(T::from_json(mirror)?);
            }
            Ok(mirrors)
        } else {
            Err(Box::new(JsonError {
                message: "Invalid mirror list".to_string(),
            }))
        }
    }
}

/// This is essentially [`Iterator::try_collect`]
/// for `Vec<T>` but without having to use nightly
pub fn try_collect<T>(i: impl Iterator<Item = Option<T>>) -> Option<Vec<T>> {
    let mut vec = vec![];
    for item in i {
        vec.push(item?);
    }

    Some(vec)
}

/// This is essentially `try_into` then `try_map` but the latter is nightly-only
pub fn json_array_to_vector<const D: usize>(
    json_array: &[Value],
) -> Option<SVector<f32, D>> {
    let array: &[Value; D] = json_array.try_into().ok()?;

    let mut center_coords_array = [0.; D];
    for (coord, value) in center_coords_array.iter_mut().zip(array) {
        *coord = value.as_f64()? as f32;
    }
    Some(SVector::from_array_storage(ArrayStorage([
        center_coords_array,
    ])))
}

#[derive(Debug)]
struct JsonError {
    message: String,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for JsonError {}

#[cfg(test)]
mod tests {
    use nalgebra::Vector;

    #[test]
    fn test_json_to_ray() {
        use super::*;
        use serde_json::json;

        let json = json!({
            "origin": [1., 2., 3.],
            "direction": [4., 5., 6.],
            "brightness": 0.5
        });

        let ray = Ray::<3>::from_json(&json).unwrap();
        assert_eq!(ray.origin, SVector::from([1., 2., 3.]));
        assert_eq!(
            ray.direction,
            Unit::new_normalize(SVector::from_vec(vec![4., 5., 6.]))
        );
        assert_eq!(ray.brightness, 0.5);
    }
}
