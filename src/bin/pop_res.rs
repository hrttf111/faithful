use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;

use image::{RgbImage, RgbaImage, Rgb, GrayImage, ImageFormat, ImageOutputFormat, ImageBuffer, DynamicImage};
use clap::{arg, Arg, ArgAction, Command};

use faithful::pop::level::{GlobeTextureParams, LevelPaths, LevelRes, ObjectPaths, read_pal};
use faithful::pop::psfb::ContainerPSFB;
use faithful::pop::landscape::common::{LandPos, LandscapeFull};
use faithful::pop::landscape::minimap::texture_minimap;
use faithful::pop::landscape::globe::texture_globe;
use faithful::pop::landscape::land::texture_land;
use faithful::pop::landscape::disp::texture_bigf0;
use faithful::pop::landscape::water::texture_water;
use faithful::pop::pls::decode;
use faithful::pop::bl320::{read_bl320, read_bl160};
use faithful::pop::types::{BinDeserializer, Image, AllocatorIter, ULCentreComposer, URCentreComposer, LayeredStorageSource, LayerComposer};
use faithful::pop::types::{ImageInfo, ImageArea};
use faithful::pop::types::{image_allocator_1d_horizontal, image_allocator_1d_vertical, image_allocator_2d};
use faithful::pop::objects::{ObjectRaw, Shape, PointRaw, FaceRaw};
use faithful::pop::animation::{AnimationsData, AnimationSequence, AnimationFrame};

/******************************************************************************/

const DEFAULT_IMG_FORMAT: ImageOutputFormat = ImageOutputFormat::Bmp;
const DEFAULT_BASE_PATH: &str = "/opt/sandbox/pop";

type PaletteArray = [(u8, u8, u8); 256];
type FramesSet = HashSet<usize>;

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

fn decode_pls(pls_path: &Path) -> Vec<u8> {
    let mut file = File::options().read(true).open(pls_path).unwrap();
    let mut data = Vec::new();
    let _file_size = file.read_to_end(&mut data);
    decode(&mut data);
    data
}

#[allow(clippy::needless_range_loop)]
fn seq_to_pal(pal: &[u8]) -> PaletteArray {
    let mut pal_tup = [(0u8, 0u8, 0u8); 256];
    for i in 0..255 {
        let pi = i * 4;
        pal_tup[i] = (pal[pi], pal[pi+1], pal[pi+2]);
    }
    pal_tup
}

fn image_to_gray(sprite: Image) -> GrayImage {
    let width = sprite.width;
    let height = sprite.height;
    GrayImage::from_raw(width as u32, height as u32, sprite.data).unwrap()
}

fn draw_image_pal(pal: &[u8], image: Image) -> RgbaImage {
    let img = image_to_gray(image);
    let pal_tup = seq_to_pal(pal);
    img.expand_palette(&pal_tup, None)
}

fn draw_image(palette: &Option<PaletteArray>, img: Image) -> DynamicImage {
    let img = image_to_gray(img);
    if let Some(p) = palette {
        return DynamicImage::ImageRgba8(img.expand_palette(p, None));
    }
    DynamicImage::ImageLuma8(img)
}

fn draw_sprites(psfb: &ContainerPSFB, start: usize, num: usize, prefix: &Path, palette: &Option<PaletteArray>) {
    if start >= num {
        return;
    }
    for i in start..(start+num) {
        if let Some(sprite) = psfb.get_image(i as usize) {
            let name = format!("{:}_{:?}.bmp", prefix.to_str().unwrap(), i);
            println!("{}", name);
            let path = Path::new(&name);
            let img = draw_image(palette, sprite);
            img.save_with_format(&path, ImageFormat::Bmp).unwrap();
        }
    }
}

fn draw_sprites_img(psfb: &ContainerPSFB, start: usize, num: usize, palette: &Option<PaletteArray>) -> DynamicImage {
    let allocator = image_allocator_2d(1500);
    let mut p = allocator.alloc_iter(&mut psfb.sprites_info().iter());
    for i in start..(start+num) {
        psfb.get_storage(i as usize, &mut p);
    }
    let image = p.get_image();
    draw_image(palette, image)
}

