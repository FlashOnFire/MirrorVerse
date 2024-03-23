use gnuplot::{AxesCommon, Caption, Color, Figure, Fix};

use crate::mirror::{Mirror, Ray};
use crate::mirror::plane::PlaneMirror;

pub fn render_gnu_plot(rays: Vec<Ray>, mirrors: Vec<Box<dyn Mirror>>) -> Result<(), Box<dyn std::error::Error>> {
    let mut fg = Figure::new();
    let mut ax = fg.axes2d();

    for (i, ray) in rays.iter().enumerate() {
        if i == rays.len() - 1 {
            //draw a infinite line representing the ray
            let x_values: Vec<f64> = vec![ray.origin[0] as f64, ray.origin[0] as f64 + (ray.direction[0] * 10000.) as f64];
            let y_values: Vec<f64> = vec![ray.origin[1] as f64, ray.origin[1] as f64 + (ray.direction[1] * 10000.) as f64];
            ax.lines(&x_values, &y_values, &[Caption("Ray"), Color("blue")]);
        } else {
            let x_values: Vec<f64> = vec![ray.origin[0] as f64, rays[i + 1].origin[0] as f64];
            let y_values: Vec<f64> = vec![ray.origin[1] as f64, rays[i + 1].origin[1] as f64];
            ax.lines(&x_values, &y_values, &[Caption("Ray"), Color("blue")]);
        }
    }

    //convert mirros to only planemirrors
    let mut plane_mirrors: Vec<Box<PlaneMirror>> = Vec::new();
    for (_, mirror) in mirrors.iter().enumerate() {
        println!("{:?}", mirror.get_type());
        // if mirror.get_type() == "plane" {
        plane_mirrors.push(mirror.downcast().unwrap());
        // }
    }
    println!("{:?}", plane_mirrors);

    //plot the plane mirrors
    for mirror in plane_mirrors {
        let x_values: Vec<f64> = vec![mirror.plane.v_0().x as f64, (mirror.plane.v_0().x * mirror.bounds[1]).into()];
        let y_values: Vec<f64> = vec![mirror.plane.v_0().y as f64, (mirror.plane.v_0().y * mirror.bounds[1]).into()];
        ax.lines(&x_values, &y_values, &[Caption("Mirror"), Color("red")]);
    }


    //scale the figure not to show the long "infinite" ray
    ax.set_x_range(Fix(0.0), Fix(10.0));

    fg.show()?;
    Ok(())
}
