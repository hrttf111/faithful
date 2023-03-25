use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

use image::{RgbImage, RgbaImage, Rgb, GrayImage, ImageFormat, ImageOutputFormat, ImageBuffer, DynamicImage};
use clap::{arg, Arg, ArgAction, Command};

use faithful::pop::level::{GlobeTextureParams, LevelPaths, ObjectPaths, read_pal};
use faithful::pop::psfb::ContainerPSFB;
use faithful::pop::landscape::LevelRes;
use faithful::pop::landscape::common::LandPos;
use faithful::pop::landscape::minimap::texture_minimap;
use faithful::pop::landscape::globe::texture_globe;
use faithful::pop::landscape::land::texture_land;
use faithful::pop::landscape::disp::texture_bigf0;
use faithful::pop::landscape::water::texture_water;
use faithful::pop::pls::decode;
use faithful::pop::bl320::{parse_bl320, parse_bl160};
use faithful::pop::types::{BinDeserializer, Image};
use faithful::pop::objects::{ObjectRaw, Shape, PointRaw, FaceRaw};

/******************************************************************************/

const DEFAULT_IMG_FORMAT: ImageOutputFormat = ImageOutputFormat::Bmp;
//const DEFAULT_BASE_PATH: &str = "/opt/sandbox/pop/data";
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

#[allow(clippy::needless_range_loop)]
fn draw_texture(pal: &[u8], width: u32, texture: Vec<u8>) -> RgbaImage {
    let img = GrayImage::from_raw(width as u32, width as u32, texture).unwrap();
    let mut pal_tup = [(0u8, 0u8, 0u8); 256];
    for i in 0..255 {
        let pi = i * 4;
        pal_tup[i] = (pal[pi], pal[pi+1], pal[pi+2]);
    }
    img.expand_palette(&pal_tup, None)
}

/*
fn draw_texture2(pal: &[u8], width: u32, texture: &[u8], tex_width: u32, disp_v: u32, disp_h: u32) -> RgbImage {
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
*/

fn draw_images(pal: &[u8], sprites: &[Image]) -> RgbImage {
    let (width, height, sprite_width) = {
        let mut width = 0;
        let mut height = 0;
        let mut sprite_width = 0;
        for sprite in sprites {
            width += sprite.width;
            height = std::cmp::max(height, sprite.height);
            sprite_width = sprite.width;
        }
        (width, height, sprite_width)
    };
    let mut texture = vec![0; width * height];
    for i in 0..height {
        for (k, sprite) in sprites.iter().enumerate() {
            for j in 0..sprite_width {
                texture[i * width + sprite_width * k + j] = sprite.data[i * sprite_width + j];
            }
        }
    }
    let mut img = RgbImage::new(width as u32, height as u32);
    for i in 0..height {
        for j in 0..width {
            let palette_index = texture[i*width + j] as usize;
            let palette_index = palette_index * 4;
            let buf: &[u8] = &pal[palette_index..=(palette_index+2)];
            img.put_pixel(j as u32, i as u32, Rgb([buf[0], buf[1], buf[2]]));
        }
    }
    img
}

fn image_to_gray(sprite: Image) -> GrayImage {
    let width = sprite.width;
    let height = sprite.height;
    GrayImage::from_raw(width as u32, height as u32, sprite.data).unwrap()
}

fn decode_pls(pls_path: &Path) -> Vec<u8> {
    let mut file = File::options().read(true).open(pls_path).unwrap();
    let mut data = Vec::new();
    let _file_size = file.read_to_end(&mut data);
    decode(&mut data);
    data
}

fn draw_image(sprite: Image, palette: &Option<[(u8, u8, u8); 256]>) -> DynamicImage {
    let img = image_to_gray(sprite);
    if let Some(p) = palette {
        return DynamicImage::ImageRgba8(img.expand_palette(p, None));
    }
    DynamicImage::ImageLuma8(img)
}

