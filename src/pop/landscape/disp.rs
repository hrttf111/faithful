use crate::pop::level::GlobeTextureParams;
use crate::pop::types::{Image, ImageStorage};

/*
 * Disp memory consists of 256x256 (65k) bytes.
 * Horizontal blocks consist of 32 bytes. In each vertical block there are 8 horizontal blocks.
 * Thus vertical block is 256 bytes long. There are 256 of vetical blocks in disp memory.
 * Disp memory could be understood as a matrix of 256x256 elements.
 * 0x0000 v1   - |h1|h2|..|h8|
 * 0x0100 v2   - |h1|h2|..|h8|
 * ...
 * 0xff00 v255 - |h1|h2|..|h8|
 * Each byte is a displacement into bigf0 array. Usually it is used in conjunction with other
 * parameters. The only purpose such displacement is to create landscape textures based on
 * height and other parameters of each individual land point.
 * Max size of texture is 32x32 bytes (pixels). Landscape position (128x128 points) is used
 * as an initial index into displacement memory. For each pixel of texture value is taken
 * according to a certain formula:
 *  - only 3 lowest bytes are used as x/y coordinates for land pos
 *  - it means that texture layout is repeated 16x16 for the whole map
 *  - x coordinate of land position makes initial index: index += x * 0x20
 *  - y coordinate of land position makes initial index: index += y * 0x2000
 *  - combined: index = x * 0x20 + y * 0x2000
 *  - then for each horizontal line of texture 32 bytes are copied starting from index
 *  - after copying each line an index is incremented by 0x100
 * For smaller textures (16x16, 8x8) only 32/N elements of memory could be used. For example,
 * for texture 8x8 only each 4th byte of horizontal memory is used and each 4th vertical block.
 */
pub fn texture_disp_full(params: &GlobeTextureParams) -> Vec<u8> {
    let width = 256;
    let mut texture = vec![0; width * width];
    for i in 0..width {
        for j in 0..width {
            let offset_param = i * 0x100;
            let disp_val = params.disp0[offset_param + j];
            texture[i*width + j] = disp_val as u8;
        }
    }
    texture
}

pub fn texture_disp_quarter(params: &GlobeTextureParams) -> Vec<u8> {
    let points = 8;
    let width = 8 * points;
    let mut texture = vec![0; width * width];
    for i in 0..points {
        for j in 0..points {
            let index = i * width * 8 + j * 8;
            for v in 0..8 {
                let disp_index: usize = ((i as usize & 0x7) << 8) + ((j as usize & 0x7) << 5) + ((v as usize) << 10);
                for h in 0..8 {
                    let disp_val = params.disp0[disp_index + h * 4];
                    texture[index + (v * width) + h] = disp_val as u8;
                }
            }
        }
    }
    texture
}

pub fn texture_bigf0(height: u16, params: &GlobeTextureParams) -> Image {
    let width = 256;
    let mut img = Image::alloc(width, width);
    for i in 0..width {
        for j in 0..width {
            let height_param_x256: i32 = height as i32 * 256;
            img.set_pixel(j, i, params.bigf0[(height_param_x256 + i as i32) as usize]);
        }
    }
    img
}

pub fn texture_bigf0_disp(height: u16, disp_div: i8, disp_add: u8, params: &GlobeTextureParams) -> Vec<u8> {
    let width = 256;
    let mut texture = vec![0; width * width];
    for i in 0..width {
        for j in 0..width {
            let offset_param = i * 0x100;
            let disp_val = params.disp0[offset_param + j] / disp_div;
            let disp_val = disp_val as i32 + disp_add as i32;
            //let disp_val = disp_val.clamp(0, 255);
            let height_param_x256: i32 = height as i32 * 256;
            texture[i*width + j] = params.bigf0[(height_param_x256 + disp_val) as usize];
        }
    }
    texture
}
