extern crate image;
extern crate libmzx;

use image::{RgbImage, DynamicImage, ImageFormat};
use libmzx::{load_world, World, Charset, Palette};
use std::env;
use std::fs::File;
use std::io::Read;

fn print_usage() {
    println!("mzxview world.mzx out.png");
}

fn draw_char(ch: u8,
             fg_color: u8,
             bg_color: u8,
             x: usize,
             y: usize,
             width: usize,
             charset: &Charset,
             palette: &Palette,
             pixels: &mut Vec<u8>) {
    let stride = width * 3;
    let mut idx = y * stride + x;
    let char_bytes = charset.nth(ch);
    for byte in char_bytes {
        for bit in 0..8 {
            let color = if byte & (1 << bit) != 0 {
                &palette.colors[fg_color as usize]
            } else {
                &palette.colors[bg_color as usize]
            };
            let mut pixel = &mut pixels[idx*3+bit..(idx+1)*3+bit];
            pixel[0] = color.r * 4;
            pixel[1] = color.g * 4;
            pixel[2] = color.b * 4;
        }
        idx += stride;
    }
}

fn draw_img(w: &World) -> Option<RgbImage> {
    let board = &w.boards[0];
    let charset = &w.charset;
    let palette = &w.palette;

    let size = board.width * 8 * board.height * 14 * 3;
    let mut pixels = Vec::with_capacity(size);
    for _ in 0..size {
        pixels.push(0);
    }

    for y in 0..board.height {
        for x in 0..board.width {
            draw_char(b'A', 0x0B, 0x01, x, y, board.width, charset, palette, &mut pixels);
        }
    }

    RgbImage::from_raw((board.width * 8) as u32, (board.height * 14) as u32, pixels)
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
