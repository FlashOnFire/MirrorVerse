use core::{iter, ops::Deref};

use super::*;

// pub mod bezier;
// pub mod cubic_bezier;
pub mod cylinder;
pub mod paraboloid;
pub mod plane;
pub mod sphere;

/// A light ray
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray<const D: usize> {
    /// Current position of the ray
    pub origin: SVector<f32, D>,
    /// Current direction of the ray
    pub direction: Unit<SVector<f32, D>>,
}

impl<const D: usize> Ray<D> {
    /// Reflect the ray with respect to the given plane
    pub fn reflect_direction(&mut self, tangent: &Tangent<D>) {
        self.direction = tangent.reflect_unit(self.direction);
    }

    pub fn advance(&mut self, t: f32) {
        self.origin += t * self.direction.into_inner();
    }

    pub fn at(&self, t: f32) -> SVector<f32, D> {
        self.origin + self.direction.into_inner() * t
    }

    /// Create a new ray with a given origin and direction
    pub fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        /*
        example json:
        {
            "origin": [9., 8., 7., ...], (N elements)
            "direction": [9., 8., 7., ...], (N elements)
        }
        */

        let origin = json
            .get("origin")
            .and_then(serde_json::Value::as_array)
            .ok_or("Missing ray origin")?;

        let direction = json
            .get("direction")
            .and_then(serde_json::Value::as_array)
            .ok_or("Missing ray direction")?;

        let origin = util::json_array_to_vector(origin).ok_or("Invalid ray origin")?;

        let direction = util::json_array_to_vector(direction).ok_or("Invalid ray direction")?;

        let direction =
            Unit::try_new(direction, f32::EPSILON).ok_or("Unable to normalize ray direction")?;

