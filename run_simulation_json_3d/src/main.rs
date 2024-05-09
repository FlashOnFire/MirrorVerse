use mirror_verse::{mirror::Mirror, serde_json, Simulation};
use std::{error::Error, fs::File};

const DEFAULT_DIM: usize = 3;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);

    let file_path = args
        .next()
        .ok_or("expected a file path as a first argument.")?;

    let max_num_reflections = args
        .next()
        .map(|arg| arg.parse().expect("expected a number as second argument"))
        .unwrap_or(500);

    let simulation = Simulation::<Box<dyn Mirror<DEFAULT_DIM>>, DEFAULT_DIM>::from_json(
        &serde_json::from_reader(File::open(file_path)?)?,
    )?;

    simulation.run_opengl(max_num_reflections);

    Ok(())
}
