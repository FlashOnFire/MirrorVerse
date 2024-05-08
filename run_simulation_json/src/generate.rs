use std::{fs::File, io::Write};

use mirror_verse::mirror::{plane::PlaneMirror, sphere::EuclideanSphereMirror, Mirror, Ray};

pub fn generate_mirror_set(n_rays: usize, n_plane: usize, n_sphere: usize, file_path: String) {
    //TODO: racisme en vue sur le dimension 3
    //generate random mirrors
    let mut mirrors: Vec<Box<dyn Mirror<3>>> = Vec::new();
    for _ in 0..n_plane {
        let mirror: Box<dyn Mirror<3>> = Box::new(PlaneMirror::random());
        mirrors.push(mirror);
    }
    for _ in 0..n_sphere {
        let mirror: Box<dyn Mirror<3>> = Box::new(EuclideanSphereMirror::random());
        mirrors.push(mirror);
    }

    let rays: Vec<Ray<3>> = (0..n_rays).map(|_| Ray::random()).collect();
    let rays_json = rays
        .iter()
        .map(|ray| ray.to_json().unwrap())
        .collect::<Vec<_>>();

    //write to file
    let jsons = mirrors
        .iter()
        .map(|mirror| mirror.to_json().unwrap())
        .collect::<Vec<_>>();
    //SAVAGELY putting the jsons in one main json
    let mut file = File::create(file_path).unwrap();
    let json = serde_json::json!({
        "rays": rays_json,
        "mirror": {
            "type": "[]dynamic",
            "mirror": jsons
        }
    });
    file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
        .unwrap();
}
