use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_filled_circle_mut;
use noise::{self, NoiseFn, Perlin};
use rand::random;
use serde::Deserialize;
use std::{env, error::Error, f64::consts::PI, fs, path::Path};

const CONFIG: &str = "config.toml";

#[derive(Deserialize)]
struct Data {
    width: u32,
    height: u32,
    complexity: f64,
    color_complexity: f64,
    min_color: f64,
    particles: u32,
    thickness: i32,
    circle: bool,
}

type MyResult = Result<(), Box<dyn Error>>;

fn main() -> MyResult {
    let args: Vec<String> = env::args().collect();
    let count: u32 = args.get(1).map(|val| val.as_str()).unwrap_or("1").parse()?;

    if !Path::new(CONFIG).is_file() {
        fs::write(CONFIG, include_str!("default.toml"))?;
        println!("Conig created...");
        return Ok(());
    }
    let data_string = fs::read_to_string(CONFIG)?;
    let data: Data = toml::from_str(&data_string)?;

    for _ in 0..count {
        make_image(&data)?;
    }
    println!("Done!");

    Ok(())
}

fn make_image(data: &Data) -> MyResult {
    /*//  Color generation
    let r: (u8, u8) = (random(), random());
    let g: (u8, u8) = (random(), random());
    let b: (u8, u8) = (random(), random());
    let r1 = r.0.min(r.1);
    let r2 = r.0.max(r.1);
    let rr = r2 - r1;
    let g1 = g.0.min(g.1);
    let g2 = g.0.max(g.1);
    let gr = g2 - g1;
    let b1 = b.0.min(b.1);
    let b2 = b.0.max(b.1);
    let br = b2 - b1;*/

    let step = 1.0 / f64::from(data.width.max(data.height));
    let noise = Perlin::new(random());
    let red = Perlin::new(random());
    let green = Perlin::new(random());
    let blue = Perlin::new(random());
    let mut image = RgbaImage::new(data.width, data.height);
    image
        .pixels_mut()
        .for_each(|val| *val = Rgba([0, 0, 0, 255]));

    for _ in 0..data.particles {
        let mut particle: (f64, f64) = (random(), random());

        while particle.0 >= -0.2 && particle.0 <= 1.2 && particle.1 >= -0.2 && particle.1 <= 1.2 {
            let coords = (
                (particle.0 * f64::from(data.width)) as i32,
                (particle.1 * f64::from(data.height)) as i32,
            );
            let color = [
                get_color(&red, &data, particle.0, particle.1),
                get_color(&green, &data, particle.0, particle.1),
                get_color(&blue, &data, particle.0, particle.1),
                255,
            ];
            draw_filled_circle_mut(&mut image, coords, data.thickness, Rgba(color));

            let (x, y) = (particle.0 * data.complexity, particle.1 * data.complexity);
            let direction = (noise.get([x, y]) * PI * 2.0).sin_cos();
            particle.0 += direction.0 * step;
            particle.1 += direction.1 * step;
        }
    }

    let mut name_index = 0;
    while Path::new(&format!("{}.png", name_index)).is_file() {
        name_index += 1;
    }

    image.save(format!("{}.png", name_index))?;
    println!("Saved image!");
    Ok(())
}

fn get_color(noise: &Perlin, data: &Data, x: f64, y: f64) -> u8 {
    let color = (((noise.get([x * data.color_complexity, y * data.color_complexity]) + 1.) / 2.)
        + data.min_color)
        .clamp(0.2, 1.0);
    let distance = if data.circle {
        1. - ((0.5 - x).powi(2) + (0.5 - y).powi(2)).sqrt() * 2.
    } else {
        1.0
    };
    let color = (color * distance * 255.).clamp(0., 255.);
    color as u8
}