struct AnimationsConfig {
    img_size: usize,
    with_tribe: bool,
    with_type: bool,
}

fn draw_anim_frames<L>(anim_seq: &Vec<AnimationSequence>
                      , psfb: &ContainerPSFB
                      , palette: &Option<PaletteArray>
                      , frames_set: &FramesSet
                      , composer: &L
                      , config: &AnimationsConfig
                      ) -> DynamicImage
    where L: LayerComposer<ComposerResult=ImageArea> {
    let allocator = image_allocator_2d(config.img_size);
    let frames = {
        let seq = AnimationSequence::get_frames(anim_seq);
        if !frames_set.is_empty() {
            seq.into_iter().filter(|f| frames_set.contains(&f.index)).collect::<Vec<AnimationFrame>>()
        } else {
            seq
        }
    };
    let composed_sprites: Vec<L::ComposerResult> = frames.iter().flat_map(|frame| {
        frame.get_permutations(config.with_tribe, config.with_type).into_iter().map(|sprites_seq| {
            let v = sprites_seq.into_iter().filter_map(|sprite| {
                psfb.get_info(sprite.sprite_index).map(|im| {
                    ImageArea::from_image(&im, sprite.coord_x as isize, sprite.coord_y as isize)
                })
            }).collect::<Vec<ImageArea>>();
            composer.compose_layers(&mut v.iter())
        }).collect::<Vec<L::ComposerResult>>()
    }).collect();
    let mut p = allocator.alloc_iter(&mut composed_sprites.iter());
    let mut cs_iter = composed_sprites.iter();
    for frame in &frames {
        let pr = frame.get_permutations(config.with_tribe, config.with_type);
        for (elems, i) in pr.into_iter().zip(&mut cs_iter) {
            let img_area = ImageArea::from_image_pos(i);
            let mut ulc = LayeredStorageSource::new(&mut p, img_area, elems.iter().map(|im| (im.coord_x as isize, im.coord_y as isize)), composer);
            for elem in &elems {
                psfb.get_storage(elem.sprite_index, &mut ulc);
            }
        }
    }
    let image = p.get_image();
    draw_image(palette, image)
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
            Command::new("units")
                .about("Units commands")
                .arg(arg!(<num> "Level number"))
                .args(&args)
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("anims")
                .about("Animations commands")
                .args([
                    Arg::new("psfb_path")
                        .long("psfb_path")
                        .action(ArgAction::Set)
                        .value_name("FILE_PATH")
                        .value_parser(clap::value_parser!(PathBuf))
                        .help("Path to PSFB file"),
                ]).arg_required_else_help(true),
        )
        .subcommand(
            Command::new("anims_draw")
                .about("Animations commands")
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
                    Arg::new("composer")
                        .long("composer")
                        .action(ArgAction::Set)
                        .value_name("COMPOSER")
                        .value_parser(clap::builder::StringValueParser::new())
                        .help("Composer type"),
                    Arg::new("ids")
                        .long("ids")
                        .action(ArgAction::Set)
                        .value_name("IDS")
                        .value_parser(clap::builder::StringValueParser::new())
                        .help("Frames ids"),
                    Arg::new("no_tribe")
                        .long("no_tribe")
                        .action(ArgAction::SetTrue)
                        .help("Do not show tribe images"),
                    Arg::new("no_type")
                        .long("no_type")
                        .action(ArgAction::SetTrue)
                        .help("Do not show type images"),
                ]).arg_required_else_help(true),
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
    let landscape = LandscapeFull::new(land_size, land);

    //let (h, v) = tex_move.unwrap_or((0, 0));

    let img = match tex_type {
        TextureType::Land => {
            texture_land(land_size, &landscape, params_globe)
        }
        TextureType::Globe => {
            texture_globe(land_size, &landscape, params_globe)
        }
        TextureType::Minimap => {
            texture_minimap(land_size, true, &landscape, &params_globe.bigf0)
        }
    };
    let img = draw_image_pal(&params_globe.palette, img);

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

