use std::{env, error::Error, fs::File};

use mirror_verse::{mirror::Mirror, rand, serde_json, Simulation};

const DIM: usize = 3;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);

    let file_path = args
        .next()
        .ok_or("please provide a path to serialize the simulation json data")?;

    let mut rng = rand::thread_rng();

    let random_simulation = Simulation::<
        Vec<Box<dyn Mirror<DIM>>>,
        DIM,
    >::random(&mut rng);

    let type_erased_simulation = Simulation {
        mirror: Box::new(random_simulation.mirror) as Box<dyn Mirror<DIM>>,
        rays: random_simulation.rays,
    };

    let json = type_erased_simulation.to_json()?;

    serde_json::to_writer_pretty(File::create(file_path)?, &json)?;

    Ok(())
}
