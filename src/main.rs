extern crate env_logger;
extern crate image;
extern crate itertools;
extern crate num_traits;
extern crate libmzx;

use image::{RgbImage, DynamicImage, ImageFormat};
use itertools::Zip;
use libmzx::{
    load_world, World, Charset, Palette, Robot, OverlayMode, Sensor, Command, Counters, Resolve,
    WorldState, Board, Thing, Color
};
use num_traits::FromPrimitive;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
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

fn char_from_id(id: u8, param: u8, robots: &[Robot], sensors: &[Sensor]) -> u8 {
    match Thing::from_u8(id).expect("invalid thing") {
        Thing::Space => b' ',
        Thing::Normal => 178,
        Thing::Solid => 219,
        Thing::Tree => 6,

        Thing::CustomBlock => param,
        Thing::Breakaway => 177,
        Thing::CustomBreak => param,
        Thing::Boulder => 233,
        Thing::Crate => 254,
        Thing::CustomPush => param,
        Thing::Box => 254,
        Thing::CustomBox => param,
        Thing::Fake => 178,
        Thing::Carpet => 177,
        Thing::Floor => 176,
        Thing::Tiles => 254,
        Thing::CustomFloor => param,


        Thing::StillWater => 176,
        Thing::NWater => 24,
        Thing::SWater => 25,
        Thing::EWater => 26,
        Thing::WWater => 27,


        Thing::Chest => 160,
        Thing::Gem => 4,
        Thing::MagicGem => 4,
        Thing::Health => 3,
        Thing::Ring => 9,
        Thing::Potion => 150,
        Thing::Energizer => 7,
        Thing::Goop => 176,

        Thing::Bomb => 11,

        Thing::Explosion => 177,
        Thing::Key => 12,
        Thing::Lock => 10,


        Thing::Stairs => 240,
        Thing::Cave => 239,


        Thing::Gate => 22,
        Thing::OpenGate => 95,

        Thing::Coin => 7,
        Thing::NMovingWall => param,
        Thing::SMovingWall => param,
        Thing::EMovingWall => param,
        Thing::WMovingWall => param,
        Thing::Pouch => 229,

        Thing::SliderNS => 18,
        Thing::SliderEW => 29,

        Thing::LazerGun => 206,




        Thing::Forest => 178,

        Thing::Whirlpool1 => 54,
        Thing::Whirlpool2 => 64,
        Thing::Whirlpool3 => 57,
        Thing::Whirlpool4 => 149,
        Thing::InvisibleWall => b' ',

        Thing::Ricochet => 42,


        Thing::CustomHurt => param,
        Thing::Text => param,


        Thing::Snake => 235,
        Thing::Eye => 236,
        Thing::Thief => 1,
        Thing::SlimeBlob => 42,
        Thing::Runner => 2,
        Thing::Ghost => 234,
        Thing::Dragon => 21,
        Thing::Fish => 224,
        Thing::Shark => 94,
        Thing::Spider => 15,
        Thing::Goblin => 5,
        Thing::SpittingTiger => 227,


        Thing::Bear => 153,
        Thing::BearCub => 148,

        Thing::Sensor => sensors[param as usize - 1].ch,
        Thing::RobotPushable | Thing::Robot => robots[param as usize - 1].ch,
        Thing::Sign => 226,
        Thing::Scroll => 232,
        Thing::Player => 0x02,

        _ => b'!',
    }
}

fn draw_img(w: &WorldState, board: &Board, robots: &[Robot]) -> Option<RgbImage> {
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
            char_from_id(id, param, &robots, &board.sensors)
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

fn run_robot_until_end(
    world: &mut WorldState,
    board: &mut Board,
    world_path: &Path,
    counters: &mut Counters,
    robot: &mut Robot,
) {
    for cmd in &robot.program {
        match cmd {
            Command::End | Command::Wait(_) => break,
            Command::LoadCharSet(ref c) => {
                let path = world_path.join(c.to_string());
                match File::open(&path) {
                    Ok(mut file) => {
                        let mut v = vec![];
                        file.read_to_end(&mut v).unwrap();
                        world.charset.data.copy_from_slice(&v);
                    }
                    Err(e) => {
                        println!("Error opening charset {} ({})", path.display(), e);
                    }
                }
            }

            Command::LoadPalette(ref p) => {
                let path = world_path.join(p.to_string());
                match File::open(&path) {
                    Ok(mut file) => {
                        let mut v = vec![];
                        file.read_to_end(&mut v).unwrap();
                        for (new, old) in v.chunks(3).zip(world.palette.colors.iter_mut()) {
                            old.r = new[0];
                            old.g = new[1];
                            old.b = new[2];
                        }
                    }
                    Err(e) => {
                        println!("Error opening palette {} ({})", path.display(), e);
                    }
                }
            }
            Command::SetColor(c, r, g, b) => {
                world.palette.colors[c.resolve(counters) as usize] =
                    Color {
                        r: r.resolve(counters) as u8,
                        g: g.resolve(counters) as u8,
                        b: b.resolve(counters) as u8,
                    };
            }

            Command::Char(ch) => {
                robot.ch = ch.resolve(counters);
            }
            Command::Color(c) => {
                board.level_at_mut(&robot.position).1 = c.resolve(counters).0;
            }
            Command::PlayerColor(c) => {
                let player_pos = board.player_pos;
                board.level_at_mut(&player_pos).1 = c.resolve(counters).0;
            }
            _ => (),
        }
    }
}

fn run_all_robots(
    world: &mut World,
    world_path: &Path,
    board_id: usize
) {
    let mut counters = Counters::new();
    for robot in &mut world.board_robots[board_id] {
        run_robot_until_end(&mut world.state, &mut world.boards[board_id], world_path, &mut counters, robot);
    }
}

fn main() {
    env_logger::init();
    let mut args = env::args();
    let _binary = args.next();
    let world_file = match args.next() {
        Some(path) => path,
        None => return print_usage(),
    };
    let board_num = match args.next() {
        Some(num) => num.parse::<usize>().unwrap(),
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
            exit(1)
        }
    };

    let mut world = match load_world(&world_data) {
        Ok(world) => world,
        Err(e) => {
            println!("Error reading {} ({:?})", world_file, e);
            exit(1)
        }
    };

    if board_num >= world.boards.len() {
        println!("World only contains {} boards", world.boards.len());
        exit(1);
    }

    let world_path = Path::new(&world_file).parent().unwrap();
    run_all_robots(&mut world, &world_path, board_num);

    match File::create(&image_path) {
        Ok(mut file) => {
            let img = match draw_img(&world.state, &world.boards[board_num], &world.board_robots[board_num]) {
                Some(img) => img,
                None => {
                    println!("Error creating image from pixel buffer");
                    exit(1)
                }
            };
            let dynamic_image = DynamicImage::ImageRgb8(img);
            if let Err(e) = dynamic_image.save(&mut file, ImageFormat::PNG) {
                println!("Failed to save {} ({}).", image_path, e);
                exit(1)
            }
        }
        Err(e) => {
            println!("Error creating {} ({})", image_path, e);
            exit(1)
        }
    }
}
