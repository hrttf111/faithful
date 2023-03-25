use std::path::Path;
use std::fs::File;
use std::io::Read;

use crate::pop::types::Image;

/******************************************************************************/

pub fn parse_bl_sprite(width: usize, height: usize, data: &[u8]) -> Vec<Image> {
    let sprite_size = width * height;
    let sprite_num = ((data.len() as f32) / (sprite_size as f32)) as usize;
    let mut sprites = Vec::new();
    for i in 0..sprite_num {
        let offset = i * sprite_size;
        let data = data[offset..(offset+sprite_size)].to_vec();
        sprites.push(Image{width, height, data})
    }
    sprites
}

pub fn parse_bl320(path: &Path) -> Vec<Image> {
    let mut file = File::options().read(true).open(path).unwrap();
    let mut buf = Vec::new();
    let _file_size = file.read_to_end(&mut buf).unwrap();
    parse_bl_sprite(256, 256, &buf)
}

pub fn parse_bl160(width: usize, height: usize, path: &Path) -> Vec<Image> {
    let mut file = File::options().read(true).open(path).unwrap();
    let mut buf = Vec::new();
    let _file_size = file.read_to_end(&mut buf).unwrap();
    parse_bl_sprite(width, height, &buf)
}

pub fn make_bl320_tex(path: &Path, pal: &[u8]) -> (usize, usize, Vec<u8>) {
    let sprites = parse_bl320(path);
    let (width, height, sprite_height) = {
        let mut width = 0;
        let mut height = 0;
        let mut sprite_height = 0;
        for sprite in &sprites {
            width = std::cmp::max(width, sprite.width);
            height += sprite.height;
            sprite_height = sprite.height;
        }
        (width, height, sprite_height)
    };

    let sprite_size = width * sprite_height * 4;
    let mut texture = vec![0u8; width * height * 4];
    for (k, sprite) in sprites.iter().enumerate() {
        let start = k * sprite_size;
        for i in 0..sprite_height {
            for j in 0..width {
                let palette_index = sprite.data[i * width + j] as usize;
                let palette_index = palette_index * 4;
                let buf: &[u8] = &pal[palette_index..=(palette_index+3)];
                let tex_index = start + i * width * 4 + j * 4;
                if palette_index == 0 {
                    texture[tex_index] = 0;
                    texture[tex_index+1] = 0;
                    texture[tex_index+2] = 0;
                    texture[tex_index+3] = 255;
                } else {
                    texture[tex_index] = buf[0];
                    texture[tex_index+1] = buf[1];
                    texture[tex_index+2] = buf[2];
                    texture[tex_index+3] = buf[3];
                }
            }
        }
    }

    (width, height, texture)
}
