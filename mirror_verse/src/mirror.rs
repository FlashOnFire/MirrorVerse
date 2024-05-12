use core::{iter, ops::Deref};

use super::*;

use format as f;

pub mod bezier;
pub mod cubic_bezier;
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
        let origin = util::random_vector(rng, 24.0);

        let direction = loop {
            if let Some(v) = Unit::try_new(util::random_vector(rng, 4.0), f32::EPSILON * 8.0) {
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

        /* bien vuu le boss
        Fill the matrix "a" with the direction of the ray and the basis of the plane
        exemple
        | -ray_direction.x | plane_basis_1.x | plane_basis_2.x | ...
        | -ray_direction.y | plane_basis_1.y | plane_basis_2.y | ...
        | -ray_direction.z | plane_basis_1.z | plane_basis_2.z | ...
        */

        a.column_iter_mut()
            .zip(iter::once((-ray.direction).as_ref()).chain(self.basis().iter()))
            .for_each(|(mut i, o)| i.set_column(0, o));

        a.try_inverse_mut()
            // a now contains a^-1
            .then(|| a * (ray.origin - self.v_0()))
    }

    /// create a new plane from a json value
    pub fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {
        /*
        example json:
        {
            "v_0": [9., 8., 7., ...], (N elements)
            "basis": [[9., 8., 7., ...], [9., 8., 7., ...], ...], (N-1 elements)
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

        Ok(Plane::new(vectors).ok_or("Failed to create plane")?)
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
    /// It is ok for this method to push intersection points that occur "behind" the ray's
    /// origin, (`ray.at(t)` where `t < 0.0`) the engine will discard these accordingly
    ///
    /// It is a logic error for this function to remove/reorder elements in `list`
    /// TODO: pass in a wrapper around a &mut Vec<_> that
    /// only allows pushing/appending/extending etc..
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<Tangent<D>>);
    /// Returns a string slice, unique to the type, coherent with it's json representation
    fn get_json_type() -> String
    where
        Self: Sized;
    /// except for trait objects, this should behave the same way as `Mirror::get_json_type`
    fn get_json_type_inner(&self) -> String;
    /// Deserialises data from the provided json string, returns `None` in case of error
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
    /// Returns a json representation of the data
    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>>;
    /// Returns a list of vertices and index primitves used to render the mirror
    /// TODO: pass in a list and push to that
    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>>;
    /// Returns a random new instance of the mirror
    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized;
}

impl<const D: usize> Mirror<D> for Box<dyn Mirror<D>> {
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<Tangent<D>>) {
        self.as_ref().append_intersecting_points(ray, list);
    }

    fn get_json_type() -> String
    where
        Self: Sized,
    {
        "dynamic".into()
    }

    fn get_json_type_inner(&self) -> String {
        self.as_ref().get_json_type_inner()
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>>
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
            .ok_or("Missing mirror type")?
            .as_str()
            .ok_or("type must be a string")?;

        let mirror = json.get("mirror").ok_or("Missing mirror data")?;

        fn into_type_erased<const D: usize, T: Mirror<D> + 'static>(
            mirror: T,
        ) -> Box<dyn Mirror<D>> {
            Box::new(mirror) as _
        }

        match mirror_type {
            "plane" => plane::PlaneMirror::<D>::from_json(mirror).map(into_type_erased),
            "sphere" => sphere::EuclideanSphereMirror::<D>::from_json(mirror).map(into_type_erased),
            "dynamic" => Box::<dyn Mirror<D>>::from_json(mirror).map(into_type_erased),
            other => {
                // flatten nested lists
                if let Some(inner) = {
                    let inner = other.trim_start_matches("[]");
                    (other != inner).then_some(inner)
                } {
                    match inner {
                        "plane" => {
                            Vec::<plane::PlaneMirror<D>>::from_json(mirror).map(into_type_erased)
                        }
                        "sphere" => Vec::<sphere::EuclideanSphereMirror<D>>::from_json(mirror)
                            .map(into_type_erased),
                        "dynamic" => {
                            Vec::<Box<dyn Mirror<D>>>::from_json(mirror).map(into_type_erased)
                        }
                        _ => Err(f!("invalid mirror type :{other}").into()),
                    }
                } else {
                    Err(f!("invalid mirror type :{other}").into())
                }
            }
        }
    }

    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let this = self.as_ref();
        Ok(serde_json::json!({
            "type": this.get_json_type_inner(),
            "mirror": this.to_json()?,
        }))
    }

    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        self.as_ref().render_data(display)
    }

    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized,
    {
        let mirror_types = ["plane", "sphere"];

        match rng.gen_range(0..mirror_types.len()) {
            0 => Box::new(plane::PlaneMirror::<D>::random(rng)),
            1 => Box::new(sphere::EuclideanSphereMirror::<D>::random(rng)),
            _ => unreachable!(),
        }
    }
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for Vec<T> {
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<Tangent<D>>) {
        self.as_slice()
            .iter()
            .for_each(|mirror| mirror.append_intersecting_points(ray, list));
    }

    fn get_json_type() -> String {
        format!("[]{}", T::get_json_type())
    }

    fn get_json_type_inner(&self) -> String {
        format!("[]{}", T::get_json_type())
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        /* example json
        [
            ... (potentially nested) list of json values whose structure depends on `T`
        ]
        */

        fn append_to_array<const D: usize, T: Mirror<D>>(
            json: &serde_json::Value,
            list: &mut Vec<T>,
        ) -> Result<(), Box<dyn Error>> {
            if let Some(array) = json.as_array() {
                for inner_json in array {
                    append_to_array(inner_json, list)?
                }
            } else {
                list.push(T::from_json(json)?)
            }
            Ok(())
        }

        let mut list = vec![];
        append_to_array(json, &mut list)?;
        Ok(list)
    }

    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let array = util::try_collect(self.iter().map(T::to_json).map(Result::ok))
            .ok_or("falied to deserialize a mirror")?;
        Ok(serde_json::json!(array))
    }

    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        self.iter()
            .flat_map(|mirror| mirror.render_data(display))
            .collect()
    }

    fn random<U: rand::Rng + ?Sized>(rng: &mut U) -> Self
    where
        Self: Sized,
    {
        const MIN_RANDOM_MIRRORS: usize = 8;
        const MAX_RANDOM_MIRRORS: usize = 64;

        let num_mirrors = rng.gen_range(MIN_RANDOM_MIRRORS..=MAX_RANDOM_MIRRORS);

        iter::repeat_with(|| T::random(rng))
            .take(num_mirrors)
            .collect()
    }
}

