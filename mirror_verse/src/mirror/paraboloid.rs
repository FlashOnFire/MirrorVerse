use glium::index;
use nalgebra::{Point2, Vector2};

use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ParaboloidMirror<const D: usize> {
    /// The plane this mirror belongs to.
    directrix_plane: Plane<D>,
    /// The focus the parabola is centered on
    focus: SVector<f32, D>,
    /// The limit of the parabola
    limit_plane: Plane<D>,
}

struct ParaboloidRenderData<const D: usize> {
    vertices: gl::VertexBuffer<render::Vertex<D>>,
}

impl<const D: usize> render::RenderData for ParaboloidRenderData<D> {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        gl::index::IndicesSource::NoIndices {
            primitives: index::PrimitiveType::LineStrip,
        }
    }
}

impl<const D: usize> ParaboloidMirror<D> {
    fn is_point_on_parabola(&self, point: &SVector<f32, D>) -> bool {
        let dist_to_directrix =
            (self.directrix_plane.orthogonal_point_projection(*point) - *point).norm();
        let dist_to_focus = (self.focus - *point).norm();
        let distance_ok = (dist_to_directrix - dist_to_focus).abs() < 1e-5;
        //check if the point is on the right side of the limit plane
        let point_projection_on_limit_plane = self.limit_plane.orthogonal_projection(*point);
        let focus_projection_on_limit_plane = self.limit_plane.orthogonal_projection(self.focus);
        //check if the two vector are in the same direction
        let same_direction = (point_projection_on_limit_plane - focus_projection_on_limit_plane)
            .dot(&(point - focus_projection_on_limit_plane))
            > f32::EPSILON;
        distance_ok && same_direction
    }

    fn get_tangent(&self, point: &SVector<f32, D>) -> Option<Plane<D>> {
        if D != 2 {
            panic!("InvalidDimension");
        }
        if !self.is_point_on_parabola(point) {
            return None;
        }
        //calculate the line to the directrix
        let point_to_directrix_direction =
            self.directrix_plane.orthogonal_point_projection(*point) - *point;
        //calculate the line to the focus
        let point_to_focus_direction = self.focus - *point;

        //calculate the tangent
        let direction = point_to_directrix_direction + point_to_focus_direction;

        let mut vectors = [SVector::zeros(); D];
        vectors[0] = *point;
        vectors[1] = direction;

        Some(Plane::new(vectors).unwrap())
    }

    fn get_points(&self) -> Vec<SVector<f32, D>> {
        if D != 2 {
            panic!("InvalidDimension");
        }
        // lets construct the formula of the parametric equation of the parabola
        let focus = self.focus;
        let directrix_point = self.directrix_plane.v_0();
        let directrix_vector = self.directrix_plane.basis()[0];

        //the directix formula from ax + by + c = 0
        let directrix_formula = SVector::from([
            directrix_vector[1],
            -directrix_vector[0],
            -directrix_vector[1] * directrix_point[0] + directrix_vector[0] * directrix_point[1],
        ]);

        /*the parabola formula (with F=(f1,f2) the focus)
            (ax+by+c)^2
            ----------- = (x-f1)^2 + (y-f2)^2
              a^2+b^2
        */
        let denominator = directrix_vector.norm_squared();
        let numerator = directrix_formula.norm_squared();

        todo!();

        // let mut points = Vec::new();
        // for x in -100..100 {
        //     let y = ((numerator - ((x as f32) / 10. - focus[0]).powi(2)) / denominator).sqrt()
        //         + focus[1];
        //     let mut vector = SVector::zeros();
        //     vector[0] = x as f32 / 10.;
        //     vector[1] = y;
        //     points.push(vector);
        // }

        // points

        // let focus = self.focus;
        // let directrix_point = self.directrix_plane.v_0();
        // let directrix_vector = self.directrix_plane.basis()[0];

        // let parabola = |t: f32| -> [f32; 2] {
        //     [
        //         focus[0] + (t.powf(2.)) * directrix_vector[0] - 2. * t * directrix_point[0],
        //         focus[1] + (t.powf(2.)) * directrix_vector[1] - 2. * t * directrix_point[1],
        //     ]
        // };

        // let mut point = SVector::zeros();
        // for i in 0..2 {
        //     point[i] = parabola(t)[i];
        // }

        // point
    }
}

