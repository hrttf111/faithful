use crate::pop::landscape::common::{LandPosPoint, LandscapeFull};
use crate::pop::types::{Image, ImageStorage};

/******************************************************************************/

pub fn texture_minimap_storage<'a, S, I>(flag: bool, land_iter: &mut I, bigf0: &[u8], storage: &mut S)
    where S: ImageStorage, I: Iterator<Item=LandPosPoint<'a>>{
    let default_val = 0;
    for pos_point in land_iter {
        let land_pos = pos_point.pos;
        let val = if flag || ((land_pos.flags & 8) != 0) {
            if land_pos.c_1 == 0 {
                bigf0[0x7840 + (land_pos.height as usize)*0x100]
            } else {
                let v = {
                    let v = land_pos.height + 0x8c;
                    if v > 0x47f {
                        0x47f
                    } else {
                        v as usize
                    }
                };
                let index = {
                    let index = v*0x100 + (land_pos.brightness as usize)*0x100;
                    if index >= bigf0.len() {
                        bigf0.len() - 1
                    } else {
                        index
                    }
                };
                bigf0[index]
            }
        } else {
            default_val
        };
        let i = pos_point.y;
        let j = pos_point.x;
        storage.set_pixel(j, i, val);
    }
}

pub fn texture_minimap(width: usize, flag: bool, land: &LandscapeFull, bigf0: &[u8]) -> Image {
    let mut image = Image::alloc(width, width);
    texture_minimap_storage(flag, &mut land.iter(), bigf0, &mut image);
    image
}
