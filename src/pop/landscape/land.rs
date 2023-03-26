use crate::pop::level::GlobeTextureParams;
use crate::pop::landscape::common::{LandPos, LandPosQuad, LandInc, get_height, LandTile, LandTileProvider, LandTileSliceProvider, DispProvider};

const DISP_SIZE: usize = (32 + 1) * (32 + 1) + 1;

struct DispProvider32 {
    disp: [i8; DISP_SIZE],
}

impl DispProvider32 {
    pub fn new() -> Self {
        Self{disp: [0i8; DISP_SIZE]}
    }
}

impl DispProvider for DispProvider32 {
    fn val(&self, i: usize, j: usize) -> i8 {
        self.disp[i * 33 + j]
    }

    fn val_adjacent(&self, i: usize, j: usize) -> f32 {
        let disp_val = self.disp[i * 33 + j];
        let disp_val_2 = self.disp[i * 33 + j + 0x22];
        disp_val_2 as f32 - disp_val as f32
    }

    fn update(&mut self, disp0: &[i8], pos: &LandPosQuad) {
        let n = 32;

        let index_1: usize = ((pos.x as usize & 0x7) << 13) + ((pos.y as usize & 0x7) << 5); // upper left
        let index_2: usize = (((pos.x+1) as usize & 0x7) << 13) + (((pos.y) as usize & 0x7) << 5); // upper right

        let index_3: usize = (((pos.x) as usize & 0x7) << 13) + (((pos.y+1) as usize & 0x7) << 5); // lower left
        let index_4: usize = (((pos.x+1) as usize & 0x7) << 13) + (((pos.y+1) as usize & 0x7) << 5); // lower right

        for i in 0..n {
            for j in 0..n {
                self.disp[i * 33 + j] = disp0[index_1 + i + ((j as usize) << 8)];
            }
            self.disp[i * 33 + n] = disp0[index_2 + i];
            self.disp[32*n + i] = disp0[index_3 + i];
            //disp[32*n + i] = disp0[index_3 + ((i as usize) << 8)];
        }
        self.disp[33*33] = disp0[index_4];
    }
}

pub fn make_land_tile<T: LandTile, D: DispProvider>(params: &GlobeTextureParams
                                                    , pos: &LandPosQuad
                                                    , disp: &D
                                                    , tile: &mut T) {
    let n = tile.tile_width();

    let height_1 = get_height(pos.p1) as f32;
    let height_2 = get_height(pos.p2) as f32;
    let height_3 = get_height(pos.p3) as f32;
    let height_4 = get_height(pos.p4) as f32;

    let c1_inc = LandInc::mk_land_inc8(pos.p1.c_1
                                       , pos.p2.c_1
                                       , pos.p3.c_1
                                       , pos.p4.c_1
                                       , n as f32);
    let brightness_inc = LandInc::mk_land_inc8(pos.p1.brightness
                                               , pos.p2.brightness
                                               , pos.p3.brightness
                                               , pos.p4.brightness
                                               , n as f32);
    let height_inc = LandInc::mk_land_inc(height_1
                                          , height_2
                                          , height_3
                                          , height_4
                                          , n as f32);

    for i in 0..n {
        for j in 0..n {
            let hp = height_inc.inc_line(i, j);
            let height_param: i32 = hp as i32;
            let height_param_x256: i32 = height_param * 256;

            let c4_val = brightness_inc.inc_line(i, j);
            let c4_disp_param = (disp.val_adjacent(i, j) / 4.0 + c4_val) as i32;
            let c4_disp_param = c4_disp_param.clamp(0, 255);

            let c1_param = c1_inc.inc_line(i, j) / 4.0;
            let c1_param: usize = c1_param.max(0.0) as usize;

            let disp_val = disp.val(i, j);
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
            tile.set_texel(i as usize, j as usize, texel);
        }
    }
}

pub fn make_land_image<'a, P, D>(width: usize
                                 , land: &[LandPos]
                                 , params: &GlobeTextureParams
                                 , disp_provider: &'a mut D
                                 , tile_provider: &'a mut P)
where P: LandTileProvider, D: DispProvider {
    for i in 0..width {
        for j in 0..width {
            let index_1 = i * width + j;
            let index_2 = i * width + ((j + 1) % width);
            let index_3 = ((i + 1) % width) * width + j;
            let index_4 = ((i + 1) % width) * width + ((j + 1) % width);
            // Set i+1 to align with texture in pop3
            let pos = LandPosQuad {x: (j & 0x7) as u16, y: ((i+1) & 0x7) as u16
                , p1: &land[index_1], p2: &land[index_2], p3: &land[index_3], p4: &land[index_4]};
            disp_provider.update(&params.disp0, &pos);
            let tile = tile_provider.next_tile(i, j);
            make_land_tile(params, &pos, disp_provider, tile);
        }
    }
}

pub fn texture_land_provider<'a, P>(width: usize
                                    , land: &[LandPos]
                                    , params: &GlobeTextureParams
                                    , tile_provider: &'a mut P)
where P: LandTileProvider {
    let mut disp = DispProvider32::new();
    make_land_image(width, land, params, &mut disp, tile_provider);
}

pub fn texture_land(width: usize
                    , land: &[LandPos]
                    , params: &GlobeTextureParams) -> Vec<u8> {
    let mut texture = vec![0; (32 * 32) * width * width];
    let mut provider = LandTileSliceProvider::new(&mut texture, width, 32);
    texture_land_provider(width, land, params, &mut provider);
    texture
}
