use core::iter;
use std::{env, error::Error, fs::File};

use mirror_verse::{
    mirror::{self, Random},
    rand, serde_json, Simulation,
};

const NUM_MIRROR_TYPES_2D: usize = 2;
const NUM_MIRROR_TYPES_3D: usize = 3;
const MIN_RANDOM_MIRRORS: usize = 8;
const MAX_RANDOM_MIRRORS: usize = 64;

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

impl Dynamic2D {
    fn random<T: rand::Rng + ?Sized>(num_mirrors: usize, rng: &mut T) -> Self {
        Self(
            iter::repeat_with(|| match rng.gen_range(0..NUM_MIRROR_TYPES_2D) {
                0 => Box::new(mirror::plane::PlaneMirror::<2>::random(rng)) as Box<dyn JsonSerDyn>,
                1 => Box::new(mirror::sphere::EuclideanSphereMirror::<2>::random(rng)),
                _ => unreachable!(),
            })
            .take(num_mirrors)
            .collect(),
        )
    }
}

impl mirror::Random for Dynamic2D {
    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized,
    {
        let num_mirrors = rng.gen_range(MIN_RANDOM_MIRRORS..=MAX_RANDOM_MIRRORS);

        Self::random(num_mirrors, rng)
    }
}

impl mirror::JsonSer for Dynamic2D {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "[]dynamic",
            "mirror": Vec::from_iter(
                self.0.iter().map(|mirror| {
                    serde_json::json!({
                        "type": mirror.json_type_dyn(),
                        "mirror": mirror.to_json(),
                    })
                })
            ),
        })
    }
}

// copy paste lol

struct Dynamic3D(Vec<Box<dyn JsonSerDyn>>);

impl Dynamic3D {
    fn random<T: rand::Rng + ?Sized>(num_mirrors: usize, rng: &mut T) -> Self {
        Self(
            iter::repeat_with(|| match rng.gen_range(0..NUM_MIRROR_TYPES_3D) {
                0 => Box::new(mirror::plane::PlaneMirror::<3>::random(rng)) as Box<dyn JsonSerDyn>,
                1 => Box::new(mirror::sphere::EuclideanSphereMirror::<3>::random(rng)),
                2 => Box::new(mirror::cylinder::CylindricalMirror::random(rng)),
                _ => unreachable!(),
            })
            .take(num_mirrors)
            .collect(),
        )
    }
}

impl mirror::Random for Dynamic3D {
    fn random<T: rand::Rng + ?Sized>(rng: &mut T) -> Self
    where
        Self: Sized,
    {
        let num_mirrors = rng.gen_range(MIN_RANDOM_MIRRORS..=MAX_RANDOM_MIRRORS);

        Self::random(num_mirrors, rng)
    }
}

impl mirror::JsonSer for Dynamic3D {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "[]dynamic",
            "mirror": Vec::from_iter(
                self.0.iter().map(|mirror| {
                    serde_json::json!({
                        "type": mirror.json_type_dyn(),
                        "mirror": mirror.to_json(),
                    })
                })
            ),
        })
    }
}

fn generate_random_simulation(
    dim: usize,
    num_mirrors: usize,
    num_rays: usize,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    if dim == 2 {
        Ok(Simulation {
            mirror: Dynamic2D::random(num_mirrors, &mut rng),
            rays: iter::repeat_with(|| mirror::Ray::<2>::random(&mut rng))
                .take(num_rays)
                .collect(),
        }
        .to_json())
    } else if dim == 3 {
        Ok(Simulation {
            mirror: Dynamic3D::random(num_mirrors, &mut rng),
            rays: iter::repeat_with(|| mirror::Ray::<3>::random(&mut rng))
                .take(num_rays)
                .collect(),
        }
        .to_json())
    } else {
        Err("dimension must be 2 or 3".into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    let file_path = args
        .next()
        .ok_or("please provide a path to serialize the simulation json data")?;

    let dim = args.next().and_then(|arg| arg.parse().ok()).unwrap_or(2);

    let num_mirrors = args.next().and_then(|arg| arg.parse().ok()).unwrap_or(12);

    let num_rays = args.next().and_then(|arg| arg.parse().ok()).unwrap_or(4);

    let json = generate_random_simulation(dim, num_mirrors, num_rays)?;

    serde_json::to_writer_pretty(File::create(file_path)?, &json)?;

    Ok(())
}