pub mod util {
    use super::*;

    pub fn random_vector<T: rand::Rng + ?Sized, const D: usize>(
        rng: &mut T,
        max_coord_mag: f32,
    ) -> SVector<f32, D> {
        // the rng generates floats in 0.0..1.0, scale and translate the range accordingly

        SVector::<f32, D>::from_fn(|_, _| (rng.gen::<f32>() - 0.5) * (max_coord_mag.abs() * 2.0))
    }

    /// This is essentially `try_into` then `try_map` but the latter is nightly-only
    pub fn json_array_to_float_array<const D: usize>(
        json_array: &[serde_json::Value],
    ) -> Option<[f32; D]> {
        let array: &[serde_json::Value; D] = json_array.try_into().ok()?;

        let mut center_coords_array = [0.; D];
        for (coord, value) in center_coords_array.iter_mut().zip(array) {
            *coord = value.as_f64()? as f32;
        }
        Some(center_coords_array)
    }

    /// This is essentially `try_into` then `try_map` but the latter is nightly-only
    pub fn json_array_to_vector<const D: usize>(
        json_array: &[serde_json::Value],
    ) -> Option<SVector<f32, D>> {
        json_array_to_float_array(json_array).map(SVector::from)
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
}

#[cfg(test)]
mod tests {
    use nalgebra::Unit;
    use serde_json::json;

    use crate::Simulation;

    use super::{plane::PlaneMirror, Mirror, Plane, Ray};

    #[test]
    fn test_loop_detection() {
        let simulation = Simulation::<Box<dyn Mirror<3>>, 3>::from_json(&json!(
            {
                "mirror":
                {"type": "[]plane",
                "mirror":
                [
                    {
                            "center": [1., 0., 0.],
                            "basis": [
                                [0., 1., 0.],
                                [0., 0., 1.],
                            ],
                            "bounds": [1.,1.],
                    },
                    {
                            "center": [-1., 0., 0.],
                            "basis": [
                                [0., 1., 0.],
                                [0., 0., 1.],
                            ],
                            "bounds": [1.,1.],
                    }
                ],
                },
                "rays": [
                    {
                        "origin": [0., 0., 0.],
                        "direction": [1., 0., 0.],
                    }
                ],
            }
        ))
        .unwrap();

        let path = simulation.get_ray_paths(100);
        assert!(path.first().unwrap().points.len() == 4);
    }
    #[test]
    fn test_no_loop_detection() {
        let simulation = Simulation::<Box<dyn Mirror<3>>, 3>::from_json(&json!(
            //Stollen from diamond of hell
            {
                "rays": [
                    {
                        "origin": [
                            1.0,
                            0.3,
                            0.0
                        ],
                        "direction": [
                            1.0,
                            1.223780308610836,
                            0.0
                        ]
                    }
                ],
                "mirror": {
                    "type": "[]plane",
                    "mirror": [
                        {
                            "center": [
                                1.0,
                                1.0,
                                0.0
                            ],
                            "basis": [
                                [
                                    -1.0,
                                    1.0,
                                    0.0
                                ],
                                [
                                    0.0,
                                    0.0,
                                    1.0
                                ]
                            ],
                            "bounds": [
                                1.5,
                                0.1
                            ]
                        },
                        {
                            "center": [
                                -1.0,
                                1.0,
                                0.0
                            ],
                            "basis": [
                                [
                                    -1.0,
                                    -1.0,
                                    0.0
                                ],
                                [
                                    0.0,
                                    0.0,
                                    1.0
                                ]
                            ],
                            "bounds": [
                                1.5,
                                0.1
                            ]
                        },
                        {
                            "center": [
                                1.0,
                                -1.0,
                                0.0
                            ],
                            "basis": [
                                [
                                    -1.0,
                                    -1.0,
                                    0.0
                                ],
                                [
                                    0.0,
                                    0.0,
                                    1.0
                                ]
                            ],
                            "bounds": [
                                1.5,
                                0.1
                            ]
                        },
                        {
                            "center": [
                                -1.0,
                                -1.0,
                                0.0
                            ],
                            "basis": [
                                [
                                    -1.0,
                                    1.0,
                                    0.0
                                ],
                                [
                                    0.0,
                                    0.0,
                                    1.0
                                ]
                            ],
                            "bounds": [
                                1.5,
                                0.1
                            ]
                        }
                    ]
                }
            }
        ))
        .unwrap();

        let path = simulation.get_ray_paths(100);
        assert!(path.first().unwrap().points.len() == 101);
    }
}
