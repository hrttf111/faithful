use std::io::Read;
use std::iter::zip;

use crate::pop::types::{BinDeserializer, Image};

/******************************************************************************/

pub struct SpritePSFB {
    pub index: usize,
    pub offset: usize,
    pub width: u16,
    pub height: u16,
}

impl SpritePSFB {
    pub fn to_image(&self, data_in: &[u8]) -> Image {
        let width = self.width as usize;
        let height = self.height as usize;

        let mut source_index = 0;
        let mut height_index = 0;
        while data_in[source_index] == 0 && height_index < height {
            source_index += 1;
            height_index += 1;
        }
        let size = width * height;
        let mut data = vec![0u8; size];

        while height_index < height {
            let mut dest_index = 0;
            while data_in[source_index] != 0 {
                let val: i8 = unsafe {
                    std::mem::transmute::<u8, i8>(data_in[source_index])
                };
                if val <= 0 {
                    dest_index += (-val) as usize;
                } else {
                    let val = val as usize;
                    let line = {
                        let line_start = source_index + 1;
                        let line_end = source_index + val;
                        &data_in[line_start..=line_end]
                    };
                    let dest_index_start = height_index * width + dest_index;
                    for (i, v) in zip(dest_index_start.., line) {
                        data[i] = *v;
                    }
                    dest_index += val;
                    source_index += val;
                }
                source_index += 1;
            }
            source_index += 1;
            height_index += 1;
        }
        Image{width, height, data}
    }
}

impl BinDeserializer for SpritePSFB {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> where Self: Sized {
        let mut buf_u16 = [0u8; 2];
        reader.read_exact(&mut buf_u16).unwrap();
        let width = u16::from_le_bytes(buf_u16);
        reader.read_exact(&mut buf_u16).unwrap();
        let height = u16::from_le_bytes(buf_u16);
        let mut buf_u32 = [0u8; 4];
        reader.read_exact(&mut buf_u32).unwrap();
        let offset = u32::from_le_bytes(buf_u32);
        Some(Self{index: 0, offset: offset as usize, width, height})
    }
}

pub struct ContainerPSFB {
    header_size: usize,
    sprites: Vec<SpritePSFB>,
    data: Vec<u8>,
}

impl ContainerPSFB {
    pub fn len(&self) -> usize {
        self.sprites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sprites.is_empty()
    }

    pub fn size(&self) -> usize {
        self.header_size + self.data.len()
    }

    pub fn sprites_info(&self) -> &[SpritePSFB] {
        &self.sprites
    }

    pub fn get_image(&self, index: usize) -> Option<Image> {
        if let Some(s) = self.sprites.get(index) {
            let offset = s.offset - self.header_size;
            return Some(s.to_image(&self.data[offset..]));
        }
        None
    }
}

impl BinDeserializer for ContainerPSFB {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> where Self: Sized {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf).unwrap();
        let marker = u32::from_le_bytes(buf);
        if marker != 0x42465350 { // note(): "PSFB" in hex
            return None;
        }
        reader.read_exact(&mut buf).unwrap();
        let sprite_num = u32::from_le_bytes(buf) as usize;
        let header_size = 8 + 8 * sprite_num;
        let mut sprites = Vec::new();
        for i in 0..sprite_num {
            if let Some(mut sprite) = SpritePSFB::from_reader(reader) {
                sprite.index = i;
                sprites.push(sprite);
            }
        }
        let mut data = Vec::new();
        reader.read_to_end(&mut data).unwrap();
        Some(Self{header_size, sprites, data})
    }
}

/******************************************************************************/
