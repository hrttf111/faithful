use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use image::{RgbImage, Rgb, ImageOutputFormat, ImageBuffer};
use clap::{arg, Arg, ArgAction, Command};

use faithful::pop::level::GlobeTextureParams;
use faithful::pop::landscape::LevelRes;
use faithful::pop::landscape::common::LandPos;
use faithful::pop::landscape::minimap::texture_minimap;
use faithful::pop::landscape::globe::texture_globe;
use faithful::pop::landscape::land::texture_land;
use faithful::pop::landscape::disp::texture_bigf0;
use faithful::pop::landscape::water::texture_water;

/******************************************************************************/

const DEFAULT_IMG_FORMAT: ImageOutputFormat = ImageOutputFormat::Bmp;
const DEFAULT_BASE_PATH: &str = "/opt/sandbox/pop";

fn draw_palette(pal: &[u8], width: u32, height: u32, num_colors: u32) -> RgbImage {
    let mut img = RgbImage::new(width, height);
    let color_height = height / num_colors;
    for c in 0..num_colors {
        let palette_index = (c * 4) as usize;
        let buf: &[u8] = &pal[palette_index..(palette_index+3)];
        let si = c * color_height;
        for i in 0..color_height {
            for j in 0..width {
                img.put_pixel(j, si + i, Rgb([buf[0], buf[1], buf[2]]));
            }
        }
    }
    img
}

fn make_disp_texture2(params: &GlobeTextureParams) -> RgbImage {
    let width = 256;
    let mut img = RgbImage::new(width, width);
    for i in 0..width {
        for j in 0..width {
            let offset_param = i * 0x100;
            let disp_val = params.disp0[(offset_param + j) as usize];
            if disp_val < 0 {
                let v = -(disp_val.clamp(-127, 0));
                img.put_pixel(i, j, Rgb([0, 0, v as u8 * 2]));
            } else {
                img.put_pixel(i, j, Rgb([(disp_val as u8) * 2, 0, 0]));
            }
        }
    }
    img
}

fn draw_texture(pal: &[u8], width: u32, texture: &[u8], tex_width: u32, disp_v: u32, disp_h: u32) -> RgbImage {
    let mut img = RgbImage::new(width, width);
    let pixel_size = width / tex_width;
    for i in 0..tex_width {
        let pixel_height = i * pixel_size;
        for j in 0..tex_width {
            let pixel_width = j * pixel_size;
            let palette_index = texture[(i*tex_width + j) as usize] as usize;
            let palette_index = palette_index.min(127) * 4;
            let buf: &[u8] = &pal[palette_index..=(palette_index+2)];
            for k1 in 0..pixel_size {
                for k2 in 0..pixel_size {
                    let h = (pixel_height + k1 + disp_h) % width;
                    let v = (pixel_width + k2 + disp_v) % width;
                    img.put_pixel(h, v, Rgb([buf[0], buf[1], buf[2]]));
                }
            }
        }
    }
    img
}

/*
 * land [args] <map_num>
 * globe [args] <map_num>
 * minimap [args] <map_num>
 * args:
 *  --move [v; h] -- move maps centre
 *  --base <path> -- base path to pop3
 *  --land <type> -- override landscape texturing type
 */

fn default_args() -> [Arg; 3] {
    [
        Arg::new("move")
            .long("move")
            .action(ArgAction::Set)
            .value_name("POSITION")
            .help("Move texture centre"),
        Arg::new("base")
            .long("base")
            .action(ArgAction::Set)
            .value_name("BASE_PATH")
            .value_parser(clap::value_parser!(PathBuf))
            .help("Path to pop3 directory"),
        Arg::new("landtype")
            .long("landtype")
            .action(ArgAction::Set)
            .value_name("LAND_TYPE")
            .value_parser(clap::builder::StringValueParser::new())
            .help("Override land type"),
    ]
}

