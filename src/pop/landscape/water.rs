use crate::pop::level::GlobeTextureParams;
use crate::pop::types::{Image, ImageStorage};

/*
 * Bigf0 contains land and water textures. The texture is 3 dimensional.
 * It consists of 1152 x 256 bytes. 1152 is a max number of heights.
 * Each byte in bigf0 memory points to a location in cliff0 or directly to color in palette.
 * Each height level consists of 256 bytes which represent texture. Values from disp0 are used as
 * an index to that array. Disp0 values are in range 0x00-0x7f. They address only a half of each
 * height texture. Another half contains shadowed pixel, to emulate shadows.
 * The most basic way of get texture is to use following formula:
 *  height * 256 + disp0[position]
 * Without displacement resulting texture is very plain and unnatural.
 */
pub fn texture_water(offset_counter: usize, params: &GlobeTextureParams) -> Image {
    let width = 256;
    let mut img = Image::alloc(width, width);
    for i in 0..width {
        for j in 0..width {
            let offset_param = ((offset_counter + i) & 0xff) * 0x100;
            let disp_val = (params.disp0[offset_param + j] as i32) * 0x1a9;
            let disp_val = if disp_val < 0 { -disp_val } else { disp_val };
            //let disp_val = ((disp_val as usize) & 0xfffffc03) >> 2;
            let disp_val = unsafe {
                let k = std::mem::transmute::<i32, u32>(disp_val) & 0xfffffc03;
                std::mem::transmute::<u32, i32>(k) >> 2
            };
            let index = (0x4b80 + disp_val) as usize;
            img.set_pixel(j, i, params.bigf0[index]);
        }
    }
    img
}
