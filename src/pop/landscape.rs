pub mod common;
pub mod minimap;
pub mod globe;
pub mod land;
pub mod water;
pub mod disp;

/******************************************************************************/

use crate::pop::level::LevelRes;

use crate::pop::landscape::common::{LandPos, LandscapeFull};
use crate::pop::landscape::land::texture_land;

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

pub fn make_texture_land(level_res: &LevelRes
                         , _tex_move: Option<(u32, u32)>) -> Vec<u8> {
    let land_size = level_res.landscape.land_size();
    let params_globe = &level_res.params;
    let land = LandPos::from_landscape_sun(&level_res.landscape);
    let landscape = LandscapeFull::new(land_size, land);
    texture_land(land_size, &landscape, params_globe).data
}
