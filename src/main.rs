extern crate image;
extern crate itertools;
extern crate libmzx;

use image::{RgbImage, DynamicImage, ImageFormat};
use itertools::Zip;
use libmzx::{load_world, World, Charset, Palette, Robot, OverlayMode};
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::exit;

fn print_usage() {
    println!("mzxview world.mzx <board#> out.png");
    exit(1);
}

fn draw_char(ch: u8,
             fg_color: u8,
             bg_color: u8,
             x: usize,
             y: usize,
             stride: usize,
             charset: &Charset,
             palette: &Palette,
             pixels: &mut Vec<u8>) {
    let char_bytes = charset.nth(ch);
    for (y_off, byte) in char_bytes.iter().enumerate() {
        for bit in 1..9 {
            let color = if byte & (1 << (bit - 1)) != 0 {
                &palette.colors[fg_color as usize]
            } else {
                &palette.colors[bg_color as usize]
            };
            let start = (y * 14 + y_off) * stride + ((x + 1) * 8 - bit) * 3;
            let end = start + 3;
            let mut pixel = &mut pixels[start..end];
            pixel[0] = color.r * 4;
            pixel[1] = color.g * 4;
            pixel[2] = color.b * 4;
        }
    }
}

fn char_from_id(id: u8, param: u8, robots: &[Robot]) -> u8 {
    match id {
        0 => b' ',
        1 => 178,
        2 => 219,
        3 => 6,

        5 => param,
        6 => 177,
        7 => param,
        8 => 233,
        9 => 254,
        10 => param,
        11 => 254,
        12 => param,
        13 => 178,
        14 => 177,
        15 => 176,
        16 => 254,
        17 => param,


        20 => 176,
        21 => 24,
        22 => 25,
        23 => 26,
        24 => 27,


        27 => 160,
        28 => 4,
        29 => 4,
        30 => 3,
        31 => 9,
        32 => 150,
        33 => 7,
        34 => 176,

        36 => 11,

        38 => 177,
        39 => 12,
        40 => 10,


        43 => 240,
        44 => 239,


        47 => 22,
        48 => 95,

        50 => 7,
        51 => param,
        52 => param,
        53 => param,
        54 => param,
        55 => 229,

        57 => 18,
        58 => 29,

        60 => 206,




        65 => 178,

        67 => 54,
        68 => 64,
        69 => 57,
        70 => 149,
        71 => b' ',

        73 => 42,


        76 => param,
        77 => param,


        80 => 235,
        81 => 236,
        82 => 1,
        83 => 42,
        84 => 2,
        85 => 234,
        86 => 21,
        87 => 224,
        88 => 94,
        89 => 15,
        90 => 5,
        91 => 227,


        94 => 153,
        95 => 148,

        123 | 124 => robots[param as usize - 1].ch,
        125 => 226,
        126 => 232,

        _ => b'!',
    }
}

#[allow(dead_code)]
fn color_from_id(id: u8) -> Option<u8> {
    Some(match id {
        3 => 0x0A,
        8 => 0x07,
        9 => 0x06,
        18 => 0x07,
        19 => 0x07,
        27 => 0x06,
        30 => 0x0C,
        31 => 0x0E,
        32 => 0x0B,
        33 => 0x0F,
        //34 => 0x18,
        35 => 0x03,
        36 => 0x08,
        37 => 0x08,
        38 => 0xEF,
        47 => 0x08,
        48 => 0x08,
        50 => 0x0E,
        55 => 0x07,
        60 => 0x04,
        61 => 0x0F,
        62 => 0x08,
        63 => 0x0C,
        65 => 0x02,
        66 => 0x0D,
        67 => 0x1F,
        68 => 0x1F,
        69 => 0x1F,
        70 => 0x1F,
        72 => 0x09,
        73 => 0x0A,
        74 => 0x0C,
        75 => 0x08,
        78 => 0x0E,
        79 => 0x0A,
        80 => 0x02,
        81 => 0x0F,
        82 => 0x0C,
        83 => 0x0A,
        84 => 0x04,
        85 => 0x07,
        86 => 0x04,
        87 => 0x0E,
        88 => 0x07,
        89 => 0x07,
        90 => 0x02,
        91 => 0x0B,
        92 => 0x0F,
        93 => 0x0F,
        94 => 0x06,
        95 => 0x06,
        97 => 0x08,
        125 => 0x08,
        126 => 0x0F,
        127 => 0x1B,
        _ => return None,
    })
}

fn draw_img(w: &World, board_num: u8) -> Option<RgbImage> {
    let board = &w.boards[board_num as usize];
    let charset = &w.charset;
    let palette = &w.palette;
    let num_colors = palette.colors.len() as u8;

    let px_width = board.width * 8;
    let px_height = board.height * 14;

    let size = px_width * px_height * 3;
    let mut pixels = Vec::with_capacity(size);
    for _ in 0..size {
        pixels.push(0);
    }

    let mut empty_overlay = vec![];
    let overlay = match board.overlay {
        Some((OverlayMode::Static, ref data)) |
        Some((OverlayMode::Normal, ref data)) => data,
        _ => {
            empty_overlay.reserve(board.width * board.height);
            for _ in 0..(board.width * board.height) {
                empty_overlay.push((32, 0x07));
            }
            &empty_overlay
        }
    };

    for (pos, (&(id, mut color, param),
               &(_under_id, under_color, _under_param),
               &(overlay_char, overlay_color)))
        in Zip::new((&board.level, &board.under, overlay)).enumerate() {
        let overlay_visible = overlay_char != b' ';
        let overlay_see_through = overlay_color / num_colors == 0 && overlay_color != 0x00;
        let ch = if !overlay_visible {
            char_from_id(id, param, &board.robots)
        } else {
            overlay_char
        };
        if color / num_colors == 0 {
            color = under_color / num_colors * num_colors + color;
        }
        if overlay_visible {
            if overlay_see_through {
                color = color / num_colors * num_colors + overlay_color;
            } else {
                color = overlay_color;
            }
        }
        draw_char(ch,
                  color % num_colors,
                  color / num_colors,
                  pos % board.width,
                  pos / board.width,
                  px_width * 3,
                  charset,
                  palette,
                  &mut pixels);
    }

    RgbImage::from_raw(px_width as u32, px_height as u32, pixels)
}

fn main() {
    let mut args = env::args();
    let _binary = args.next();
    let world_file = match args.next() {
        Some(path) => path,
        None => return print_usage(),
    };
    let board_num = match args.next() {
        Some(num) => num.parse::<u8>().unwrap(),
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
            return exit(1);
        }
    };

    let world = match load_world(&world_data) {
        Ok(world) => world,
        Err(e) => {
            println!("Error reading {} ({:?})", world_file, e);
            return exit(1);
        }
    };

    match File::create(&image_path) {
        Ok(mut file) => {
            let img = match draw_img(&world, board_num) {
                Some(img) => img,
                None => {
                    println!("Error creating image from pixel buffer");
                    return exit(1);
                }
            };
            let dynamic_image = DynamicImage::ImageRgb8(img);
            if let Err(e) = dynamic_image.save(&mut file, ImageFormat::PNG) {
                println!("Failed to save {} ({}).", image_path, e);
                return exit(1);
            }
        }
        Err(e) => {
            println!("Error creating {} ({})", image_path, e);
            return exit(1);
        }
    }
}
