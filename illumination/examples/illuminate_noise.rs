#![feature(unwrap_infallible)]
#![feature(exit_status_error)]

use std::{
    f64::consts::PI,
    ffi::OsString,
    hash::{DefaultHasher, Hash, Hasher},
    io::{Cursor, Write},
    num::NonZeroUsize,
    path::PathBuf,
    process::Stdio,
};

use clap::{Parser, Subcommand};
use field::Field;
use illumination::{Illumination, SunSetup};
use image::{GrayImage, Luma};
use nalgebra::{point, vector};
use noise::{NoiseFn, Simplex};
use rand::{thread_rng, Rng};
use sim_time::{humanized::Humanized, Duration, Time};

#[derive(Debug, Parser, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(long)]
    /// Seed for the noise
    seed: Option<OsString>,

    #[clap(long, short = 'y', default_value = "128")]
    /// Vertical size of the map
    tile_y: f64,
    #[clap(long, short = 'x', default_value = "128")]
    /// Orizontal size of the map
    tile_x: f64,
    #[clap(long, short, default_value = "1")]
    /// Resolution of the map
    res: f64,

    #[clap(long, short, default_value = "3")]
    /// Scale of the features in the map
    scale: f64,

    #[clap(long, default_value = "1d", value_parser = |s:&str| s.parse::<Humanized<Duration>>().map(Humanized::inner))]
    /// Lenght of one day
    day_lenght: Duration,
    #[clap(long, default_value = "45")]
    /// Latitude, in degrees
    latitude: f64,
    #[clap(long, default_value = "615.15")]
    /// Solar constant, in W/m^2
    solar_constant: f64,
    #[clap(long, default_value = "10")]
    /// Ambient illumination, in percentage of total energy flux
    ambient: f64,

    #[clap(long, short)]
    /// Output file for the noise map
    noise_map: Option<PathBuf>,

    #[clap(long, default_value = "2")]
    /// Resolution of the output, in dots for unit
    dpu: NonZeroUsize,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Photograph a single illuminated frame
    Frame {
        #[clap(long, short, default_value = "0", value_parser = |s:&str| s.parse::<Humanized<Time>>().map(Humanized::inner))]
        /// Time at which the calculation must be done
        time: Time,
        /// Output file for the illumination map
        illuminate_map: PathBuf,
    },

    /// Generate a mp4 video of a whole day, from sunset to sunset
    Video {
        #[clap(long, short, default_value = "25")]
        /// Framerate of the video
        framerate: u8,

        #[clap(long, short, default_value = "15")]
        /// Lenght of the video in seconds
        lenght: f64,

        /// Output video
        illuminate_video: PathBuf,
    },
}

fn main() {
    let Args {
        seed,
        tile_y,
        tile_x,
        res,
        scale,
        day_lenght,
        latitude,
        solar_constant,
        noise_map,
        ambient,
        dpu,
        command,
    } = Args::parse();

    let seed = if let Some(seed) = seed {
        let mut s = DefaultHasher::new();
        seed.hash(&mut s);
        (s.finish() & (u32::MAX as u64)) as u32
    } else {
        thread_rng().gen()
    };
    let noise = noise::ScalePoint::new(noise::Multiply::new(
        Simplex::new(seed),
        noise::Constant::new(scale),
    ))
    .set_scale(scale);

    let map = Field::new_from_fun(tile_x, tile_y, res, |pos| {
        // conversion to normalized coordinates
        let u = pos.x / tile_x;
        let v = pos.y / tile_y;
        // conversion to toroidal 4d coordinates
        let u = vector![(u * 2. * PI).cos(), (u * 2. * PI).sin()];
        let v = vector![(v * 2. * PI).cos(), (v * 2. * PI).sin()];
        // zooming of the torus to make features the right size
        let u = u / 2.;
        let v = v / 2.;
        // sampling the 4D noise map
        noise.get([u.x, u.y, v.x, v.y])
    });

    if let Some(noise_map) = noise_map {
        img_from_map(&map, dpu)
            .save(noise_map)
            .expect("Cannot save map image")
    }

    let illumination = Illumination::new(SunSetup {
        day_lenght,
        latitude,
        solar_constant,
        ambient,
    })
    .into_ok();

    match command {
        Command::Frame {
            time,
            illuminate_map,
        } => {
            let illuminated = illumination.illuminate(&map, time);
            img_from_illumination(&illuminated, illumination.solar_constant, dpu)
                .save(illuminate_map)
                .expect("Cannot save map image")
        }
        Command::Video {
            framerate,
            lenght,
            illuminate_video,
        } => {
            let frames = (framerate as f64 * lenght) as u64;

            let mut ffmpeg = std::process::Command::new("ffmpeg")
                .arg("-y")
                .args(["-f", "image2pipe"])
                .args(["-c:v", "bmp"])
                .arg("-framerate")
                .arg(format!("{}", framerate))
                .args(["-i", "-"])
                .args(["-c:v", "libx264"])
                .args(["-pix_fmt", "yuv420p"])
                .arg("-r")
                .arg(format!("{}", framerate))
                .arg(illuminate_video)
                .stdin(Stdio::piped())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Cannot launch ffmpeg");
            let mut frame_buffer = Vec::<u8>::new();

            for f in 0..frames {
                let time = Time::ZERO - illumination.day_lenght * 0.5
                    + (illumination.day_lenght * f) / frames;

                let illuminated = illumination.illuminate(&map, time);

                // sending to ffmpeg
                frame_buffer.clear();
                img_from_illumination(&illuminated, solar_constant, dpu)
                    .write_to(&mut Cursor::new(&mut frame_buffer), image::ImageFormat::Bmp)
                    .expect("Cannot save frame as image");
                ffmpeg
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(&frame_buffer)
                    .expect("Cannot write frame to ffmpeg pipe");
            }

            ffmpeg
                .wait()
                .unwrap()
                .exit_ok()
                .expect("Ffmpeg closed with error.")
        }
    }
}

/// Generate an image from a grid of elevations
fn img_from_map(map: &Field<f64>, dpu: NonZeroUsize) -> GrayImage {
    let min = map.min_by(f64::total_cmp);
    let max = map.max_by(f64::total_cmp);

    let res_x = (map.tile_x() * dpu.get() as f64) as u32;
    let res_y = (map.tile_y() * dpu.get() as f64) as u32;

    GrayImage::from_fn(res_x, res_y, |x, y| {
        let x = map.tile_x() * (x as f64 / res_x as f64);
        let y = map.tile_y() * (1. - y as f64 / res_y as f64);

        let value = (u8::MAX as f64 * (map.value(point![x, y]) - min) / (max - min)) as u8;
        Luma([value])
    })
}

/// Generate an image from a grid of illumintions
fn img_from_illumination(
    illumination: &Field<f64>,
    solar_constant: f64,
    dpu: NonZeroUsize,
) -> GrayImage {
    let res_x = (illumination.tile_x() * dpu.get() as f64) as u32;
    let res_y = (illumination.tile_y() * dpu.get() as f64) as u32;

    GrayImage::from_fn(res_x, res_y, |x, y| {
        let x = illumination.tile_x() * (x as f64 / res_x as f64);
        let y = illumination.tile_y() * (1. - y as f64 / res_y as f64);

        let value = (u8::MAX as f64 * (illumination.value(point![x, y]) / solar_constant)) as u8;
        Luma([value])
    })
}
