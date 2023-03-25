use crate::pop::landscape::common::LandPos;

/******************************************************************************/

pub fn texture_minimap(width: usize, flag: bool, land: &[LandPos], bigf0: &[u8]) -> Vec<u8> {
    let mut texture = vec![0; width * width];
    let default_val = 0;
    for i in 0..width {
        let h_offset = i * width;
        for j in 0..width {
            let land_pos = &land[h_offset + j];
            let tex_pos = h_offset + j;
            texture[tex_pos] = if flag || ((land_pos.flags & 8) != 0) {
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
            }
        }
    }
    texture
}
