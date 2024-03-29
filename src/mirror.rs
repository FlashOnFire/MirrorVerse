use core::{fmt, iter, ops::Sub};
use format as f;
use nalgebra::{ArrayStorage, Point, SMatrix, SVector, Unit, SVD};
use rand::Rng;
use serde_json::Value;
use std::error::Error;

pub mod bezier;
pub mod cubic_bezier;
pub mod paraboloid;
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
    /// Reflect the ray with respect to the given plane
    pub fn reflect(&self, reflection_point: &ReflectionPoint<D>, darkness_coef: &f32) -> Ray<D> {
        let plane_origin = reflection_point.origin;
        let plane_normal = reflection_point.normal.into_inner();
        let ray_origin = self.origin;
        let ray_direction = self.direction.into_inner();

        let t = (plane_origin - ray_origin).dot(&plane_normal) / ray_direction.dot(&plane_normal);

        let intersection_point = ray_origin + t * ray_direction;

        let reflected_direction: SVector<f32, D> =
            ray_direction - 2.0 * self.direction.dot(&plane_normal) * plane_normal;

        let reflected_origin = intersection_point - ray_direction * f32::EPSILON; // add a small offset to avoid self-intersection
        Ray {
            origin: reflected_origin,
            direction: Unit::new_normalize(reflected_direction),
            brightness: self.brightness * darkness_coef,
        }
    }

    pub fn at(&self, t: f32) -> SVector<f32, D> {
        self.origin + self.direction.into_inner() * t
    }

    /// Create a new ray with a given origin and direction
    pub fn from_json(json: &Value) -> Result<Self, Box<dyn Error>> {
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
            .ok_or("Missing ray origin")?;

        let direction = json
            .get("direction")
            .and_then(Value::as_array)
            .ok_or("Missing ray direction")?;

        let brightness = json.get("brightness").ok_or("Missing ray brightness")?;

        let origin = json_array_to_vector(origin).ok_or("Invalid ray origin")?;

        let direction = json_array_to_vector(direction).ok_or("Invalid ray direction")?;

        let direction =
            Unit::try_new(direction, f32::EPSILON).ok_or("Unable to normalize ray direction")?;

        let brightness = brightness
            .as_f64()
            .ok_or("Invalid ray brightness (not a number)")? as f32;

        Ok(Self {
            origin,
            direction,
            brightness,
        })
    }
}

/// A vector with an origin representing the normal to a plan used to reflect the ray
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionPoint<const D: usize = DEFAULT_DIM> {
    /// The point where the ray intersects the plane
    pub origin: SVector<f32, D>,
    /// The normal vector of the plane
    pub normal: Unit<SVector<f32, D>>,
}

impl<const D: usize> ReflectionPoint<D> {
    /// Create a new reflection point with a given point and normal
    pub fn new(point: SVector<f32, D>, normal: Unit<SVector<f32, D>>) -> Self {
        Self {
            origin: point,
            normal: normal,
        }
    }

    pub fn distance_to_ray(&self, ray: Ray<D>) -> f32 {
        (self.origin - ray.origin).norm()
    }
}

