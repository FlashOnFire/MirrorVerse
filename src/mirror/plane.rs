use core::{mem, ops::Add};

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
    pub fn vector_bounds(&self) -> &[f32] {
        &self.bounds[1..]
    }

    pub fn get_vertices(&self) -> Vec<SVector<f32, D>> {
        // WARNING: black magic

        const SHIFT: usize = mem::size_of::<f32>() * 8 - 1;
        // f32::to_bits is not const yet
        let f_one_bits = f32::to_bits(1.0);

        let start_pt = *self.plane.v_0();
        (0..1 << D - 1)
            .into_iter()
            .map(|i| {
                self.vector_bounds()
                    .iter()
                    .enumerate()
                    // returns `mu` with the sign flipped if the `j`th bit in `i` is 1
                    .map(|(j, mu)| f32::from_bits(mu.abs().to_bits() ^ i >> j << SHIFT))
                    .zip(self.plane.basis())
                    .map(|(mu_signed, v)| mu_signed * v)
                    .fold(start_pt, Add::add)
            })
            .collect()
    }
}

impl<const D: usize> Mirror<D> for PlaneMirror<D> {
    fn intersecting_points(&self, ray: &Ray<D>) -> Vec<(f32, Tangent<D>)> {
        let mut list = vec![];
        self.append_intersecting_points(ray, &mut list);
        list
    }

    fn append_intersecting_points(&self, ray: &Ray<D>, list: &mut Vec<(f32, Tangent<D>)>) {
        if let Some(coords) = self
            .plane
            .intersection_coordinates(ray)
            .filter(|v| {
                v.iter()
                    .skip(1)
                    .zip(self.vector_bounds())
                    .all(|(mu, mu_max)| mu.abs() <= mu_max.abs())
            })
        {
            list.push((self.darkness_coef, Tangent::Plane(self.plane)));
        }
    }

    fn get_type(&self) -> &'static str {
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
            *i = (o.as_f64().ok_or("Failed to parse bound")? as f32).abs();
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

#[cfg(test)]
mod tests {
    
}
