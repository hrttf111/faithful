use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

/******************************************************************************/

pub struct BL320Sprite {
    pub size: u8,
    pub data: Vec<u8>,
}

fn mk_based_path(base: &Path, s: String) -> PathBuf {
    let mut base = base.to_path_buf();
    base.push(s);
    base
}

pub fn bl320_path(base: &Path, key: &str) -> PathBuf {
    let key_upper = key.to_uppercase();
    mk_based_path(base, format!("BL320-{key_upper}.DAT"))
}

pub fn parse_bl320(path: &Path) -> Vec<BL320Sprite> {
    let mut file = File::options().read(true).open(path).unwrap();
    let mut buf = Vec::new();
    let file_size = file.read_to_end(&mut buf).unwrap();
    let size = 256;
    let sprite_size = size * size;
    let sprite_num = ((file_size as f32) / (sprite_size as f32)) as usize;
    let mut sprites = Vec::new();
    for i in 0..sprite_num {
        let offset = i * sprite_size;
        let sprite = BL320Sprite{size: size as u8, data: buf[offset..(offset+sprite_size)].to_vec()};
        sprites.push(sprite);
    }
    sprites
}