/// An affine hyperplane
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane<const D: usize = DEFAULT_DIM> {
    /// The first element of this array is the plane's "starting point" (i. e. v_0).
    /// The remaining N-1 vectors are a family spanning it's associated subspace.
    ///
    /// Note that an expression like `[T ; N - 1]`
    /// is locked under `#[feature(const_generic_exprs)]`
    vectors: [SVector<f32, D>; D],
}

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
    /// Assumes `b` is an orthonormal family. If such isn't
    /// the case, the result is unspecified.
    pub fn orthogonal_projection(&self, v: SVector<f32, D>) -> SVector<f32, D> {
        self.basis().iter().map(|e| v.dot(e) * e).sum()
    }

    /// Project a point onto the plane
    pub fn orthogonal_point_projection(&self, point: SVector<f32, D>) -> SVector<f32, D> {
        let v = point - self.v_0();
        self.v_0() + self.orthogonal_projection(v)
    }

    /// Calculate the normal vector of the plane by solving a linear system
    pub fn normal(&self) -> Option<Unit<SVector<f32, D>>> {
        match D {
            2 => {
                let mut normal = SVector::<f32, D>::zeros();
                normal[0] = -self.basis()[0][1];
                normal[1] = self.basis()[0][0];
                Some(Unit::new_normalize(normal))
            }
            3 => {
                // use cross product
                let mut normal = SVector::<f32, D>::zeros();
                let basis = self.basis();
                normal = basis[0].cross(&basis[1]);
                Some(Unit::new_normalize(normal))
            }
            _ => {
                const TRIAL_LIMIT: usize = 100;

                (0..TRIAL_LIMIT)
                    .map(|_| SVector::from_fn(|_, _| rand::random()))
                    // v in H <=> v == p_H(v)
                    .find_map(|v| Unit::try_new(v - self.orthogonal_projection(v), f32::EPSILON))
            }
        }
    }

    /// Calculate the normal vector of the plane and orient it to the side of the point
    pub fn normal_directed(&self, point: SVector<f32, D>) -> Option<Unit<SVector<f32, D>>> {
        let normal = self.normal().unwrap();
        let pointed_by_normal = self.v_0() + normal.as_ref();
        let pointed_by_neg_normal = self.v_0() - normal.as_ref();
        if (point - pointed_by_normal).norm() < (point - pointed_by_neg_normal).norm() {
            Some(normal)
        } else {
            Some(-normal)
        }
    }

    /// Returns the distance between the plane and a point
    /// probaly useless for distance to ray because ray have a direction
    ///           /
    /// -->     / <- nearest point to the plane using the ray direction
    ///       /  <- nearest point with orthogonal projection
    pub fn distance_to_point(&self, point: SVector<f32, D>) -> f32 {
        let v = point - self.v_0();
        let projection = self.orthogonal_projection(v);
        (v - projection).norm()
    }

    /// Returns the distance between the plane and a ray
    /// This function takes care of the ray direction
    pub fn distance_to_ray(&self, ray: Ray<D>) -> f32 {
        let plane_origin = self.v_0();
        let plane_normal = self.normal().unwrap();

        let plane_to_ray_origin = ray.origin - plane_origin;
        let distance_along_normal = plane_to_ray_origin.dot(&plane_normal);

        if distance_along_normal < 0.0 {
            // The closest point on the ray is behind the plane's origin
            return plane_to_ray_origin.norm();
        }

        let closest_point_on_ray = ray.origin + ray.direction.into_inner() * distance_along_normal;
        let distance_to_plane =
            (closest_point_on_ray - plane_origin).norm() + distance_along_normal;

        distance_to_plane
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
    fn intersecting_points(&self, ray: &Ray<D>) -> Vec<(f32, ReflectionPoint<D>)>;
    /// An optimised version of `Self::reflect` that potentially saves
    /// an allocation by writing into another `Vec`. Override this if needed.
    ///
    /// It is a logic error for this function to remove/reorder elements in `list`
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<(f32, ReflectionPoint<D>)>) {
        list.append(&mut self.intersecting_points(ray))
    }
    /// Returns a string slice, unique to the type
    /// (or inner type if type-erased) and coherent with it's json representation
    // TODO: should this be 'static ?
    fn get_type(&self) -> &'static str;
    /// Deserialises the mirror's data from the provided json string, returns `None` in case of error
    fn from_json(json: &Value) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

// Note that `T` is implicitly `Sized`
//
// This impl might not be necessary for the time being
impl<const D: usize, T: Mirror<D>> Mirror<D> for Box<T> {
    fn intersecting_points(&self, ray: &Ray<D>) -> Vec<(f32, ReflectionPoint<D>)> {
        self.as_ref().intersecting_points(ray)
    }

    fn get_type(&self) -> &'static str {
        self.as_ref().get_type()
    }

    fn from_json(json: &Value) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        T::from_json(json).map(Box::new)
    }
}

impl<const D: usize> Mirror<D> for Box<dyn Mirror<D>> {
    fn intersecting_points(&self, ray: &Ray<D>) -> Vec<(f32, ReflectionPoint<D>)> {
        self.as_ref().intersecting_points(ray)
    }