fn draw_sprites_img(psfb: &ContainerPSFB, start: u32, num: u32, prefix: &Path, palette: &Option<[(u8, u8, u8); 256]>) {
    if start >= num {
        return;
    }
    for i in start..(start+num) {
        if let Some(sprite) = psfb.get_image(i as usize) {
            let name = format!("{:}_{:?}.bmp", prefix.to_str().unwrap(), i);
            println!("{}", name);
            let path = Path::new(&name);
            let img = draw_image(sprite, palette);
            img.save_with_format(&path, ImageFormat::Bmp).unwrap();
        }
    }
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
            Command::new("bl320")
                .about("Create image for BL320")
                .args(&args)
        )
        .subcommand(
            Command::new("bl160")
                .about("Create image for BL160")
                .args(&args)
                .arg(arg!(<width> "Sprite width"))
                .arg(arg!(<height> "Sprite height"))
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
        .subcommand(
            Command::new("objects")
                .about("Objects commands")
                .arg(arg!(<num> "Bank num"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("pls")
                .about("Decode pls files")
                .arg(
                    Arg::new("pls_path")
                    .action(ArgAction::Set)
                    .value_name("PLS_PATH")
                    .value_parser(clap::value_parser!(PathBuf))
                    .help("Path to pls file")
                )
        )
        .subcommand(
            Command::new("psfb")
                .about("Read sprites from psfb file")
                .args([
                    Arg::new("path")
                        .long("path")
                        .action(ArgAction::Set)
                        .value_name("FILE_PATH")
                        .value_parser(clap::value_parser!(PathBuf))
                        .help("Path to PSFB file"),
                    Arg::new("palette")
                        .long("palette")
                        .action(ArgAction::Set)
                        .value_name("PALETTE_PATH")
                        .value_parser(clap::value_parser!(PathBuf))
                        .help("Path to palette file"),
                    Arg::new("info")
                        .long("info")
                        .action(ArgAction::SetTrue)
                        .help("Show file info"),
                    Arg::new("start")
                        .long("start")
                        .action(ArgAction::Set)
                        .value_name("START_NUM")
                        .value_parser(clap::value_parser!(u32).range(0..20000))
                        .help("Start from sprite number"),
                    Arg::new("num")
                        .long("num")
                        .action(ArgAction::Set)
                        .value_name("SPRITE_NUM")
                        .value_parser(clap::value_parser!(u32).range(0..20000))
                        .help("Draw number of sprites"),
                    Arg::new("prefix")
                        .long("prefix")
                        .action(ArgAction::Set)
                        .value_name("PREFIX_PATH")
                        .value_parser(clap::value_parser!(PathBuf))
                        .help("Prefix for generated images"),
                ]).arg_required_else_help(true),
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

fn write_dyn_img_stdout(img: &DynamicImage, format: ImageOutputFormat) {
    let mut temp_vec = Vec::new();
    img.write_to(&mut Cursor::new(&mut temp_vec), format).unwrap();
    std::io::stdout().write_all(&temp_vec).unwrap();
}

fn make_texture_land(tex_type: TextureType
                     , level_num: u8
                     , base: &Path
                     , level_type_opt: Option<&String>
                     , _tex_move: Option<(u32, u32)>
                     ) {
    let level_res = LevelRes::new(base, level_num, level_type_opt.map(|s| s.as_str()));

    let land_size = level_res.landscape.land_size();
    let params_globe = &level_res.params;
    let land = LandPos::from_landscape_sun(&level_res.landscape);

    //let (h, v) = tex_move.unwrap_or((0, 0));

    let img = match tex_type {
        TextureType::Land => {
            let texture = texture_land(land_size, &land, params_globe);
            draw_texture(&params_globe.palette, ((land_size)* 32) as u32, texture)
        }
        TextureType::Globe => {
            let texture = texture_globe(land_size, &land, params_globe);
            draw_texture(&params_globe.palette, (land_size * 8) as u32, texture)
        }
        TextureType::Minimap => {
            let texture = texture_minimap(land_size, true, &land, &params_globe.bigf0);
            draw_texture(&params_globe.palette, land_size as u32, texture)
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

#[allow(clippy::needless_range_loop)]
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
        Some(("bl320", sub_matches)) => {
            let level_type: String = sub_matches.get_one::<String>("landtype").expect("required").parse().unwrap();
            let paths = LevelPaths::from_base(base_path, &level_type);
            let pal = read_pal(&paths);
            let sprites = parse_bl320(&paths.bl320);
            let img = draw_images(&pal, &sprites);
            write_img_stdout(&img, DEFAULT_IMG_FORMAT);
        }
        Some(("bl160", sub_matches)) => {
            let level_type: String = sub_matches.get_one::<String>("landtype").expect("required").parse().unwrap();
            let width: usize = sub_matches.get_one::<String>("width").expect("required").parse().unwrap();
            let height: usize = sub_matches.get_one::<String>("height").expect("required").parse().unwrap();
            let paths = LevelPaths::from_base(base_path, &level_type);
            let pal = read_pal(&paths);
            let sprites = parse_bl160(width, height, &paths.bl160);
            let img = draw_images(&pal, &sprites);
            write_img_stdout(&img, DEFAULT_IMG_FORMAT);
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
            let img = draw_texture(&level_res.params.palette, 256_u32, texture);
            write_img_stdout(&img, DEFAULT_IMG_FORMAT);
        }
        Some(("bigf0", sub_matches)) => {
            let level_num = sub_matches.get_one::<String>("num").expect("required").parse().unwrap();
            let height = sub_matches.get_one::<String>("height").expect("required").parse().unwrap();
            let level_res = LevelRes::new(base_path, level_num, None);
            let texture = texture_bigf0(height, &level_res.params);
            let img = draw_texture(&level_res.params.palette, 256_u32, texture);
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
        Some(("objects", sub_matches)) => {
            let bank_num = sub_matches.get_one::<String>("num").expect("required");
            let paths = ObjectPaths::from_base(base_path, bank_num);
            let objects = ObjectRaw::from_file_vec(&paths.objs0_dat);
            let points = PointRaw::from_file_vec(&paths.pnts0);
            let faces = FaceRaw::from_file_vec(&paths.facs0);
            let shapes = Shape::from_file_vec(&paths.shapes);
            println!("Num objects = {}", objects.len());
            for obj in &objects {
                println!("  {:?}", obj);
            }
            println!("Num shapes = {}", shapes.len());
            for shape in shapes {
                println!("  {:?}", shape);
            }
            println!("Num points = {}", points.len());
            for point in &points {
                println!("  {:?}", point);
            }
            println!("Num faces = {}", faces.len());
            for face in &faces {
                println!("  {:?}", face);
            }
        }
        Some(("pls", sub_matches)) => {
            let path = sub_matches.get_one::<PathBuf>("pls_path").expect("required");
            let pls_data = decode_pls(path);
            std::io::stdout().write_all(&pls_data).unwrap();
        }
        Some(("psfb", sub_matches)) => {
            let file_path: PathBuf = sub_matches.get_one("path").cloned().unwrap();
            let palette_path: Option<PathBuf> = sub_matches.get_one("palette").cloned();
            let info: bool = sub_matches.get_flag("info");
            let start_num: Option<u32> = sub_matches.get_one("start").copied();
            let num: Option<u32> = sub_matches.get_one("num").copied();
            let palette = if let Some(path) = palette_path {
                let mut file = File::options().read(true).open(path).unwrap();
                let mut pal = Vec::new();
                file.read_to_end(&mut pal).unwrap();
                let mut pal_tup = [(0u8, 0u8, 0u8); 256];
                for i in 0..255 {
                    let pi = i * 4;
                    pal_tup[i] = (pal[pi], pal[pi+1], pal[pi+2]);
                }
                Some(pal_tup)
            } else {
                None
            };
            if let Some(c) = ContainerPSFB::from_file(&file_path) {
                if info {
                    println!("PSFB file '{file_path:?}': ");
                    println!("    size = {:?}", c.size());
                    println!("    sprites count = {:?}", c.len());
                    for sprite in c.sprites_info() {
                        let size = sprite.width as usize * sprite.height as usize;
                        println!(" Sprite index={:?} offset={:?}/0x{:x}, size={:?}, width={:?}, height={:?}"
                                 , sprite.index, sprite.offset, sprite.offset, size, sprite.width, sprite.height);
                    }
                } else {
                    match (start_num, num) {
                        (Some(i), Some(n)) => {
                            let prefix: PathBuf = sub_matches.get_one("prefix").cloned().unwrap();
                            draw_sprites_img(&c, i, n, &prefix, &palette);
                        }
                        (None, Some(n)) => {
                            let prefix: PathBuf = sub_matches.get_one("prefix").cloned().unwrap();
                            draw_sprites_img(&c, 0, n, &prefix, &palette);
                        }
                        (Some(i), None) => {
                            if let Some(image) = c.get_image(i as usize) {
                                let img = draw_image(image, &palette);
                                write_dyn_img_stdout(&img, DEFAULT_IMG_FORMAT);
                            }
                        }
                        (None, None) => {
                            let prefix: PathBuf = sub_matches.get_one("prefix").cloned().unwrap();
                            draw_sprites_img(&c, 0, c.len() as u32, &prefix, &palette);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
