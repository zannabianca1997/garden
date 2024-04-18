use std::{
    f64::consts::PI,
    ffi::{OsStr, OsString},
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    num::NonZeroU32,
    path::PathBuf,
};

use clap::Parser;
use field::Field;
use image::{GrayImage, Luma, Rgb, RgbImage};
use nalgebra::{point, vector, Matrix3, Point, Point2, Vector2, Vector3};
use noise::{core::value, NoiseFn, Simplex};
use rand::{thread_rng, Rng};

#[derive(Debug, Parser, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(long)]
    /// Seed for the noise
    seed: Option<OsString>,

    #[clap(long, short = 'x', default_value = "16")]
    /// Width of the noise tiling
    tile_x: f64,
    #[clap(long, short = 'y', default_value = "9")]
    /// Height of the noise tiling
    tile_y: f64,
    #[clap(long, short, default_value = "0.2")]
    /// Sampling resolution
    res: f64,

    #[clap(long, short = 'X', default_value = "1600")]
    /// Output width in pixels
    res_x: NonZeroU32,
    #[clap(long, short = 'Y', default_value = "900")]
    /// Output height in pixels
    res_y: NonZeroU32,
    #[clap(long, short, default_value = "100")]
    /// Output scale in pixel/unit
    scale: NonZeroU32,

    /// Output file
    output: PathBuf,
}

fn main() {
    let Args {
        seed,
        tile_x,
        tile_y,
        res,
        res_x,
        res_y,
        scale,
        output,
    } = Args::parse();

    // wrapping the noise in a closure
    let noise = {
        let seed = if let Some(seed) = seed {
            let mut s = DefaultHasher::new();
            seed.hash(&mut s);
            (s.finish() & (u32::MAX as u64)) as u32
        } else {
            thread_rng().gen()
        };
        let source = Simplex::new(seed);
        let tile_phi = tile_x / (2. * PI);
        let tile_psi = tile_y / (2. * PI);
        move |pos: Point2<f64>| -> f64 {
            let phi = pos.x / tile_phi;
            let psi = pos.y / tile_psi;
            source.get([
                phi.cos() * tile_phi,
                phi.sin() * tile_phi,
                psi.cos() * tile_psi,
                psi.sin() * tile_psi,
            ])
        }
    };
    let noise = |pos: Point2<f64>| (2. * PI * pos.x).cos();

    // sampling the noise and generating our tessellation
    let field = Field::new_from_fun(tile_x, tile_y, res, noise);

    let min = dbg!(field.min_by(f64::total_cmp));
    let max = dbg!(field.max_by(f64::total_cmp));

    let sun_pos = vector![0., -1. / 2f64.sqrt(), 1. / 2f64.sqrt()];

    // Generating the tessellated output

    let image = GrayImage::from_fn(res_x.get(), res_y.get(), |i, j| {
        // sampling the field
        let value = field.value(point![i as f64, (res_y.get() - j) as f64] / scale.get() as f64);

        // rescaling into the grayscale
        let value = (value - min) / (max - min);
        let value = (value * u8::MAX as f64) as u8;

        Luma([value])
    });

    // saving the output
    image.save(&output).expect("Cannot save output");

    // Direct noise for comparison

    let image = GrayImage::from_fn(res_x.get(), res_y.get(), |i, j| {
        // sampling the noise
        let value = noise(point![i as f64, (res_y.get() - j) as f64] / scale.get() as f64);

        // rescaling into the grayscale
        let value = (value - min) / (max - min);
        let value = (value * u8::MAX as f64) as u8;

        Luma([value])
    });

    // saving the output
    image
        .save(output.with_file_name({
            let mut name = output.file_stem().unwrap_or(OsStr::new("")).to_owned();
            name.push(".src.");
            name.push(output.extension().unwrap_or(OsStr::new("")));
            name
        }))
        .expect("Cannot save output");

    // Normal map

    let image = GrayImage::from_fn(res_x.get(), res_y.get(), |i, j| {
        // sampling the normals
        let normal = field.normal(point![i as f64, (res_y.get() - j) as f64] / scale.get() as f64);
        let value = normal.dot(&sun_pos).clamp(0., 1.);

        // rescaling into the grayscale
        let value = (value * u8::MAX as f64) as u8;

        Luma([value])
    });

    // saving the output
    image
        .save(output.with_file_name({
            let mut name = output.file_stem().unwrap_or(OsStr::new("")).to_owned();
            name.push(".nrm.");
            name.push(output.extension().unwrap_or(OsStr::new("")));
            name
        }))
        .expect("Cannot save output");

    // Raycasted map

    let camera = point![0., 0., 10.];
    let point_to = point![10., 0., 0.];
    let fov = 40.;

    {
        let fov = (fov / 90. * PI).tan();
        let px = 2. * fov / res_x.get() as f64;

        let camera_direction = (point_to - camera).normalize();
        let camera_right = camera_direction.cross(&Vector3::z()).normalize();
        let camera_up = camera_right.cross(&camera_direction);
        let from_camera_to_world =
            Matrix3::from_columns(&[camera_right, camera_up, camera_direction]);

        dbg!(from_camera_to_world * Vector3::x());
        dbg!(from_camera_to_world * Vector3::y());
        dbg!(from_camera_to_world * Vector3::z());

        let caster = field.raycaster();

        let image = RgbImage::from_fn(res_x.get(), res_y.get(), |i, j| {
            // sampling the normals
            let dir = (from_camera_to_world
                * vector![
                    (i as i32 - res_x.get() as i32 / 2) as f64 * px,
                    (res_y.get() as i32 / 2 - j as i32) as f64 * px,
                    1.
                ])
            .normalize();

            let Some(hit) = caster.cast(camera, dir) else {
                return Rgb([21, 148, 207]);
            };

            /*
            let sunny = if let Some(_) = caster.cast(hit + 0.001 * Vector3::z(), sun_pos) {
                // It's in the shade
                0.
            } else {
                field.normal(hit.xy()).dot(&sun_pos)
            }
            .clamp(0.3, 1.);

            let rgb = sunny * vector![53., 115., 42.];
            Rgb([rgb.x as u8, rgb.y as u8, rgb.x as u8])
            */

            let value = (hit.z - min) / (max - min);
            let value = (value * u8::MAX as f64) as u8;

            Rgb([value; 3])
        });

        image
            .save(output.with_file_name({
                let mut name = output.file_stem().unwrap_or(OsStr::new("")).to_owned();
                name.push(".ray.");
                name.push(output.extension().unwrap_or(OsStr::new("")));
                name
            }))
            .expect("Cannot save output");
    }
}
