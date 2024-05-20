use core::ops::Deref;

use super::*;

// pub mod bezier;
// pub mod cubic_bezier;
// pub mod paraboloid;
pub mod cylinder;
pub mod plane;
pub mod sphere;

use util::List;

/// A light ray, represented as a half-line of starting point `origin` and direction `direction`
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray<const D: usize> {
    /// Current position of the ray
    pub origin: SVector<Float, D>,
    /// Current direction of the ray
    pub direction: Unit<SVector<Float, D>>,
}

impl<const D: usize> Ray<D> {
    /// Reflect the ray's direction with respect to the given tangent's directing (hyper)plane
    pub fn reflect_direction(&mut self, tangent: &Tangent<D>) {
        self.direction = tangent.reflect_unit(self.direction);
    }

    /// Move the ray's position forward (or backward if t < 0.0) by `t`
    pub fn advance(&mut self, t: Float) {
        self.origin += t * self.direction.into_inner();
    }

    /// Get the point at distance `t` (can be negative) from the ray's origin
    pub fn at(&self, t: Float) -> SVector<Float, D> {
        self.origin + self.direction.into_inner() * t
    }

    /// Deserialize a new ray from a JSON object.
    ///
    /// The JSON object must follow the following format:
    ///
    /// ```ignore
    /// {
    ///     "origin": [9., 8., 7., ...], (an array of D floats)
    ///     "direction": [9., 8., 7., ...], (an array of D floats, must have at least one non-zero value)
    /// }
    /// ```
    pub fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
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
            Unit::try_new(direction, Float::EPSILON).ok_or("Unable to normalize ray direction")?;