    fn get_type(&self) -> &'static str {
        "dynamic"
    }

    fn from_json(json: &Value) -> Result<Self, Box<dyn Error>>
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
            .ok_or("Invalid mirror type")?;
        let mirror = json.get("mirror").ok_or("Missing mirror data")?;

        match mirror_type {
            "plane" => plane::PlaneMirror::<D>::from_json(mirror)
                .map(|mirror| Box::new(mirror) as Box<dyn Mirror<D>>),
            "sphere" => {
                sphere::SphereMirror::<D>::from_json(mirror).map(|mirror| Box::new(mirror) as _)
            }
            _ => Err("Invalid mirror type".into()),
        }
    }
}

impl<const D: usize, T: Mirror<D>> Mirror<D> for Vec<T> {
    fn intersecting_points(&self, ray: &Ray<D>) -> Vec<(f32, ReflectionPoint<D>)> {
        let mut list = vec![];
        self.append_intersecting_points(ray, &mut list);
        list
    }

    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<(f32, ReflectionPoint<D>)>) {
        self.iter()
            .for_each(|mirror| mirror.append_intersecting_points(ray, list));
    }

    fn get_type(&self) -> &'static str {
        "composite"
    }

    fn from_json(json: &Value) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized,
    {
        /* example json
        [
            ... list of json values whose structure depends on `T`
        ]
         */

        json.as_array()
            .and_then(|json| try_collect(json.iter().map(T::from_json).map(Result::ok)))
            .ok_or_else(|| "Invalid mirror list".into())
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

pub fn json_array_to_float_array<const D: usize>(json_array: &[Value]) -> Option<[f32; D]> {
    let array: &[Value; D] = json_array.try_into().ok()?;

    let mut center_coords_array = [0.; D];
    for (coord, value) in center_coords_array.iter_mut().zip(array) {
        *coord = value.as_f64()? as f32;
    }
    Some(center_coords_array)
}

/// This is essentially `try_into` then `try_map` but the latter is nightly-only
pub fn json_array_to_vector<const D: usize>(json_array: &[Value]) -> Option<SVector<f32, D>> {
    json_array_to_float_array(json_array).map(SVector::from)
}

#[cfg(test)]
mod tests {
    use crate::mirror::Plane;
    use nalgebra::SVector;

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
            Unit::new_normalize(SVector::from([4., 5., 6.]))
        );
        assert_eq!(ray.brightness, 0.5);
    }

    #[test]
    fn test_normal_3d() {
        let plane = Plane::<3>::new([
            SVector::from([0., 0., 0.]),
            SVector::from([1., 0., 0.]),
            SVector::from([0., 1., 0.]),
        ])
        .unwrap();
        assert_eq!(
            plane.normal().unwrap().into_inner(),
            SVector::<f32, 3>::from([0., 0., 1.])
        );
    }

    #[test]
    fn test_normal_3d_2() {
        let plane = Plane::<3>::new([
            SVector::from([0., 0., 0.]),
            SVector::from([-2., 1., 3.]),
            SVector::from([1., 0., 3.]),
        ])
        .unwrap();
        let normal = plane.normal().unwrap();
        let theoric_normal = SVector::<f32, 3>::from([-3., -9., 1.]);
        //check that the normal is a multiple of the theoric normal
        println!("{:?} {:?}", normal, theoric_normal);
        for i in 0..3 {
            assert!(
                normal[i] / theoric_normal[i] - (normal[i] / theoric_normal[i]).round()
                    < f32::EPSILON
            );
        }
    }

    #[test]
    fn test_normal_2d() {
        let plane = Plane::<2>::new([SVector::from([0., 0.]), SVector::from([1., 0.])]).unwrap();
        assert_eq!(
            plane.normal().unwrap().into_inner(),
            SVector::<f32, 2>::from([0., 1.])
        );
    }

    #[test]
    fn test_normal_4d() {
        let plane = Plane::<4>::new([
            SVector::from([0., 0., 0., 0.]),
            SVector::from([1., 0., 0., 0.]),
            SVector::from([0., 1., 0., 0.]),
            SVector::from([0., 0., 1., 0.]),
        ])
        .unwrap();
        assert_eq!(
            plane.normal().unwrap().into_inner(),
            SVector::<f32, 4>::from([0., 0., 0., 1.])
        );
    }

    #[test]
    fn test_normal_2d_diagonal() {
        let plane =
            Plane::<2>::new([SVector::from([0., 0.]), SVector::from([-1.0, -1.0])]).unwrap();

        assert!(plane.normal().unwrap().sum() < f32::EPSILON * 8.0);
    }
}
