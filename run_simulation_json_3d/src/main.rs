use mirror_verse::{mirror::Mirror, serde_json, Simulation};
use std::fs::File;

const DEFAULT_DIM: usize = 3;

fn main() {
    let mut args = std::env::args().skip(1);

    let file_path = args
        .next()
        .expect("expected a file path as a first argument.");

    // TODO: Show help menu on failure
    // let Some(file_path) = ... else {
    //    // display error
    //    // show help menu
    //    return;
    // }

    let max_num_reflections = args
        .next()
        .map(|arg| arg.parse().expect("expected a number as second argument"))
        .unwrap_or(500);

    let simulation = Simulation::<Box<dyn Mirror<DEFAULT_DIM>>, DEFAULT_DIM>::from_json(
        &serde_json::from_reader(File::open(file_path).unwrap()).unwrap(),
    )
    .unwrap();

    // TODO: Show help menu on failure
    // let Ok(simulation) = ... else {
    //    // display error
    //    // show help menu
    //    return;
    // }

    simulation.run_opengl(max_num_reflections);
}
