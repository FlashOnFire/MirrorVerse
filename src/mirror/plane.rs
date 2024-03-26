use super::*;

/// A parallelotope-shaped reflective (hyper)plane
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlaneMirror<const D: usize = DEFAULT_DIM> {
    /// The plane this mirror belongs to.
    pub plane: Plane<D>,
    /// maximum magnitudes (mu_i_max) of the scalars in the linear combination of the
    /// basis vectors of the associated hyperplane.
    ///
    /// Formally, for all vectors `v = sum mu_i * v_i` of
    /// the hyperplane, `v` is in this plane mirror iff for all `i`, `|mu_i| <= |mu_i_max|`
    ///
    /// Note: the first value of this array is irrelevant
    pub bounds: [f32; D],
    /// Coefficient describing the darkness of the mirror which will be applied to the brightness
    darkness_coef: f32,
}

impl<const D: usize> PlaneMirror<D> {
    pub fn vertex(&self) -> Vec<SVector<f32, D>> {
        let mut vertices = Vec::<SVector<f32, D>>::with_capacity(2usize.pow((D - 1) as u32));
        let possibility = vec![vec![-1.0, 1.0]; D - 1];
        let combinations = cartesian_product(&possibility);

        for combination in combinations.into_iter() {
            let mut vertex = *self.plane.v_0();
            for (i, value) in combination.iter().enumerate() {
                vertex += (self.plane.basis()[i] * self.bounds[i+1]) * *value;
            }
            vertices.push(vertex);
        }

        vertices
    }
}

impl<const D: usize> Mirror<D> for PlaneMirror<D> {
    fn intersecting_planes(&self, ray: &Ray<D>) -> Vec<(f32, Plane<D>)> {
        let mut list = vec![];
        self.append_intersecting_planes(ray, &mut list);
        list
    }

    fn append_intersecting_planes(&self, ray: &Ray<D>, list: &mut Vec<(f32, Plane<D>)>) {
        let mut a = SMatrix::<f32, D, D>::zeros();

        /* // bien vuu le boss
        Fill the matrix "a" with the direction of the ray and the basis of the plane
        exemple
        | ray_direction.x | plane_basis_1.x | plane_basis_2.x | ...
        | ray_direction.y | plane_basis_1.y | plane_basis_2.y | ...
        | ray_direction.z | plane_basis_1.z | plane_basis_2.z | ...
        */

        a.column_iter_mut()
            .zip(iter::once(ray.direction.as_ref()).chain(self.plane.basis().iter()))
            .for_each(|(mut i, o)| i.set_column(0, o));

        if a.try_inverse_mut() {
            // a now contains a^-1
            let v = a * (self.plane.v_0() - ray.origin);
            if v.iter()
                .zip(&self.bounds)
                .skip(1)
                .all(|(mu, mu_max)| mu.abs() <= mu_max.abs())
                && ray.direction.dot(&-self.plane.normal_directed(ray.origin).unwrap()) > 0.0
            {
                list.push((self.darkness_coef, self.plane));
            }
        }
    }

    fn get_type(&self) -> &str {
        "plane"
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>>
        where
            Self: Sized,
    {
        /*
        example json:
        {
            "center": [9., 8., 7., ...], (N elements)
            "basis": [ (N - 1 elements)
                [9., 8., 7., ...], (N elements)
                [6., 5., 4., ...],
            ],
            "bounds": [6., 9., ...] (N - 1 elements)
            "darkness": 0.5,
        }
        */

        let mut vectors = [SVector::zeros(); D];

        let (v_0, basis) = vectors.split_first_mut().unwrap();

        *v_0 = json
            .get("center")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .and_then(json_array_to_vector)
            .ok_or("Failed to parse center")?;

        let basis_json = json
            .get("basis")
            .and_then(Value::as_array)
            .filter(|l| l.len() == D - 1)
            .ok_or("Failed to parse basis")?;

        for (value, vector) in basis_json.iter().zip(basis) {
            *vector = value
                .as_array()
                .map(Vec::as_slice)
                .and_then(json_array_to_vector)
                .ok_or("Failed to parse basis vector")?;
        }

        let bounds_json = json
            .get("bounds")
            .and_then(Value::as_array)
            .filter(|l| l.len() == D - 1)
            .ok_or("Failed to parse bounds")?;

        let mut bounds = [0.; D];
        for (i, o) in bounds[1..].iter_mut().zip(bounds_json.iter()) {
            *i = o.as_f64().ok_or("Failed to parse bound")? as f32;
        }

        let darkness_coef = json
            .get("darkness")
            .and_then(Value::as_f64)
            .map(|f| f as f32)
            .unwrap_or(1.0);

        let plane = Plane::new(vectors).ok_or("Failed to create plane")?;

        Ok(Self {
            plane,
            bounds,
            darkness_coef,
        })
    }
}

fn cartesian_product(arrays: &Vec<Vec<f32>>) -> Vec<Vec<f32>> {
    let mut result: Vec<Vec<f32>> = vec![vec![]];
    for array in arrays {
        let mut temp = Vec::new();
        for x in &result {
            for y in array {
                let mut z = x.clone();
                z.push(*y);
                temp.push(z);
            }
        }
        result = temp;
    }
    result
}

#[cfg(test)]
mod tests {
    use core::f32::consts::FRAC_1_SQRT_2;

