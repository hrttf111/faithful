pub mod common;
pub mod minimap;
pub mod globe;
pub mod land;
pub mod water;
pub mod disp;

/******************************************************************************/

use crate::pop::level::{GlobeTextureParams, LevelPaths, read_landscape_type, Landscape};

use crate::pop::landscape::common::LandPos;
use crate::pop::landscape::land::texture_land;

use std::path::{Path, PathBuf};

pub struct LevelRes {
    pub paths: LevelPaths,
    pub params: GlobeTextureParams,
    pub landscape: Landscape<128>,
}

impl LevelRes {
    pub fn new(base: &Path, level_num: u8, level_type_opt: Option<&str>) -> LevelRes {
        let data_dir = base.join("data");
        let level_dir = base.join("levels");
        let (level_path, level_type) = read_level(&level_dir, level_num);

        let paths = match level_type_opt {
            Some(v) => LevelPaths::from_base(&data_dir, v),
            None => LevelPaths::from_base(&data_dir, &level_type),
        };

        let landscape = Landscape::from_file(&level_path);
        let params = GlobeTextureParams::from_level(&paths);
        LevelRes {
            paths,
            params,
            landscape,
        }
    }
}

pub fn read_level(base: &Path, num: u8) -> (PathBuf, String) {
    let dat_path = LevelPaths::dat_path(base, num);
    let hdr_path = LevelPaths::hdr_path(base, num);
    let s = read_landscape_type(&hdr_path);
    (dat_path, s)
}

pub fn draw_texture_f32(pal: &[u8], width: usize, texture: &[u8]) -> Vec<f32> {
    let mut img = vec![0f32; 3 * width * width];
    for i in 0..width {
        for j in 0..width {
            let palette_index = texture[(i*width + j) as usize] as usize;
            let palette_index = palette_index.min(127) * 4;
            let buf: &[u8] = &pal[palette_index..=(palette_index+2)];
            let img_index = 3 * width * i + 3 * j;
            img[img_index] = buf[0] as f32 / 255.0;
            img[img_index+1] = buf[1] as f32 / 255.0;
            img[img_index+2] = buf[2] as f32 / 255.0;
        }
    }
    img
}

pub fn draw_texture_u8(pal: &[u8], width: usize, texture: &[u8]) -> Vec<u8> {
    let mut img = vec![0u8; 3 * width * width];
    for i in 0..width {
        for j in 0..width {
            let palette_index = texture[(i*width + j) as usize] as usize;
            let palette_index = palette_index.min(127) * 4;
            let buf: &[u8] = &pal[palette_index..=(palette_index+2)];
            let img_index = 3 * width * i + 3 * j;
            img[img_index] = buf[0];
            img[img_index+1] = buf[1];
            img[img_index+2] = buf[2];
        }
    }
    img
}

pub fn make_texture_land(
                     level_res: &LevelRes
                     , _tex_move: Option<(u32, u32)>
                     ) -> Vec<u8> {
    let land_size = 128;
    let params_globe = &level_res.params;
    let land = LandPos::from_landscape(&level_res.landscape);
    texture_land(land_size, &land, params_globe)
}