fn cli() -> Command {
    let args = default_args();
    Command::new("pop_res")
        .about("Read pop3 resources")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("globe")
                .about("Create globe texture image")
                .arg(arg!(<num> "Level number"))
                .args(&args)
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("land")
                .about("Create full land texture image")
                .arg(arg!(<num> "Level number"))
                .args(&args)
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("minimap")
                .about("Create minimap texture image")
                .arg(arg!(<num> "Level number"))
                .args(&args)
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("water")
                .about("Create water texture image")
                .arg(arg!(<num> "Level number"))
                .arg(arg!(<offset> "Offset"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("bigf0")
                .about("Create bigf0 texture image")
                .arg(arg!(<num> "Level number"))
                .arg(arg!(<height> "Height"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("disp")
                .about("Create disp texture image")
                .arg(arg!(<num> "Level number"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("palette")
                .about("Create palette texture image")
                .arg(arg!(<num> "Level number"))
                .arg_required_else_help(true),
        )
}

enum TextureType {
    Land,
    Globe,
    Minimap,
}

fn write_img_stdout<P, C>(img: &ImageBuffer<P, C>, format: ImageOutputFormat)
    where P: image::Pixel + image::PixelWithColorType,
          C: std::ops::Deref<Target = [P::Subpixel]>,
          [<P as image::Pixel>::Subpixel]: image::EncodableLayout {
    let mut temp_vec = Vec::new();
    img.write_to(&mut Cursor::new(&mut temp_vec), format).unwrap();
    std::io::stdout().write_all(&temp_vec).unwrap();
}

fn make_texture_land(tex_type: TextureType
                     , level_num: u8
                     , base: &Path
                     , level_type_opt: Option<&String>
                     , tex_move: Option<(u32, u32)>
                     ) {
    let level_res = LevelRes::new(base, level_num, level_type_opt.map(|s| s.as_str()));

    let land_size = level_res.landscape.land_size();
    let params_globe = &level_res.params;
    let land = LandPos::from_landscape(&level_res.landscape);

    let (h, v) = tex_move.unwrap_or((0, 0));

    let img = match tex_type {
        TextureType::Land => {
            let texture = texture_land(land_size, &land, params_globe);
            draw_texture(&params_globe.palette, ((land_size)* 32) as u32, &texture, ((land_size)* 32) as u32, h, v)
        }
        TextureType::Globe => {
            let texture = texture_globe(land_size, &land, params_globe);
            draw_texture(&params_globe.palette, ((land_size)* 8) as u32, &texture, ((land_size)* 8) as u32, h, v)
        }
        TextureType::Minimap => {
            let texture = texture_minimap(land_size, true, &land, &params_globe.bigf0);
            draw_texture(&params_globe.palette, (land_size * 8) as u32, &texture, land_size as u32, h, v)
        }
    };

    write_img_stdout(&img, DEFAULT_IMG_FORMAT);
}

fn parse_move(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split(';').collect();
    if parts.len() != 2 {
        return None;
    }
    let a = &parts[0];
    let b = &parts[1];
    Some((a.parse().unwrap(), b.parse().unwrap()))
}

fn main() {
    let base_path = Path::new(DEFAULT_BASE_PATH);

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("globe", sub_matches)) => {
            let level_num = sub_matches.get_one::<String>("num").expect("required").parse().unwrap();
            let level_type = sub_matches.get_one::<String>("landtype");
            let tex_move = sub_matches.get_one::<String>("move").and_then(|s| parse_move(s));
            make_texture_land(TextureType::Globe, level_num, base_path, level_type, tex_move);
        }
        Some(("land", sub_matches)) => {
            let level_num = sub_matches.get_one::<String>("num").expect("required").parse().unwrap();
            let level_type = sub_matches.get_one::<String>("landtype");
            let tex_move = sub_matches.get_one::<String>("move").and_then(|s| parse_move(s));
            make_texture_land(TextureType::Land, level_num, base_path, level_type, tex_move);
        }
        Some(("minimap", sub_matches)) => {
            let level_num = sub_matches.get_one::<String>("num").expect("required").parse().unwrap();
            let level_type = sub_matches.get_one::<String>("landtype");
            make_texture_land(TextureType::Minimap, level_num, base_path, level_type, None);
        }
        Some(("water", sub_matches)) => {
            let level_num = sub_matches.get_one::<String>("num").expect("required").parse().unwrap();
            let offset = sub_matches.get_one::<String>("offset").expect("required").parse().unwrap();
            let level_res = LevelRes::new(base_path, level_num, None);
            let texture = texture_water(offset, &level_res.params);
            let img = draw_texture(&level_res.params.palette, 256_u32, &texture, 256_u32, 0, 0);
            write_img_stdout(&img, DEFAULT_IMG_FORMAT);
        }
        Some(("bigf0", sub_matches)) => {
            let level_num = sub_matches.get_one::<String>("num").expect("required").parse().unwrap();
            let height = sub_matches.get_one::<String>("height").expect("required").parse().unwrap();
            let level_res = LevelRes::new(base_path, level_num, None);
            let texture = texture_bigf0(height, &level_res.params);
            let img = draw_texture(&level_res.params.palette, 256_u32, &texture, 256_u32, 0, 0);
            write_img_stdout(&img, DEFAULT_IMG_FORMAT);
        }
        Some(("disp", sub_matches)) => {
            let level_num = sub_matches.get_one::<String>("num").expect("required").parse().unwrap();
            let level_res = LevelRes::new(base_path, level_num, None);
            let img = make_disp_texture2(&level_res.params);
            write_img_stdout(&img, DEFAULT_IMG_FORMAT);
        }
        Some(("palette", sub_matches)) => {
            let level_num = sub_matches.get_one::<String>("num").expect("required").parse().unwrap();
            let level_res = LevelRes::new(base_path, level_num, None);
            let img = draw_palette(&level_res.params.palette, 1024, 1024, 128);
            write_img_stdout(&img, DEFAULT_IMG_FORMAT);
        }
        _ => {}
    }
}
