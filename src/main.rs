use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_filled_circle_mut;
use lerp::Lerp;
use noise::{self, NoiseFn, Perlin};
use palette::{IntoColor, Oklab, OklabHue, Oklch, Srgb};
use rand::random;
use serde::Deserialize;
use std::{env, error::Error, fs, path::Path};

const CONFIG: &str = "config.toml";

#[derive(Deserialize)]
struct Config {
    width: u32,
    height: u32,
    complexity: f64,
    particles: u32,
    thickness: i32,
    background: [f64; 3],
    randomness: f64,
    circle_sharpness: f64,
    circle: bool,
    luminance: [f64; 2],
    chroma: [f64; 2],
    hue_offset: [u16; 2],
    stripes: u64,
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
    let data: Config = toml::from_str(&data_string)?;

    for _ in 0..count {
        make_image(&data)?;
    }
    println!("Done!");

    Ok(())
}

fn make_image(data: &Config) -> MyResult {
    let background: Srgb<f64> =
        Srgb::new(data.background[0], data.background[1], data.background[2]);
    let step = 1.0 / f64::from(data.width.max(data.height));
    let noise = Perlin::new(random());
    //let color_noise = Perlin::new(random());
    let mut image = RgbaImage::new(data.width, data.height);
    image.pixels_mut().for_each(|val| {
        *val = Rgba([
            (background.red * 255.) as u8,
            (background.green * 255.) as u8,
            (background.blue * 255.) as u8,
            255,
        ])
    });

    let luminance1 = data.luminance[0].lerp(data.luminance[1], random::<f64>());
    let luminance2 = data.luminance[0].lerp(data.luminance[1], random::<f64>());
    let chroma1 = data.chroma[0].lerp(data.chroma[1], random::<f64>());
    let chroma2 = data.chroma[0].lerp(data.chroma[1], random::<f64>());
    let hue1: OklabHue<f64> = random();
    let hue2: OklabHue<f64> = OklabHue::from_degrees(
        hue1.into_degrees()
            + (-f64::from(data.hue_offset[0]).lerp(data.hue_offset[1] as f64, random::<f64>())),
    );
    let color1: Oklab<f64> = Oklch::new(luminance1, chroma1, hue1).into_color();
    let color2: Oklab<f64> = Oklch::new(luminance2, chroma2, hue2).into_color();

    for _ in 0..data.particles {
        let mut particle: (f64, f64) = (random(), random());

        let mut gen: u64 = 0;

        while particle.0 >= -0.2
            && particle.0 <= 1.2
            && particle.1 >= -0.2
            && particle.1 <= 1.2
            && gen < data.stripes
        {
            let mv = (-data.randomness).lerp(data.randomness, random::<f64>());
            let coords = (
                (particle.0 * f64::from(data.width)) as i32,
                (particle.1 * f64::from(data.height)) as i32,
            );
            let mult = 1.0 - (gen as f64 / data.stripes as f64);
            let color = get_color(&data, &color1, &color2, particle.0, particle.1, mult);
            draw_filled_circle_mut(&mut image, coords, data.thickness, Rgba(color));

            let (x, y) = (particle.0 * data.complexity, particle.1 * data.complexity);
            let direction =
                ((noise.get([x, y]) * 0.5 + 0.5) * std::f64::consts::PI * 2.0 + mv).sin_cos();
            particle.0 += direction.0 * step;
            particle.1 += direction.1 * step;
            gen += 1;
        }
    }

    let mut name_index = 0;
    while Path::new(&format!("{}.png", name_index)).is_file() {
        name_index += 1;
    }

    image.save(format!("{}.png", name_index))?;
    println!(
        "Saved image:\nColor1:\nL = {}\nc = {}\nh = {}\nColor2:\nL = {}\nc = {}\nh = {}",
        luminance1,
        chroma1,
        hue1.into_degrees(),
        luminance2,
        chroma2,
        hue2.into_degrees(),
    );
    Ok(())
}

fn get_color(
    data: &Config,
    color1: &Oklab<f64>,
    color2: &Oklab<f64>,
    x: f64,
    y: f64,
    gen: f64,
) -> [u8; 4] {
    let background: Srgb<f64> =
        Srgb::new(data.background[0], data.background[1], data.background[2]);
    let gen = gen; //(gen + 0.4).max(0.0);
    let l = color1.l.lerp(color2.l, gen);
    let a = color1.a.lerp(color2.a, gen);
    let b = color1.b.lerp(color2.b, gen);
    let color = Oklab::new(l, a, b);
    let rgb: Srgb<f64> = color.into_color();
    let distance = if data.circle {
        ((1.0 - ((0.5 - x).powi(2) + (0.5 - y).powi(2)).sqrt() * 2.) * data.circle_sharpness)
            .clamp(0.0, 1.0)
    } else {
        1.0
    };
    let mult = 0.0.lerp(1.6, gen).min(1.0) * distance;
    let r = (background.red.lerp(rgb.red, mult) * 255.) as u8;
    let g = (background.green.lerp(rgb.green, mult) * 255.) as u8;
    let b = (background.blue.lerp(rgb.blue, mult) * 255.) as u8;

    [r, g, b, 255]
}
