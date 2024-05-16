use self::render::OpenGLRenderable;

use super::*;

/// An open, cylinder-shaped mirror,
pub struct CylindricalMirror {
    start: SVector<Float, 3>,
    dist: SVector<Float, 3>,
    inv_norm_dist_squared: Float,
    radius: Float,
    radius_sq: Float,
}

impl CylindricalMirror {
    /// Create a new cylinder from a line segment and a radius
    pub fn new(line_segment: [SVector<Float, 3>; 2], radius: Float) -> Option<Self> {
        const E: Float = Float::EPSILON * 8.0;

        let [start, end] = line_segment;
        let dist = end - start;
        let dist_sq = dist.norm_squared();

        let r_abs = radius.abs();
        (dist_sq.sqrt() > E && r_abs > E).then(|| Self {
            start,
            dist,
            radius,
            radius_sq: radius * radius,
            inv_norm_dist_squared: dist_sq.recip(),
        })
    }
}

impl Mirror<3> for CylindricalMirror {
    fn append_intersecting_points(&self, ray: &Ray<3>, list: &mut Vec<Tangent<3>>) {
        let line_coord = |v| self.dist.dot(&v) * self.inv_norm_dist_squared;
        let p = |v| line_coord(v) * self.dist;

        let m = ray.origin - self.start;
        let d = ray.direction.into_inner();
        let pm = p(m);
        let pd = p(d);

        let c = (m - pm).norm_squared() - self.radius_sq;

        let b = m.dot(&d) - 2.0 * d.dot(&pm) + pm.dot(&pd);

        let a = (d - pd).norm_squared();

        let delta = b * b - a * c;

        if delta > Float::EPSILON {

            let root_delta = delta.sqrt();
            let neg_b = -b;

            for t in [(neg_b - root_delta) / a, (neg_b + root_delta) / a] {
                let origin = ray.at(t);
                let coord = line_coord(origin);

                let line_pt = self.dist * coord;

                if coord <= 1.0 && coord >= 0.0 {
                    // SAFETY: the length of origin - line_pt is always self.radius
                    let normal = Unit::new_unchecked((origin - line_pt) / self.radius);

                    list.push(Tangent::Normal { origin, normal })
                }
            }
        }
    }
}

impl JsonType for CylindricalMirror {
    fn json_type() -> String {
        "cylinder".into()
    }
}

impl JsonDes for CylindricalMirror {
    /// Deserialize a new cylindrical mirror from a JSON object.
    /// 
    /// The JSON object must follow the following format:
    /// 
    /// ```ignore
    /// {
    ///     "start": [1.0, 2.0, 3.0],
    ///     "end": [4.0, 5.0, 6.0],
    ///     "radius": 69.0,
    /// }
    /// ```
    fn from_json(json: &serde_json::Value) -> Result<Self, Box<dyn Error>> {

        let start = json
            .get("start")
            .and_then(serde_json::Value::as_array)
            .map(Vec::as_slice)
            .and_then(util::json_array_to_vector)
            .ok_or("Failed to parse start")?;

        let end = json
            .get("end")
            .and_then(serde_json::Value::as_array)
            .map(Vec::as_slice)
            .and_then(util::json_array_to_vector)
            .ok_or("Failed to parse end")?;

        let radius = json
            .get("radius")
            .and_then(serde_json::Value::as_f64)
            .ok_or("Failed to parse radius")? as Float;

        Self::new([start, end], radius)
            .ok_or("radius is too small or start and end vectors are too close".into())
    }
}

impl JsonSer for CylindricalMirror {
    /// Serialize a cylindrical mirror into a JSON object.
    /// 
    /// The format of the returned object is explained in [`Self::from_json`]
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "start": self.start.as_slice(),
            "end": (self.start + self.dist).as_slice(),
            "radius": self.radius,
        })
    }
}

struct CylinderRenderData {
    vertices: gl::VertexBuffer<render::Vertex3D>,
}

impl render::RenderData for CylinderRenderData {
    fn vertices(&self) -> gl::vertex::VerticesSource {
        (&self.vertices).into()
    }

    fn indices(&self) -> gl::index::IndicesSource {
        gl::index::IndicesSource::NoIndices {
            primitives: gl::index::PrimitiveType::TriangleStrip,
        }
    }
}

impl OpenGLRenderable for CylindricalMirror {
    fn render_data(&self, display: &gl::Display) -> Vec<Box<dyn render::RenderData>> {
        const NUM_POINTS: usize = 360;

        let k = SVector::from([0.0, 0.0, 1.0]) + self.dist.normalize().map(|s| s as f32);

        let m = SMatrix::<_, 3, 3>::from_fn(|i, j| k[i] * k[j]);

        // rotation matrix to rotate the circle so it faces the axis formed by our line segment
        let rot = 2.0 / k.norm_squared() * m - SMatrix::identity();

        let r = self.radius as f32;
        let d = self.dist.map(|s| s as f32);
        let start = self.start.map(|s| s as f32);

        use core::f32::consts::TAU;

        let vertices: Vec<_> = (0..=NUM_POINTS)
            .flat_map(|i| {
                let [x, y]: [f32; 2] = (i as f32 / NUM_POINTS as f32 * TAU).sin_cos().into();
                let vertex = [x * r, y * r, 0.0];
                let v = rot * SVector::<f32, 3>::from(vertex) + start;
                [v, v + d]
            })
            .map(render::Vertex3D::from)
            .collect();

        let vertices = gl::VertexBuffer::immutable(display, vertices.as_slice()).unwrap();

        vec![Box::new(CylinderRenderData { vertices })]
    }
}

impl Random for CylindricalMirror {
    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized,
    {
        loop {
            if let Some(mirror) = Self::new(
                [util::random_vector(rng, 10.0), util::random_vector(rng, 10.0)],
                rng.gen::<Float>() * 4.0,
            ) {
                break mirror;
            }
        }
    }
}
