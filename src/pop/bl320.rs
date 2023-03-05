use std::path::Path;
use std::fs::File;
use std::io::Read;

/******************************************************************************/

pub struct BLSprite {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

pub fn parse_bl_sprite(width: usize, height: usize, data: &[u8]) -> Vec<BLSprite> {
    let sprite_size = width * height;
    let sprite_num = ((data.len() as f32) / (sprite_size as f32)) as usize;
    let mut sprites = Vec::new();
    for i in 0..sprite_num {
        let offset = i * sprite_size;
        let sprite = BLSprite{
            width: width as u16
            , height: height as u16
            , data: data[offset..(offset+sprite_size)].to_vec()};
        sprites.push(sprite);
    }
    sprites
}

pub fn parse_bl320(path: &Path) -> Vec<BLSprite> {
    let mut file = File::options().read(true).open(path).unwrap();
    let mut buf = Vec::new();
    let _file_size = file.read_to_end(&mut buf).unwrap();
    parse_bl_sprite(256, 256, &buf)
}

pub fn parse_bl160(width: usize, height: usize, path: &Path) -> Vec<BLSprite> {
    let mut file = File::options().read(true).open(path).unwrap();
    let mut buf = Vec::new();
    let _file_size = file.read_to_end(&mut buf).unwrap();
    parse_bl_sprite(width, height, &buf)
}
