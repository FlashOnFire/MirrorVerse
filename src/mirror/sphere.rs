use super::*;

#[derive(Clone, Copy)]
pub struct SphereMirror<const D: usize = DEFAULT_DIM> {
    center: SVector<f32, D>,
    radius: f32,
    darkness_coef: f32,
}

impl<const D: usize> Mirror<D> for SphereMirror<D> {
    fn intersecting_points(&self, ray: &Ray<D>) -> Vec<(f32, Tangent<D>)> {
        let mut list = vec![];
        let oc = ray.origin - self.center;
        let a = ray.direction.norm_squared();
        let b = oc.dot(&ray.direction);
        let c = oc.norm_squared() - self.radius * self.radius;
        let delta = b * b - a * c;

        if delta > 0.0 {
            let sqrt_delta = delta.sqrt();
            let neg_b = -b;
            let t = [neg_b - sqrt_delta / a, neg_b + sqrt_delta / a];
            for &t in t.iter() {
                if t > 0.0 {
                    let point = ray.at(t);
                    let normal = Unit::new_normalize(point - self.center);
                    //orient the normal to the ray
                    let normal = if normal.dot(&ray.direction) > 0.0 {
                        -normal
                    } else {
                        normal
                    };
                    list.push((self.darkness_coef, Tangent::new(point, normal)));
                }
            }
        }
        list
    }

    fn get_type(&self) -> &'static str {
        "sphere"
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        /* example json
        {
            "center": [1.0, 2.0, 3.0],
            "radius": 4.0,
            "darkness_coef": 0.5
        }
         */

        let center = json
            .get("center")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .and_then(json_array_to_vector)
            .ok_or("Failed to parse center")?;

        let radius = json
            .get("radius")
            .and_then(Value::as_f64)
            .ok_or("Failed to parse radius")? as f32;

        let darkness_coef = json
            .get("darkness_coef")
            .and_then(Value::as_f64)
            .ok_or("Failed to parse darkness coeff")? as f32;

        Ok(Self {
            center,
            radius,
            darkness_coef,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_mirror_from_json() {
        let json = serde_json::json!({
            "center": [1.0, 2.0],
            "radius": 4.0,
            "darkness_coef": 0.5,
        });

        let mirror = SphereMirror::<2>::from_json(&json).expect("json deserialisation failed");

        assert_eq!(mirror.center, SVector::from([1.0, 2.0]));
        assert_eq!(mirror.radius, 4.0);
        assert_eq!(mirror.darkness_coef, 0.5);
    }

    #[test]
    fn test_sphere_mirror_reflect() {
        let mirror = SphereMirror::<2> {
            center: SVector::from([0.0, 0.0]),
            radius: 1.0,
            darkness_coef: 1.0,
        };

        let ray = Ray {
            origin: SVector::from([-5.0, 0.0]),
            direction: Unit::new_normalize(SVector::from([1.0, 0.0])),
            brightness: 1.0,
        };

        let reflection_points = mirror.intersecting_points(&ray);

        assert_eq!(reflection_points.len(), 2);
        let (t, reflection_point) = &reflection_points[0];
        assert_eq!(*t, 1.0);
        assert_eq!(reflection_point.origin, SVector::from([-1.0, 0.0]));
        assert_eq!(
            reflection_point.normal.into_inner(),
            SVector::from([-1.0, 0.0])
        );

        let (t, reflection_point) = &reflection_points[1];
        assert_eq!(*t, 1.0);
        assert_eq!(reflection_point.origin, SVector::from([1.0, 0.0]));
        assert_eq!(
            reflection_point.normal.into_inner(),
            SVector::from([-1.0, 0.0])
        );
    }
    #[test]
    fn test_sphere_mirror_reflect_2() {
        let mirror = SphereMirror::<2> {
            center: SVector::from([0.0, 0.0]),
            radius: 1.0,
            darkness_coef: 1.0,
        };

        let ray = Ray {
            origin: SVector::from([0.0, 0.0]),
            direction: Unit::new_normalize(SVector::from([1.0, 0.0])),
            brightness: 1.0,
        };

        let reflection_points = mirror.intersecting_points(&ray);

        println!("{reflection_points:?}");

        assert_eq!(reflection_points.len(), 1);
        let (t, reflection_point) = &reflection_points[0];
        assert_eq!(*t, 1.0);
        assert_eq!(reflection_point.origin, SVector::from([1.0, 0.0]));
        assert_eq!(
            reflection_point.normal.into_inner(),
            SVector::from([-1.0, 0.0])
        );
    }

    #[test]
    fn test_sphere_mirror_reflect_3() {
        let mirror = SphereMirror::<2> {
            center: SVector::from([0.0, 0.0]),
            radius: 1.0,
            darkness_coef: 1.0,
        };

        let ray = Ray {
            origin: SVector::from([0.0, -5.0]),
            direction: Unit::new_normalize(SVector::from([0.0, 1.0])),
            brightness: 1.0,
        };

        let reflection_points = mirror.intersecting_points(&ray);

        assert_eq!(reflection_points.len(), 2);
        let (t, reflection_point) = &reflection_points[0];
        assert_eq!(*t, 1.0);
        assert_eq!(reflection_point.origin, SVector::from([0.0, -1.0]));
        assert_eq!(
            reflection_point.normal.into_inner(),
            SVector::from([0.0, -1.0])
        );

        let (t, reflection_point) = &reflection_points[1];
        assert_eq!(*t, 1.0);
        assert_eq!(reflection_point.origin, SVector::from([0.0, 1.0]));
        assert_eq!(
            reflection_point.normal.into_inner(),
            SVector::from([0.0, -1.0])
        );
    }
}
