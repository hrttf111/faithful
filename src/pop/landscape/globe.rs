use crate::pop::level::GlobeTextureParams;
use crate::pop::landscape::common::{LandPos, LandPosQuad, LandInc, get_height, LandTile, LandTileSlice};

fn make_tile<T: LandTile>(params: &GlobeTextureParams
                          , pos: &LandPosQuad
                          , tile: &mut T) {
    let n = tile.tile_width();

    let height_1 = get_height(pos.p1) as f32;
    let height_2 = get_height(pos.p2) as f32;
    let height_3 = get_height(pos.p3) as f32;
    let height_4 = get_height(pos.p4) as f32;

    let c1_inc = LandInc::mk_land_inc8(pos.p1.c_1, pos.p2.c_1, pos.p3.c_1, pos.p4.c_1, n as f32);
    let c4_inc = LandInc::mk_land_inc8(pos.p1.c_4, pos.p2.c_4, pos.p3.c_4, pos.p4.c_4, n as f32);
    let height_inc = LandInc::mk_land_inc(height_1, height_2, height_3, height_4, n as f32);

    let mut disp: [i8; 64] = [0; 64];
    for i in 0..n {
        let disp_index: usize = ((pos.x as usize & 0x7) << 13) + ((pos.y as usize & 0x7) << 5) + i*4;
        for j in 0..n {
            disp[i * 8 + j] = params.disp0[disp_index + ((j as usize) << 10)];
        }
    }

    for i in 0..n {
        for j in 0..n {
            let hp = height_inc.inc_line(i, j);
            let height_param: i32 = hp as i32;
            let height_param_x256: i32 = height_param * 256;

            let c4_val = c4_inc.inc_line(i, j);
            let disp_val = disp[i * 8 + j];
            let c4_disp_param = (disp_val as f32 / 4.0 + c4_val) as i32;
            let c4_disp_param = c4_disp_param.clamp(0, 255);

            let c1_param = c1_inc.inc_line(i, j) / 4.0;
            let c1_param: usize = c1_param.max(0.0) as usize;

            let static_component: i32 = params.static_landscape_array[height_param as usize] as i32 * (disp_val as i32);
            let static_component = unsafe {
                let k = std::mem::transmute::<i32, u32>(static_component) & 0xfffffc03;
                std::mem::transmute::<u32, i32>(k) >> 2
            };
            let height_component = height_param_x256 & 0x7fffff00;
            let big_index = (static_component + height_component + c4_disp_param) as usize;
            if big_index > params.bigf0.len() {
                panic!("{height_component:} | {static_component:?} | {disp_val:?} | {c4_disp_param:?} | {big_index:?}");
            }
            let big_component: usize = (params.bigf0[big_index]).into();
            let texel = params.cliff0[big_component + c1_param * 0x80];
            tile.set_texel(i, j, texel);
        }
    }
}

pub fn texture_globe(width: usize
                     , land: &[LandPos]
                     , params: &GlobeTextureParams) -> Vec<u8> {
    let mut texture = vec![0; 64 * width * width];

    for i in 0..(width) {
        for j in 0..(width) {
            let index_1 = i * width + j;
            let index_2 = i * width + (j + 1) % width;
            let index_3 = ((i + 1) % width) * width + j;
            let index_4 = ((i + 1) % width) * width + (j + 1) % width;
            let start = i * width * 64 + j * 8;
            let pos = LandPosQuad {x: (j & 0x7) as u16, y: (i & 0x7) as u16
                , p1: &land[index_1], p2: &land[index_2], p3: &land[index_3], p4: &land[index_4]};
            let mut tile = LandTileSlice::new(&mut texture, start, 8 * width, 8);
            make_tile(params, &pos, &mut tile);
        }
    }

    texture
}