        Ok(Self { origin, direction })
    }

    /// Serialize a ray into a JSON object.
    ///
    /// The format of the returned object is explained in [`Self::from_json`]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "origin": self.origin.as_slice(),
            "direction": self.direction.into_inner().as_slice(),
        })
    }

    pub fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self {
        let origin = util::random_vector(rng, 7.0);

        let direction = loop {
            if let Some(v) = Unit::try_new(util::random_vector(rng, 1.0), Float::EPSILON * 8.0) {
                break v;
            }
        };
        Self { origin, direction }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Different ways of representing an affine hyperplane in D-dimensional euclidean space
pub enum Tangent<const D: usize> {
    /// Stored as a starting point v_0 and a basis of the direction hyperlane
    Plane(Plane<D>),
    ///
    Normal {
        origin: SVector<Float, D>,
        normal: Unit<SVector<Float, D>>,
    },
}

impl<const D: usize> Tangent<D> {
    /// Reflect a unit vector with respect to this tangent plane's direction space
    pub fn reflect_unit(&self, vector: Unit<SVector<Float, D>>) -> Unit<SVector<Float, D>> {
        // SAFETY: orthogonal symmetry preserves euclidean norms
        // This function is supposed to be unsafe, why nalgebra? why?
        Unit::new_unchecked(self.reflect(vector.into_inner()))
    }

    /// Reflect a vector with respect to this tangent plane's direction space
    pub fn reflect(&self, vector: SVector<Float, D>) -> SVector<Float, D> {
        match self {
            Tangent::Plane(plane) => 2.0 * plane.orthogonal_projection(vector) - vector,
            Tangent::Normal { normal, .. } => {
                let n = normal.as_ref();
                vector - 2.0 * vector.dot(n) * n
            }
        }
    }

    /// Return the distance `t` such that `ray.at(t)` intersects with this tangent plane
    ///
    /// Returns `None` if `ray` is parallel to `self`
    pub fn try_intersection_distance(&self, ray: &Ray<D>) -> Option<Float> {
        match self {
            Tangent::Plane(plane) => plane.intersection_coordinates(ray).map(|v| v[0]),
            Tangent::Normal { origin, normal } => {
                let u = ray.direction.dot(normal);
                (u.abs() > Float::EPSILON).then(|| (origin - ray.origin).dot(normal) / u)
            }
        }
    }
}

/// An affine hyperplane
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane<const D: usize> {
    /// The first element of this array is the plane's "starting point" (i. e. v_0).
    /// The remaining N-1 vectors are an orthonormal family spanning it's direction hyperplane
    ///
    /// Note that an expression like `[T ; N - 1]`
    /// is locked under `#[feature(const_generic_exprs)]`
    vectors: [SVector<Float, D>; D],
    /// A cache containing an orthonormalized version of the family in the `vectors`
    /// field, to facilitate orthogonal projection
    /// TODO: not all planes are used for projections, seperate this.
    orthonormalized: [SVector<Float, D>; D],
}

impl<const D: usize> Plane<D> {
    /// `vectors` must respect the layout/specification of the `vectors` field
    /// returns None if the provided family isn't free
    pub fn new(vectors: [SVector<Float, D>; D]) -> Option<Self> {
        let mut orthonormalized = vectors;
        (SVector::orthonormalize(&mut orthonormalized[1..]) == D - 1).then_some(Self {
            vectors,
            orthonormalized,
        })
    }

    /// The plane's starting point
    pub fn v_0(&self) -> &SVector<Float, D> {
        self.vectors.first().unwrap()
    }

    /// Change this plane's starting vector
    pub fn set_v_0(&mut self, v: SVector<Float, D>) {
        self.vectors[0] = v;
        self.orthonormalized[0] = v;
    }

    /// A reference to the basis of the plane's direction space.
    ///
    /// The returned slice is garanteed to be of length D - 1.
    pub fn basis(&self) -> &[SVector<Float, D>] {
        &self.vectors[1..]
    }

    fn orthonormalized_basis(&self) -> &[SVector<Float, D>] {
        &self.orthonormalized[1..]
    }

    /// Project a vector using the orthonormal basis projection formula.
    pub fn orthogonal_projection(&self, v: SVector<Float, D>) -> SVector<Float, D> {
        self.orthonormalized_basis()
            .iter()
            .map(|e| v.dot(e) * e)
            .sum()
    }

    /// Project a point onto the plane
    pub fn orthogonal_point_projection(&self, p: SVector<Float, D>) -> SVector<Float, D> {
        let v0 = self.v_0();
        let v = p - v0;
        v0 + self.orthogonal_projection(v)
    }

    /// Returns a vector `[t_1, ..., t_d]` whose coordinates represent
    /// the `intersection` of the given `ray` and `self`.
    ///
    /// If it exists, the following holds:
    ///
    /// `intersection = ray.origin + t_1 * ray.direction` and,
    ///
    /// let `[v_2, ..., v_d]` be the basis of `self`'s direction space
    ///
    /// `interserction = plane.origin + sum for k in [2 ; n] t_k * v_k`
    pub fn intersection_coordinates(&self, ray: &Ray<D>) -> Option<SVector<Float, D>> {
        let mut a = SMatrix::<Float, D, D>::from_columns(&self.vectors);
        a.set_column(0, ray.direction.as_ref());

        a.try_inverse_mut()
            // a now contains a^-1
            .then(|| {
                let mut v = a * (ray.origin - self.v_0());
                let first = &mut v[0];
                *first = -*first;
                v
            })
    }

    /// Deserialize a new plane from a JSON object.
    ///
    /// The JSON object must follow the following format:
    ///
    /// ```ignore
    /// {
    ///     "center": [9., 8., 7., ...], (N elements)
    ///     "basis": [
    ///         [9., 8., 7., ...], (N elements)
    ///         [9., 8., 7., ...],
    ///         ...
    ///     ] (N-1 elements, must be a free family of vectors)
    /// }
    /// ```
    pub fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
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

    /// Serialize a ray into a JSON object.
    ///
    /// The format of the returned object is explained in [`Self::from_json`]
    fn to_json(self) -> serde_json::Value {
        let slices = self.vectors.each_ref().map(SVector::as_slice);
        let (center, basis) = slices.split_first().unwrap();

        serde_json::json!({
            "center": center,
            "basis": basis,
        })
    }
}

// D could have been an associated constant but, lack of
// `#[feature(generic_const_exprs)]` screws us over, once again.
pub trait Mirror<const D: usize> {
    /// Appends to `list` a number of affine (hyper)planes, tangent to this mirror, in no particular order.
    ///
    /// `ray` is expected to "bounce" off the closest plane.
    ///
    /// Here, "bounce" refers to the process of:
    ///     - Moving forward until it intersects the plane.
    ///     - Then, orthogonally reflecting it's direction vector with
    ///       respect to the direction hyperplane.
    ///
    /// Appends nothing if the ray doesn't intersect with the mirror that `self` represents.
    ///
    /// This method may push intersection points that occur "behind" the ray's
    /// origin, (`ray.at(t)` where `t < 0.0`) simulations must discard these accordingly.
    /// 
    /// This method is deterministic, i. e. not random: for some `ray`, it always has the same behavior for that `ray`.
    fn append_intersecting_points(&self, ray: &Ray<D>, list: List<Tangent<D>>);
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for [T] {
    fn append_intersecting_points(&self, ray: &Ray<D>, mut list: List<Tangent<D>>) {
        self.iter().for_each(|mirror| mirror.append_intersecting_points(ray, list.reborrow()))
    }
}

impl<const D: usize, T: Deref> Mirror<D> for T
where
    T::Target: Mirror<D>,
{
    fn append_intersecting_points(&self, ray: &Ray<D>, list: List<Tangent<D>>) {
        self.deref().append_intersecting_points(ray, list)
    }
}

pub trait JsonType {
    /// Returns a string, unique to the type, found in the "type" field of the json
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
    /// Serialize `self` into a JSON object.
    fn to_json(&self) -> serde_json::Value;
}

impl<T: JsonSer> JsonSer for [T] {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!(Vec::from_iter(self.iter().map(T::to_json)))
    }
}

impl<T: Deref> JsonSer for T
where
    T::Target: JsonSer,
{
    fn to_json(&self) -> serde_json::Value {
        self.deref().to_json()
    }
}

pub trait JsonDes {
    /// Deserialize from a JSON object.
    ///
    /// Returns an error if `json`'s format or values are invalid.
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

impl<T: JsonDes> JsonDes for Vec<T> {
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        util::map_json_array(json, T::from_json)
    }
}

pub trait Random {
    /// Generate a randomized version of this mirror using the provided `rng`
    ///
    /// This method must not fail. If creating a mirror is faillible, keep trying until success
    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized;
}
