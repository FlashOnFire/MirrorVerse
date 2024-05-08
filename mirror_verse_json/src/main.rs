use mirror_verse::{mirror::Mirror, serde_json, Simulation};
use std::fs::File;

//je sais pas ce que je fait mais bon ca passe
mod generate;
use generate::generate_mirror_set;

const DEFAULT_DIM: usize = 2;

fn main() {
    let args = std::env::args().skip(1);
    let args_vec: Vec<String> = args.collect(); // Convert args iterator into a Vec
                                                //fetch the params
    let mut args_flag: Vec<String> = Vec::new();
    let mut args_no_flag: Vec<String> = Vec::new();

    // RACISME RACISME RACISME
    let mut droping: bool = false;
    for i in 0..args_vec.len() {
        if !droping {
            if args_vec[i].starts_with('-') {
                args_flag.push(args_vec[i].clone());
                //if theres no = in the arg, then the next arg is a value (generate is an exception to this rule)
                if !args_vec[i].contains('=') && args_vec[i] != "-g" && args_vec[i] != "--generate"
                {
                    args_flag.push(args_vec[i + 1].clone());
                    droping = true;
                }
            } else {
                args_no_flag.push(args_vec[i].clone());
            }
        } else {
            droping = false;
        }
    }
    drop(args_vec);

    // Check if the help flag is present in the arguments
    if args_flag.contains(&"-h".to_string()) || args_flag.contains(&"--help".to_string()) {
        println!("Usage: mirror_verse_json <file_path> [max_num_reflections]");
        println!("       mirror_verse_json [-g | --generate] [-p | --plane [num_mirrors]] [-s | --sphere [num_mirrors]] [-r | --ray [num_rays]] <file_path> [max_num_reflections]");
        return;
    }

    let file_path: String = match args_no_flag.first() {
        Some(path) => path.to_string(),
        None => {
            println!("Expected a file path as the first non param argument.");
            return;
        }
    };

    if args_flag.contains(&"-g".to_string()) || args_flag.contains(&"--generate".to_string()) {
        println!("Generating mirror set...");
        let n_plane = args_flag
            .clone()
            .into_iter()
            .skip_while(|arg| arg != "-p" && arg != "--plane")
            .nth(1)
            .map(|arg| arg.parse().unwrap_or(0))
            .unwrap_or(0);
        let n_sphere = args_flag
            .clone()
            .into_iter()
            .skip_while(|arg| arg != "-s" && arg != "--sphere")
            .nth(1)
            .map(|arg| arg.parse().unwrap_or(0))
            .unwrap_or(0);
        let n_rays = args_flag
            .into_iter()
            .skip_while(|arg| arg != "-r" && arg != "--ray")
            .nth(1)
            .map(|arg| arg.parse().unwrap_or(1))
            .unwrap_or(1);

        println!(
            "n_plane: {}, n_sphere: {}, n_rays: {} file: {}",
            n_plane, n_sphere, n_rays, file_path
        );
        generate_mirror_set(n_rays, n_plane, n_sphere, file_path.to_string());
    }

    let max_num_reflections = match args_no_flag.get(1) {
        Some(arg) => match arg.parse::<usize>() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid number provided, using default value.");
                500
            }
        },
        None => 500,
    };

    let simulation = Simulation::<Box<dyn Mirror<DEFAULT_DIM>>, DEFAULT_DIM>::from_json(
        &serde_json::from_reader(File::open(file_path).unwrap()).unwrap(),
    )
    .unwrap();

    simulation.run_opengl(max_num_reflections);
}
