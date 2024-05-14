use mirror_verse::{mirror, render, serde_json, Simulation};
use std::{error::Error, format as f, fs::File};

trait SimulationMirror2D: mirror::Mirror<2> + render::OpenGLRenderable {}

trait SimulationMirror3D: mirror::Mirror<3> + render::OpenGLRenderable {}

impl<T: mirror::Mirror<2> + render::OpenGLRenderable + ?Sized> SimulationMirror2D for T {}

impl<T: mirror::Mirror<3> + render::OpenGLRenderable + ?Sized> SimulationMirror3D for T {}

impl mirror::JsonDes for Box<dyn SimulationMirror2D> {
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

        fn into_type_erased<T: SimulationMirror2D + 'static>(
            mirror: T,
        ) -> Box<dyn SimulationMirror2D> {
            Box::new(mirror) as _
        }

        match mirror_type {
            "plane" => mirror::plane::PlaneMirror::<2>::from_json(mirror).map(into_type_erased),
            "sphere" => {
                mirror::sphere::EuclideanSphereMirror::<2>::from_json(mirror).map(into_type_erased)
            }
            "dynamic" => Box::<dyn SimulationMirror2D>::from_json(mirror).map(into_type_erased),
            other => {
                // flatten nested lists
                if let Some(inner) = {
                    let inner = other.trim_start_matches("[]");
                    (other != inner).then_some(inner)
                } {
                    match inner {
                        "plane" => {
                            Vec::<mirror::plane::PlaneMirror<2>>::from_json(mirror).map(into_type_erased)
                        }
                        "sphere" => Vec::<mirror::sphere::EuclideanSphereMirror<2>>::from_json(mirror)
                            .map(into_type_erased),
                        "dynamic" => {
                            Vec::<Box<dyn SimulationMirror2D>>::from_json(mirror).map(into_type_erased)
                        }
                        _ => Err(f!("invalid mirror type :{other}").into()),
                    }
                } else {
                    Err(f!("invalid mirror type :{other}").into())
                }
            }
        }
    }
}

// copy paste lol

impl mirror::JsonDes for Box<dyn SimulationMirror3D> {
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

        println!("{mirror_type}");

        let mirror = json.get("mirror").ok_or("Missing mirror data")?;

        fn into_type_erased<T: SimulationMirror3D + 'static>(
            mirror: T,
        ) -> Box<dyn SimulationMirror3D> {
            Box::new(mirror) as _
        }

        match mirror_type {
            "plane" => mirror::plane::PlaneMirror::<3>::from_json(mirror).map(into_type_erased),
            "sphere" => {
                mirror::sphere::EuclideanSphereMirror::<3>::from_json(mirror).map(into_type_erased)
            }
            "dynamic" => Box::<dyn SimulationMirror3D>::from_json(mirror).map(into_type_erased),
            other => {
                // flatten nested lists
                if let Some(inner) = {
                    let inner = other.trim_start_matches("[]");
                    (other != inner).then_some(inner)
                } {
                    match inner {
                        "plane" => {
                            Vec::<mirror::plane::PlaneMirror<3>>::from_json(mirror).map(into_type_erased)
                        }
                        "sphere" => Vec::<mirror::sphere::EuclideanSphereMirror<3>>::from_json(mirror)
                            .map(into_type_erased),
                        "dynamic" => {
                            Vec::<Box<dyn SimulationMirror3D>>::from_json(mirror).map(into_type_erased)
                        }
                        _ => Err(f!("invalid mirror type :{other}").into()),
                    }
                } else {
                    Err(f!("invalid mirror type :{other}").into())
                }
            }
        }
    }
}

fn run_simulation(reflection_cap: usize, json: &serde_json::Value) -> Result<(), Box<dyn Error>> {
    let dim = json
        .get("dim")
        .ok_or(r#"invalid json: expected a "dim" field"#)?
        .as_u64()
        .ok_or(r#""dim" field must be a number"#)?;

    if dim == 2 {
        Simulation::<Box<dyn SimulationMirror2D>, 2>::from_json(json)?
            .run_opengl_3d(reflection_cap);
    } else if dim == 3 {
        Simulation::<Box<dyn SimulationMirror3D>, 3>::from_json(json)?
            .run_opengl_3d(reflection_cap);
    } else {
        panic!("dimension must be 2 or 3");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);

    let file_path = args
        .next()
        .ok_or("expected a file path as a first argument.")?;

    let max_num_reflections = args
        .next()
        .map(|arg| arg.parse().expect("expected a number as second argument"))
        .unwrap_or(5000);

    run_simulation(
        max_num_reflections,
        &serde_json::from_reader(File::open(file_path)?)?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_detection() {
        let simulation =
            Simulation::<Box<dyn SimulationMirror3D>, 3>::from_json(&serde_json::json!(
                {
                    "mirror": {
                        "type": "[]plane",
                        "mirror": [
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
        assert!(path.first().unwrap().all_points_raw().len() == 4);
    }

    #[test]
    fn test_no_loop_detection() {
        let simulation = Simulation::<Box<dyn SimulationMirror3D>, 3>::from_json(
            &include_str!("../../assets/diamond_of_hell.json")
                .parse()
                .expect("invalid json in assets/diamond_of_hell.json"),
        )
        .unwrap();

        let path = simulation.get_ray_paths(100);
        assert!(path.first().unwrap().all_points_raw().len() == 101);
    }
}