fn parse_ids(s: &str) -> HashSet<usize> {
    HashSet::from_iter(s.split(',').map(|s| s.parse::<usize>().unwrap()))
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
        Some(("bl320", sub_matches)) => {
            let level_type: String = sub_matches.get_one::<String>("landtype").expect("required").parse().unwrap();
            let paths = LevelPaths::from_default_dir(base_path, &level_type);
            let pal = read_pal(&paths);
            let allocator = image_allocator_1d_horizontal();
            let provider = read_bl320(&allocator, &paths.bl320);
            let img = draw_image_pal(&pal, provider.get_image());
            write_img_stdout(&img, DEFAULT_IMG_FORMAT);
        }
        Some(("bl160", sub_matches)) => {
            let level_type: String = sub_matches.get_one::<String>("landtype").expect("required").parse().unwrap();
            let width: usize = sub_matches.get_one::<String>("width").expect("required").parse().unwrap();
            let height: usize = sub_matches.get_one::<String>("height").expect("required").parse().unwrap();
            let paths = LevelPaths::from_default_dir(base_path, &level_type);
            let pal = read_pal(&paths);
            let allocator = image_allocator_1d_vertical();
            let provider = read_bl160(width, height, &allocator, &paths.bl160);
            let img = draw_image_pal(&pal, provider.get_image());
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
            let img = texture_water(offset, &level_res.params);
            let img = draw_image_pal(&level_res.params.palette, img);
            write_img_stdout(&img, DEFAULT_IMG_FORMAT);
        }
        Some(("bigf0", sub_matches)) => {
            let level_num = sub_matches.get_one::<String>("num").expect("required").parse().unwrap();
            let height = sub_matches.get_one::<String>("height").expect("required").parse().unwrap();
            let level_res = LevelRes::new(base_path, level_num, None);
            let img = texture_bigf0(height, &level_res.params);
            let img = draw_image_pal(&level_res.params.palette, img);
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
            let paths = ObjectPaths::from_default_dir(base_path, bank_num);
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
        Some(("units", sub_matches)) => {
            let level_num = sub_matches.get_one::<String>("num").expect("required").parse().unwrap();
            let level_res = LevelRes::new(base_path, level_num, None);
            println!("Num units = {}", level_res.units.len());
            for unit in &level_res.units {
                if unit.unit_class != 0 {
                    println!("  {:?}", unit);
                }
            }
            for tribe in &level_res.tribes {
                println!("  {:?}", tribe);
            }
            println!("  {:?}", level_res.sunlight);
        }
        Some(("anims", sub_matches)) => {
            let psfb_path = sub_matches.get_one::<PathBuf>("psfb_path");
            println!("PSFB = {:?}", psfb_path);
            let psfb_container = psfb_path.and_then(|p| ContainerPSFB::from_file(Path::new(&p)));
            let anims_data = AnimationsData::from_path(&base_path.join("data"));
            println!("Num vele={:?}, vfra={:?}, vstart={:?}"
                    , anims_data.vele.len(), anims_data.vfra.len(), anims_data.vstart.len());
            for (index, vele) in (0..).zip(&anims_data.vele) {
                println!("  {:?}:{:?}", index, vele);
            }
            for (index, vfra) in (0..).zip(&anims_data.vfra) {
                println!("  {:?}:{:?}", index, vfra);
            }
            for vstart in &anims_data.vstart {
                println!("  {:?}", vstart);
            }
            let anim_seq_vec = AnimationSequence::from_data(&anims_data);
            for anim_seq in &anim_seq_vec {
                println!("AnimationSequence {:?} ({:?})", anim_seq.index, anim_seq.frames.len());
                for anim_frame in &anim_seq.frames {
                    println!("  AnimationFrame {:?} ({:?}, {:?})", anim_frame.index, anim_frame.width, anim_frame.height);
                    for anim_sprite in &anim_frame.sprites {
                        println!("    AnimationElement {:?} ({:?}, {:?}, tribe={:?}, flags=0x{:x}, uvar5=0x{:x}, original_flags=0x{:x})"
                                , anim_sprite.sprite_index
                                , anim_sprite.coord_x
                                , anim_sprite.coord_y
                                , anim_sprite.tribe
                                , anim_sprite.flags
                                , anim_sprite.uvar5
                                , anim_sprite.original_flags);
                        if let Some(sprite) = psfb_container.as_ref().and_then(|p| p.get_info(anim_sprite.sprite_index)) {
                            let fit = (anim_frame.width >= sprite.width()) && (anim_frame.height >= sprite.height());
                            let fit32 = (32 >= sprite.width()) && (32 >= sprite.height());
                            println!("     Sprite({:?}, {:?}, fit={:?}, fit32={:?})", sprite.width(), sprite.height(), fit, fit32);
                        }
                    }
                }
            }
        }
        Some(("anims_draw", sub_matches)) => {
            let anims_data = AnimationsData::from_path(&base_path.join("data"));
            let anim_seq_vec = AnimationSequence::from_data(&anims_data);
            let file_path: PathBuf = sub_matches.get_one("path").cloned().unwrap();
            let palette_path: Option<PathBuf> = sub_matches.get_one("palette").cloned();
            let s = String::from("ul");
            let composer_type = {
                sub_matches.get_one::<String>("composer").unwrap_or(&s)
            };
            let palette = if let Some(path) = palette_path {
                let mut file = File::options().read(true).open(path).unwrap();
                let mut pal = Vec::new();
                file.read_to_end(&mut pal).unwrap();
                let pal_tup = seq_to_pal(&pal);
                Some(pal_tup)
            } else {
                None
            };
            let frames_ids = sub_matches.get_one::<String>("ids").map(|s| parse_ids(s)).unwrap_or_default();
            let img_size = 800;
            let with_tribe: bool = !sub_matches.get_flag("no_tribe");
            let with_type: bool = !sub_matches.get_flag("no_type");
            let anims_config = AnimationsConfig{img_size, with_tribe, with_type};
            if let Some(c) = ContainerPSFB::from_file(&file_path) {
                let img = {
                    match composer_type.as_str() {
                        "ul" => {
                            let composer = ULCentreComposer{vertical: 5, horizontal: 5};
                            draw_anim_frames(&anim_seq_vec, &c, &palette, &frames_ids, &composer, &anims_config)
                        },
                        _ => {
                            let composer = URCentreComposer{vertical: 5, horizontal: 5};
                            draw_anim_frames(&anim_seq_vec, &c, &palette, &frames_ids, &composer, &anims_config)
                        },
                    }
                };
                write_dyn_img_stdout(&img, DEFAULT_IMG_FORMAT);
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
                let pal_tup = seq_to_pal(&pal);
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
                    let (start, num) = match (start_num, num) {
                        (Some(i), Some(n)) => {
                            (i as usize, n as usize)
                        }
                        (None, Some(n)) => {
                            (0, n as usize)
                        }
                        (Some(i), None) => {
                            (i as usize, 0)
                        }
                        (None, None) => {
                            (0, c.len())
                        }
                    };
                    if num <= 1 {
                        if let Some(image) = c.get_image(start) {
                            let img = draw_image(&palette, image);
                            write_dyn_img_stdout(&img, DEFAULT_IMG_FORMAT);
                        }
                    } else {
                        let prefix_opt: Option<PathBuf> = sub_matches.get_one("prefix").cloned();
                        if let Some(prefix) = prefix_opt {
                            draw_sprites(&c, start, num, &prefix, &palette);
                        } else {
                            let img = draw_sprites_img(&c, start, num, &palette);
                            write_dyn_img_stdout(&img, DEFAULT_IMG_FORMAT);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
