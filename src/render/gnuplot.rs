use gnuplot::{AxesCommon, Caption, Color, Figure, Fix};

use crate::mirror::plane::PlaneMirror;
use crate::mirror::Ray;

pub fn render_gnu_plot(fg: &mut Figure, rays: &[Ray], mirrors: &[PlaneMirror]) {
    let mut ax = fg.axes2d();

    // Note that slice::array_windows is experimental
    for [ray, next_ray] in rays
        .windows(2)
        .map(TryFrom::try_from)
        .filter_map(Result::<&[Ray; 2], _>::ok)
    {
        let x_values: Vec<f64> = vec![ray.origin[0] as f64, next_ray.origin[0] as f64];
        let y_values: Vec<f64> = vec![ray.origin[1] as f64, next_ray.origin[1] as f64];
        ax.lines(&x_values, &y_values, &[Caption("Ray"), Color("blue")]);
    }

    let last_ray = rays.last().expect("at least one ray expected");

    //draw a infinite line representing the ray
    let x_values: Vec<f64> = vec![
        last_ray.origin[0] as f64,
        last_ray.origin[0] as f64 + (last_ray.direction[0] * 10000.) as f64,
    ];
    let y_values: Vec<f64> = vec![
        last_ray.origin[1] as f64,
        last_ray.origin[1] as f64 + (last_ray.direction[1] * 10000.) as f64,
    ];
    ax.lines(&x_values, &y_values, &[Caption("Ray"), Color("blue")]);

    //plot the plane mirrors
    for mirror in mirrors {
        let x_values = [
            (mirror.plane.v_0().x - mirror.plane.basis()[0].x * mirror.bounds[1]) as f64,
            (mirror.plane.v_0().x + mirror.plane.basis()[0].x * mirror.bounds[1]).into(),
        ];
        let y_values = [
            (mirror.plane.v_0().y - mirror.plane.basis()[0].y * mirror.bounds[1]) as f64,
            (mirror.plane.v_0().y + mirror.plane.basis()[0].y * mirror.bounds[1]).into(),
        ];
        ax.lines(&x_values, &y_values, &[Caption("Mirror"), Color("red")]);
    }

    //scale the figure not to show the long "infinite" ray
    ax.set_x_range(Fix(0.0), Fix(10.0));
}