    use super::*;

    #[test]
    fn test_2d_horizontal() {
        /*
                |
          ----->|
                |
        */

        let mirror = PlaneMirror {
            plane: Plane::new([
                SVector::from([0.0, 0.0]),
                //              x    y
                SVector::from([0.0, 1.0]),
            ])
                .unwrap(),
            bounds: [1.0; 2],
            darkness_coef: 1.0,
        };
        let ray = Ray {
            origin: [-1.0, 0.0].into(),
            direction: Unit::new_normalize([1.0, 0.0].into()),
            brightness: 1.0,
        };
        let reflections = mirror.intersecting_planes(&ray);

        let &[(brightness, plane)] = reflections.as_slice() else {
            panic!("there must be one plane");
        };

        assert_eq!(brightness, 1.0);
        assert_eq!(
            plane,
            Plane::new([[0.0, 0.0].into(), [0.0, 1.0].into(), ]).unwrap()
        )
    }

    #[test]
    fn test_2d_vertical() {
        /*
        ---------
            ^
            |
            |
        */

        let mirror = PlaneMirror {
            plane: Plane::new([
                SVector::from_vec(vec![0.0, 0.0]),
                //                      x    y
                SVector::from_vec(vec![1.0, 0.0]),
            ])
                .unwrap(),
            bounds: [1.0; 2],
            darkness_coef: 1.0,
        };
        let ray = Ray {
            origin: [0.0, -1.0].into(),
            direction: nalgebra::Unit::new_normalize(SVector::from_vec(vec![0.0, 1.0])),
            brightness: 1.0,
        };
        let reflections = mirror.intersecting_planes(&ray);

        let &[(brightness, plane)] = reflections.as_slice() else {
            panic!("there must be one plane");
        };

        assert_eq!(brightness, 1.0);
        assert_eq!(
            plane,
            Plane::new([
                SVector::from_vec(vec![0.0, 0.0]),
                SVector::from_vec(vec![1.0, 0.0]),
            ])
                .unwrap()
        );
    }

    #[test]
    fn test_2d_diagonal() {
        let mirror = PlaneMirror {
            plane: Plane::new([
                SVector::from_vec(vec![0.0, 0.0]),
                SVector::from_vec(vec![FRAC_1_SQRT_2, FRAC_1_SQRT_2]),
            ])
                .unwrap(),
            bounds: [1.0; 2],
            darkness_coef: 1.0,
        };

        let ray = Ray {
            origin: [-1.0, 1.0].into(),
            direction: nalgebra::Unit::new_normalize(SVector::from_vec(vec![1.0, -1.0])),
            brightness: 1.0,
        };

        let reflections = mirror.intersecting_planes(&ray);

        let &[(brightness, plane)] = reflections.as_slice() else {
            panic!("there must be one plane");
        };

        assert_eq!(brightness, 1.0);
        assert_eq!(
            plane,
            Plane::new([
                SVector::from_vec(vec![0.0, 0.0]),
                SVector::from_vec(vec![FRAC_1_SQRT_2, FRAC_1_SQRT_2]),
            ])
                .unwrap()
        );
    }

    #[test]
    fn test_no_reflection_2d() {
        /*
        ---->
        _____
        */

        let mirror = PlaneMirror {
            plane: Plane::new([
                SVector::from_vec(vec![0.0, 0.0]),
                SVector::from_vec(vec![1.0, 0.0]),
            ])
                .unwrap(),
            bounds: [1.0; 2],
            darkness_coef: 1.0,
        };

        let ray = Ray {
            origin: [0.0, 1.0].into(),
            direction: nalgebra::Unit::new_normalize(SVector::from_vec(vec![1.0, 0.0])),
            brightness: 1.0,
        };

        let reflections = mirror.intersecting_planes(&ray);
        assert!(reflections.is_empty());
    }