impl<const D: usize> Mirror<D> for ParaboloidMirror<D> {
    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<Tangent<D>>) {
        if D != 2 {
            panic!("InvalidDimension");
        }
        // Define the focus and directrix
        let focus = Point2::new(self.focus[0], self.focus[1]); // Focus of the parabola
        let directrix_point =
            Point2::new(self.directrix_plane.v_0()[0], self.directrix_plane.v_0()[1]); // A point on the directrix line
        let directrix_vector = Vector2::new(
            self.directrix_plane.basis()[0][0],
            self.directrix_plane.basis()[0][1],
        ); // Direction vector of the directrix line

        // Define the line
        let line_point = Point2::new(ray.origin[0], ray.origin[1]); // A point on the line
        let line_direction = Unit::new_normalize(Vector2::new(ray.direction[0], ray.direction[1])); // Direction vector of the line

        let func = |t: f32| -> f32 {
            //x and y of the line
            let x = line_point[0] + t * line_direction[0];
            let y = line_point[1] + t * line_direction[1];
            let dx = x - directrix_point[0];
            let dy = y - directrix_point[1];
            let numerator = (x - focus[0]).powi(2) + (y - focus[1]).powi(2);
            let denominator = directrix_vector[1].powi(2) + directrix_vector[0].powi(2);
            numerator - (dx * directrix_vector[1] - dy * directrix_vector[0]).powi(2) / denominator
        };

        // Solve the equation
        let t0 = 1.; // Initial guess for the first root
        let solution = newton_raphson(t0, func).unwrap();
        let mut intersection_points = [Point2::new(0., 0.); 2];
        intersection_points[0] = line_point + solution * line_direction.into_inner();

        //calculate the t1 by adding the distance beetween the ray and the focus or substract if if we are on the right side

        let ray_to_focus = focus - line_point;
        let t1 = if ray_to_focus.dot(&line_direction) > 0. {
            solution + ray_to_focus.norm()
        } else {
            solution - ray_to_focus.norm()
        };

        let solution = newton_raphson(t1, func).unwrap();
        intersection_points[1] = line_point + solution * line_direction.into_inner();

        for intersection_point in intersection_points[1..].iter() {
            if self.is_point_on_parabola(&SVector::from_vec(vec![
                intersection_point[0],
                intersection_point[1],
            ])) {
                println!("saiohcbnpriouygoriuhnostz");
                self.get_tangent(&SVector::from_vec(vec![
                    intersection_point[0],
                    intersection_point[1],
                ]));
                let tangent = self.get_tangent(&SVector::from_vec(vec![
                    intersection_point[0],
                    intersection_point[1],
                ]));
                //TODO j'ai mis ca de manière random
                let normal = tangent.unwrap().basis()[0];
                list.push(Tangent::Normal {
                    origin: SVector::from_vec(vec![intersection_point[0], intersection_point[1]]),
                    normal: Unit::new_normalize(normal),
                });
            }
        }
    }

    fn get_json_type() -> String {
        "paraboloid".into()
    }

    fn get_json_type_inner(&self) -> String {
        "paraboloid".into()
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        let directrix_plane = Plane::from_json(&json["directrix_plane"])?;
        let limit_plane = Plane::from_json(&json["limit_plane"])?;

        let focus_json = json["focus"].as_array().ok_or("Invalid JSON")?;
        let focus = SVector::from_vec(
            focus_json
                .iter()
                .map(|val| val.as_f64().unwrap() as f32)
                .collect(),
        );

        Ok(Self {
            directrix_plane,
            focus,
            limit_plane,
        })
    }

    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let mut json = serde_json::Map::new();
        json.insert("directrix_plane".into(), self.directrix_plane.to_json()?);
        json.insert("focus".into(), self.focus.iter().cloned().collect());
        json.insert("limit_plane".into(), self.limit_plane.to_json()?);
        Ok(json.into())
    }

    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        let mut points: Vec<SVector<f32, D>> = Vec::new();
        for i in self.get_points() {
            points.push(i);
        }
        // for i in (-1000..=1000).step_by(1) {
        //     points.push(self.get_point(i as f32 / 10.));
        // }

        let paraboloid_render_data = ParaboloidRenderData {
            vertices: gl::VertexBuffer::new(
                display,
                &points
                    .iter()
                    .map(|point| {
                        let mut position: [f32; 3] = [0.; 3];
                        for i in 0..2 {
                            position[i] = point[i];
                        }
                        render::Vertex { position }
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        };
        vec![Box::new(paraboloid_render_data)]
    }

    fn random<T: rand::Rng>(rng: &mut T) -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

fn newton_raphson<F>(guess: f32, f: F) -> Option<f32>
where
    F: Fn(f32) -> f32,
{
    let mut x = guess;
    let mut dx;

    for _ in 0..1000 {
        // Maximum 1000 iterations
        dx = f(x) / (f(x + 0.01) - f(x)) * 0.01; // Numerical derivative
        if dx.abs() < f32::EPSILON {
            // Convergence criterion
            return Some(x);
        }
        x -= dx;
    }

    None // Did not converge
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_on_parabola() {
        let paraboloid = super::ParaboloidMirror::<2> {
            directrix_plane: super::Plane::new([
                super::SVector::from_vec(vec![0., -1.]),
                super::SVector::from_vec(vec![1., 1.]),
            ])
            .unwrap(),
            focus: super::SVector::from_vec(vec![0., 0.]),
            limit_plane: super::Plane::new([
                super::SVector::from_vec(vec![0., 0.]),
                super::SVector::from_vec(vec![0., 1.]),
            ])
            .unwrap(),
        };
        assert!(paraboloid.is_point_on_parabola(&super::SVector::from_vec(vec![0.44948974, 1.])));
    }
}
