use std::path::Path;
use std::fs::File;
use std::io::Read;

use crate::pop::types::{ImageStorage, ImageStorageSource, AllocatorEqual, pal_image_allocator_1d_vertical};

/******************************************************************************/

fn read_textures_seq<R: Read, P: ImageStorageSource>(info: (usize, usize), reader: &mut R, p: &mut P, data: &mut [u8]) {
    while let Ok(()) = reader.read_exact(data) {
        if let Some(s) = p.get_storage(&info) {
            s.set_image(data);
        }
    }
}

pub fn read_textures<R: Read, P: ImageStorageSource, A: AllocatorEqual<P>>(allocator: &A, reader: &mut R, total_size: usize, width: usize, height: usize) -> P {
    let texture_size = width * height;
    let texture_num = ((total_size as f32) / (texture_size as f32)) as usize;
    let info = (width, height);
    let mut p = allocator.alloc_equal(&info, texture_num);
    let mut data = vec![0u8; texture_size];
    read_textures_seq(info, reader, &mut p, &mut data);
    p
}

pub fn read_bl320<P: ImageStorageSource, A: AllocatorEqual<P>>(allocator: &A, path: &Path) -> P {
    let mut file = File::options().read(true).open(path).unwrap();
    let file_size = file.metadata().unwrap().len() as usize;
    let width = 256;
    let height = 256;
    read_textures(allocator, &mut file, file_size, width, height)
}

pub fn read_bl160<P: ImageStorageSource, A: AllocatorEqual<P>>(width: usize, height: usize, allocator: &A, path: &Path) -> P {
    let mut file = File::options().read(true).open(path).unwrap();
    let file_size = file.metadata().unwrap().len() as usize;
    read_textures(allocator, &mut file, file_size, width, height)
}

pub fn make_bl320_texture_rgba(path: &Path, pal: &[u8]) -> (usize, usize, Vec<u8>) {
    let allocator = pal_image_allocator_1d_vertical(pal);
    let provider = read_bl320(&allocator, path);
    let image = provider.get_image();
    (image.width, image.height, image.data)
}