    #[test]
    fn test_no_reflection_2d_2() {
        /*
                |
        <----   |
                |
        */

        let mirror = PlaneMirror {
            plane: Plane::new([
                SVector::from_vec(vec![1.0, 0.0]),
                SVector::from_vec(vec![0.0, 1.0]),
            ])
                .unwrap(),
            bounds: [1.0; 2],
            darkness_coef: 1.0,
        };

        let ray = Ray {
            origin: [0.0, 0.0].into(),
            direction: nalgebra::Unit::new_normalize(SVector::from_vec(vec![-1.0, 0.0])),
            brightness: 1.0,
        };

        let reflections = mirror.intersecting_planes(&ray);
        assert!(reflections.is_empty());
    }

    #[test]
    fn test_json() {
        let json = serde_json::json!({
            "center": [0., 0.],
            "basis": [
                [1., 0.],
            ],
            "bounds": [1.],
            "darkness": 0.5,
        });

        let mirror: PlaneMirror<2> = PlaneMirror::<2>::from_json(&json).unwrap();
        assert_eq!(
            mirror,
            PlaneMirror {
                plane: Plane::new([
                    SVector::from_vec(vec![0.0, 0.0]),
                    SVector::from_vec(vec![1.0, 0.0]),
                ])
                    .unwrap(),
                bounds: [0., 1.],
                darkness_coef: 0.5,
            }
        );
    }

    #[test]
    fn test_vertex() {
        /*
                |
                |
                |
        */
        let mirror = PlaneMirror {
            plane: Plane::new([SVector::from([0.0, 0.0]), SVector::from([0.0, 1.0])]).unwrap(),
            bounds: [1.0; 2],
            darkness_coef: 1.0,
        };
        let vertices = mirror.vertex();
        assert_eq!(vertices.len(), 2);
        assert_eq!(vertices[0], SVector::from([0.0, -1.0]));
        assert_eq!(vertices[1], SVector::from([0.0, 1.0]));
    }

    #[test]
    fn test_vertex_2() {
        let mirror = PlaneMirror {
            plane: Plane::new([SVector::from([1.0, 0.0]), SVector::from([0.0, 1.0])]).unwrap(),
            bounds: [0., 1.0],
            darkness_coef: 1.0,
        };
        let vertices = mirror.vertex();
        assert_eq!(vertices.len(), 2);
        println!("{:?}", vertices);
        assert_eq!(vertices[0], SVector::from([1.0, -1.0]));
        assert_eq!(vertices[1], SVector::from([1.0, 1.0]));
    }

    #[test]
    fn test_vertex_3() {
        let mirror = PlaneMirror {
            plane: Plane::new([SVector::from([0.0, 0.0]), SVector::from([-1.0, -1.0])]).unwrap(),
            bounds: [1.0; 2],
            darkness_coef: 1.0,
        };
        let vertices = mirror.vertex();
        assert_eq!(vertices.len(), 2);
        println!("{:?}", vertices);
        assert_eq!(vertices[0], SVector::from([0.70710677, 0.70710677]));
        assert_eq!(vertices[1], SVector::from([-0.70710677, -0.70710677]));
    }

    #[test]
    fn test_vertex_4() {
        let mirror = PlaneMirror {
            plane: Plane::new([SVector::from([0.0, -5.0]), SVector::from([1.0, -1.0])]).unwrap(),
            bounds: [1.0; 2],
            darkness_coef: 1.0,
        };
        let vertices = mirror.vertex();
        assert_eq!(vertices.len(), 2);
        println!("{:?}", vertices);
        assert_eq!(vertices[0], SVector::from([-0.70710677, -4.2928934]));
        assert_eq!(vertices[1], SVector::from([0.70710677, -5.7071066]));
    }

    #[test]
    fn test_vertex_3d(){
        let mirror = PlaneMirror {
            plane: Plane::new([
                SVector::from([0.0, 0.0, 0.0]),
                SVector::from([0.0, 1.0, 0.0]),
                SVector::from([0.0, 0.0, 1.0]),
            ])
            .unwrap(),
            bounds: [1.0; 3],
            darkness_coef: 1.0,
        };
        let vertices = mirror.vertex();
        assert_eq!(vertices.len(), 4);
        println!("{:?}", vertices);
        assert_eq!(vertices[0], SVector::from([0.0, -1.0, -1.0]));
        assert_eq!(vertices[1], SVector::from([0.0, -1.0, 1.0]));
        assert_eq!(vertices[2], SVector::from([0.0, 1.0, -1.0]));
        assert_eq!(vertices[3], SVector::from([0.0, 1.0, 1.0]));
    }
}
