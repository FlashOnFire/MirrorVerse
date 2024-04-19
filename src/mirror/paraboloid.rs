use nalgebra::{Point2, Vector2};

use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ParaboloidMirror<const D: usize = DEFAULT_DIM> {
    /// The plane this mirror belongs to.
    directrix_plane: Plane<D>,
    /// The focus the parabola is centered on
    focus: SVector<f32, D>,
    /// The limit of the parabola
    limit_plane: Plane<D>,
}

impl<const D: usize> ParaboloidMirror<D> {
    fn is_point_on_parabola(&self, point: &SVector<f32, D>) -> bool {
        let dist_to_directrix =
            (self.directrix_plane.orthogonal_point_projection(*point) - *point).norm();
        let dist_to_focus = (self.focus - *point).norm();
        let distance_ok = (dist_to_directrix.powi(2) - 2. * dist_to_focus).abs() < f32::EPSILON;
        //check if the point is on the right side of the limit plane
        let point_projection_on_limit_plane = self.limit_plane.orthogonal_projection(*point);
        let focus_projection_on_limit_plane = self.limit_plane.orthogonal_projection(self.focus);
        //check if the two vector are in the same direction
        let same_direction = (point_projection_on_limit_plane - focus_projection_on_limit_plane)
            .dot(&(point - focus_projection_on_limit_plane))
            > f32::EPSILON;
        distance_ok && same_direction
    }
}

impl ParaboloidMirror<2> {
    fn get_tangent(&self, point: &SVector<f32, 2>) -> Option<Plane<2>> {
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

        Some(Plane::new([*point, direction]).unwrap())
    }
}

impl Mirror<2> for ParaboloidMirror<2> {
    fn append_intersecting_points(&self, ray: &Ray<2>, list: &mut Vec<Tangent<2>>) {
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
        let solution = newton_raphson(t0, func).unwrap(); // You need to implement the Newton-Raphson method
        let mut intersection_points = [Point2::new(0., 0.); 2];
        intersection_points[0] = line_point + solution * line_direction.into_inner();

        //calculate the t1 by adding the distance beetween the ray and the focus or substract if if we are on the right side

        let ray_to_focus = focus - line_point;
        let t1 = if ray_to_focus.dot(&line_direction) > 0. {
            solution + ray_to_focus.norm()
        } else {
            solution - ray_to_focus.norm()
        };

        let solution = newton_raphson(t1, func).unwrap(); // You need to implement the Newton-Raphson method
        intersection_points[1] = line_point + solution * line_direction.into_inner();

        for intersection_point in intersection_points.iter() {
            if self.is_point_on_parabola(&SVector::from_vec(vec![
                intersection_point[0],
                intersection_point[1],
            ])) {
                list.push(
                    // TODO with the new method of momo aucun soucis on utilise
                    // la tangent self.get_tangent(
                    //     &[intersection_point[0], intersection_point[1]].into()
                    // ).unwrap(),
                    Tangent::Normal {
                        origin: [intersection_point[0], intersection_point[1]].into(),
                        normal: Unit::new_normalize([1., 1.].into()),
                    },
                );
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
        /*
        example json:

        */

        todo!()
    }

    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        todo!()
    }

    fn render_data(
        &self,
        display: &gl::Display,
    ) -> Vec<(gl::index::NoIndices, gl::VertexBuffer<render::Vertex<2>>)> {
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
mod tests {}
