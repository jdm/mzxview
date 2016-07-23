extern crate image;
extern crate libmzx;

use image::{RgbImage, DynamicImage, ImageFormat};
use libmzx::{load_world, World};
use std::env;
use std::fs::File;
use std::io::Read;

fn print_usage() {
    println!("mzxview world.mzx out.png");
}

fn draw_img(_w: &World) -> Option<RgbImage> {
    RgbImage::from_raw(1, 1, vec![0,0,0])
}

fn main() {
    let mut args = env::args();
    let _binary = args.next();
    let world_file = match args.next() {
        Some(path) => path,
        None => return print_usage(),
    };
    let image_path = match args.next() {
        Some(path) => path,
        None => return print_usage(),
    };

    let world_data = match File::open(&world_file) {
        Ok(mut file) => {
            let mut v = vec![];
            file.read_to_end(&mut v).unwrap();
            v
        }
        Err(e) => {
            println!("Error opening {} ({})", world_file, e);
            return;
        }
    };

    let world = match load_world(&world_data) {
        Ok(world) => world,
        Err(e) => {
            println!("Error reading {} ({:?})", world_file, e);
            return;
        }
    };

    match File::create(&image_path) {
        Ok(mut file) => {
            let img = match draw_img(&world) {
                Some(img) => img,
                None => {
                    println!("Error creating image from pixel buffer");
                    return;
                }
            };
            let dynamic_image = DynamicImage::ImageRgb8(img);
            if let Err(e) = dynamic_image.save(&mut file, ImageFormat::PNG) {
                println!("Failed to save {} ({}).", image_path, e);
                return;
            }
        }
        Err(e) => {
            println!("Error creating {} ({})", image_path, e);
            return;
        }
    }
}
