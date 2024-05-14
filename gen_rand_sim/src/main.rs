use core::iter;
use std::{env, error::Error, fs::File};

use mirror_verse::{mirror, rand, serde_json, Simulation};

pub(crate) trait JsonTypeDyn {
    fn json_type_dyn(&self) -> String;
}

impl<T: mirror::JsonType + ?Sized> JsonTypeDyn for T {
    fn json_type_dyn(&self) -> String {
        T::json_type()
    }
}

trait JsonSerDyn: JsonTypeDyn + mirror::JsonSer {}

impl<T: JsonTypeDyn + mirror::JsonSer + ?Sized> JsonSerDyn for T {}

struct Dynamic2D(Vec<Box<dyn JsonSerDyn>>);

impl mirror::Random for Dynamic2D {
    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized,
    {
        let mirror_types = ["plane", "sphere"];

        const MIN_RANDOM_MIRRORS: usize = 8;
        const MAX_RANDOM_MIRRORS: usize = 64;

        let num_mirrors = rng.gen_range(MIN_RANDOM_MIRRORS..=MAX_RANDOM_MIRRORS);

        Self(
            iter::repeat_with(|| match rng.gen_range(0..mirror_types.len()) {
                0 => Box::new(mirror::plane::PlaneMirror::<2>::random(rng)),
                1 => Box::new(mirror::sphere::EuclideanSphereMirror::<2>::random(rng)) as Box<dyn JsonSerDyn>,
                _ => unreachable!(),
            })
            .take(num_mirrors)
            .collect(),
        )
    }
}

impl mirror::JsonSer for Dynamic2D {
    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {

        let mut json = vec![];

        for mirror in self.0.iter() {
            json.push(serde_json::json!({
                "type": mirror.json_type_dyn(),
                "mirror": mirror.to_json()?,
            }));
        }

        Ok(serde_json::json!({
            "type": "[]dynamic",
            "mirror": json,
        }))
    }
}

// copy paste lol

struct Dynamic3D(Vec<Box<dyn JsonSerDyn>>);

impl mirror::Random for Dynamic3D {
    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized,
    {
        let mirror_types = ["plane", "sphere"];

        const MIN_RANDOM_MIRRORS: usize = 8;
        const MAX_RANDOM_MIRRORS: usize = 64;

        let num_mirrors = rng.gen_range(MIN_RANDOM_MIRRORS..=MAX_RANDOM_MIRRORS);

        Self(
            iter::repeat_with(|| match rng.gen_range(0..mirror_types.len()) {
                0 => Box::new(mirror::plane::PlaneMirror::<3>::random(rng)),
                1 => Box::new(mirror::sphere::EuclideanSphereMirror::<3>::random(rng)) as Box<dyn JsonSerDyn>,
                _ => unreachable!(),
            })
            .take(num_mirrors)
            .collect(),
        )
    }
}

impl mirror::JsonSer for Dynamic3D {
    fn to_json(&self) -> Result<serde_json::Value, Box<dyn Error>> {

        let mut json = vec![];

        for mirror in self.0.iter() {
            json.push(serde_json::json!({
                "type": mirror.json_type_dyn(),
                "mirror": mirror.to_json()?,
            }));
        }

        Ok(serde_json::json!({
            "type": "[]dynamic",
            "mirror": json,
        }))
    }
}

fn generate_random_simulation(dim: usize) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    if dim == 2 {
        Simulation::<Dynamic2D, 2>::random(&mut rng).to_json()
    } else if dim == 3 {
        Simulation::<Dynamic3D, 3>::random(&mut rng).to_json()
    } else {
        Err("dimension must be 2 or 3".into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    let file_path = args
        .next()
        .ok_or("please provide a path to serialize the simulation json data")?;

    let dim = args
        .next()
        .ok_or("please provide a dimension (2 or 3) for the simulation")?
        .parse()?;

    let json = generate_random_simulation(dim)?;

    serde_json::to_writer_pretty(File::create(file_path)?, &json)?;

    Ok(())
}
