extern crate env_logger;
extern crate image;
extern crate libmzx;

use image::{RgbImage, DynamicImage, ImageFormat};
use libmzx::{
    load_world, World, Charset, Palette, Robot, Command, Counters, Resolve,
    WorldState, Board, Color, Renderer
};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::exit;

fn print_usage() {
    println!("mzxview world.mzx <board#> out.png");
    exit(1);
}

struct ImgRenderer {
    pixels: Vec<u8>,
    stride: usize,
}

fn render(w: &WorldState, board: &Board, robots: &[Robot]) -> Option<RgbImage> {
    let px_width = board.width * 8;
    let px_height = board.height * 14;

    let size = px_width * px_height * 3;

    let mut r = ImgRenderer {
        pixels: vec![0; size],
        stride: px_width * 3,
    };
    libmzx::render(w, board, robots, &mut r);
    RgbImage::from_raw(px_width as u32, px_height as u32, r.pixels)
}

impl Renderer for ImgRenderer {
    fn put_pixel(
        &mut self,
        x: usize,
        y: usize,
        r: u8,
        g: u8,
        b: u8,
    ) {
        let start = y * self.stride + x * 3;
        let end = start + 3;
        self.pixels[start..end].copy_from_slice(&[r, g, b]);
    }
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
            let img = match render(&world.state, &world.boards[board_num], &world.board_robots[board_num]) {
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