        Ok(Self { origin, direction })
    }

    pub fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        Ok(serde_json::json!({
            "origin": self.origin.as_slice(),
            "direction": self.direction.into_inner().as_slice(),
        }))
    }

    pub fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self {
        let origin = util::random_vector(rng, 7.0);

        let direction = loop {
            if let Some(v) = Unit::try_new(util::random_vector(rng, 1.0), f32::EPSILON * 8.0) {
                break v;
            }
        };
        Self { origin, direction }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tangent<const D: usize> {
    Plane(Plane<D>),
    Normal {
        origin: SVector<f32, D>,
        normal: Unit<SVector<f32, D>>,
    },
}

impl<const D: usize> Tangent<D> {
    pub fn reflect_unit(&self, vector: Unit<SVector<f32, D>>) -> Unit<SVector<f32, D>> {
        // SAFETY: orthogonal symmetry preserves euclidean norms
        // This function is supposed to be unsafe, why nalgebra? why?
        Unit::new_unchecked(self.reflect(vector.into_inner()))
    }

    pub fn reflect(&self, vector: SVector<f32, D>) -> SVector<f32, D> {
        match self {
            Tangent::Plane(plane) => 2.0 * plane.orthogonal_projection(vector) - vector,
            Tangent::Normal { normal, .. } => {
                let n = normal.as_ref();
                vector - 2.0 * vector.dot(n) * n
            }
        }
    }

    pub fn try_intersection_distance(&self, ray: &Ray<D>) -> Option<f32> {
        match self {
            Tangent::Plane(plane) => plane.intersection_coordinates(ray).map(|v| v[0]),
            Tangent::Normal { origin, normal } => {
                let u = ray.direction.dot(normal);
                (u.abs() > f32::EPSILON).then(|| (origin - ray.origin).dot(normal) / u)
            }
        }
    }
}

/// An affine hyperplane
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane<const D: usize> {
    /// The first element of this array is the plane's "starting point" (i. e. v_0).
    /// The remaining N-1 vectors are an orthonormal family spanning it's associated subspace.
    ///
    /// Note that an expression like `[T ; N - 1]`
    /// is locked under `#[feature(const_generic_exprs)]`
    vectors: [SVector<f32, D>; D],
    /// A cache containing an orthonormalized version of the family in the `vectors`
    /// field, to facilitate orthogonal projection
    /// TODO: not all planes are used for projections, seperate this.
    orthonormalized: [SVector<f32, D>; D],
}

impl<const D: usize> Plane<D> {
    /// `vectors` must respect the layout/specification of the `vectors` field
    /// returns None if the provided family isn't free
    pub fn new(vectors: [SVector<f32, D>; D]) -> Option<Self> {
        let mut orthonormalized = vectors;
        (SVector::orthonormalize(&mut orthonormalized[1..]) == D - 1).then_some(Self {
            vectors,
            orthonormalized,
        })
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

    fn orthonormalized_basis(&self) -> &[SVector<f32, D>] {
        &self.orthonormalized[1..]
    }

    /// Project a vector using the orthonormal basis projection formula.
    pub fn orthogonal_projection(&self, v: SVector<f32, D>) -> SVector<f32, D> {
        self.orthonormalized_basis()
            .iter()
            .map(|e| v.dot(e) * e)
            .sum()
    }

    /// Project a point onto the plane
    pub fn orthogonal_point_projection(&self, point: SVector<f32, D>) -> SVector<f32, D> {
        let v = point - self.v_0();
        self.v_0() + self.orthogonal_projection(v)
    }

    /// Returns a vector `[t_1, ..., t_d]` whose coordinates represent
    /// the `intersection` of the given `ray` and `self`.
    ///
    /// If it exists, the following holds:
    ///
    /// `intersection = ray.origin + t_1 * ray.direction` and,
    ///
    /// let `[v_2, ..., v_d]` be the basis of `self`'s associated hyperplane
    ///
    /// `interserction = plane.origin + sum for k in [2 ; n] t_k * v_k`
    pub fn intersection_coordinates(&self, ray: &Ray<D>) -> Option<SVector<f32, D>> {
        let mut a = SMatrix::<f32, D, D>::zeros();

        a.column_iter_mut()
            .zip(iter::once(ray.direction.as_ref()).chain(self.basis().iter()))
            .for_each(|(mut i, o)| i.set_column(0, o));

        a.try_inverse_mut()
            // a now contains a^-1
            .then(|| {
                let mut v = a * (ray.origin - self.v_0());
                let first = &mut v[0];
                *first = -*first;
                v
            })
    }

    /// create a new plane from a json value
    pub fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        /*
        example json:
        {
            "center": [9., 8., 7., ...], (N elements)
            "basis": [
                [9., 8., 7., ...], (N elements)
                [9., 8., 7., ...],
                ...
            ] (N-1 elements)
        }
        */

        let mut vectors = [SVector::zeros(); D];

        let (v_0, basis) = vectors.split_first_mut().unwrap();

        *v_0 = json
            .get("center")
            .and_then(serde_json::Value::as_array)
            .map(Vec::as_slice)
            .and_then(util::json_array_to_vector)
            .ok_or("Failed to parse center")?;

        let basis_json = json
            .get("basis")
            .and_then(serde_json::Value::as_array)
            .filter(|l| l.len() == D - 1)
            .ok_or("Failed to parse basis")?;

        for (value, vector) in basis_json.iter().zip(basis) {
            *vector = value
                .as_array()
                .map(Vec::as_slice)
                .and_then(util::json_array_to_vector)
                .ok_or("Failed to parse basis vector")?;
        }

        Ok(Plane::new(vectors).ok_or("the provided family of vectors must be free")?)
    }

    fn to_json(self) -> Result<serde_json::Value, Box<dyn Error>> {
        let center: Vec<f32> = self.v_0().iter().copied().collect();

        let basis: Vec<Vec<f32>> = self
            .basis()
            .iter()
            .map(|v| v.iter().copied().collect())
            .collect();

        Ok(serde_json::json!({
            "center": center,
            "basis": basis,
        }))
    }
}

pub trait Mirror<const D: usize> {
    /// Appends to the list a number of tangent planes, in no particular order.
    ///
    /// The laser is expected to "bounce" off the closest plane.
    ///
    /// Here, "bounce" refers to the process of:
    ///     - Moving forward until it intersects the plane
    ///     - Then, orthogonally reflecting it's direction vector with
    ///       respect to the directing hyperplane
    ///
    /// Appends nothing if the ray doesn't intersect with the mirror that `self` represents
    ///
    /// This method may push intersection points that occur "behind" the ray's
    /// origin, (`ray.at(t)` where `t < 0.0`) simulations must discard these accordingly
    ///
    /// It is a logic error for this function to remove/reorder elements in `list`
    /// TODO: pass in a wrapper around a &mut Vec<_> that
    /// only allows pushing/appending/extending etc..
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<Tangent<D>>);
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for [T] {
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<Tangent<D>>) {
        self.iter()
            .for_each(|mirror| mirror.append_intersecting_points(ray, list))
    }
}

impl<const D: usize, T: Deref> Mirror<D> for T
where
    T::Target: Mirror<D>,
{
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<Tangent<D>>) {
        self.deref().append_intersecting_points(ray, list)
    }
}

pub trait JsonType {
    /// Returns a string slice, unique to the type, found in the "type" field of the json
    /// representation of a "dynamic" mirror containing a mirror of this type
    fn json_type() -> String;
}

impl<T: JsonType> JsonType for [T] {
    fn json_type() -> String {
        format!("[]{}", T::json_type())
    }
}

impl<T: Deref> JsonType for T
where
    T::Target: JsonType,
{
    fn json_type() -> String {
        T::Target::json_type()
    }
}

pub trait JsonSer {
    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>>;
}

impl<T: JsonSer> JsonSer for [T] {
    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let array = util::try_collect(self.iter().map(T::to_json).map(Result::ok))
            .ok_or("falied to deserialize a mirror")?;
        Ok(serde_json::json!(array))
    }
}

impl<T: Deref> JsonSer for T
where
    T::Target: JsonSer,
{
    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        self.deref().to_json()
    }
}

pub trait JsonDes {
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

impl<T: JsonDes> JsonDes for Vec<T> {
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        json.as_array()
            .ok_or("json value must be an array to deserialise an array of mirrors")?
            .iter()
            .map(T::from_json)
            .collect()
    }
}

pub trait Random {
    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized;
}
