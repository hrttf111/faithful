use crate::pop::level::GlobeTextureParams;
use crate::pop::landscape::common::{LandPos, LandPosQuad, LandTileProvider, LandTileSliceProvider, DispProvider};
use crate::pop::landscape::land::make_land_image;

struct DispProvider8 {
    disp: [i8; 64],
}

impl DispProvider8 {
    pub fn new() -> Self {
        Self{disp: [0i8; 64]}
    }
}

impl DispProvider for DispProvider8 {
    fn val(&self, i: usize, j: usize) -> i8 {
        self.disp[i * 8 + j]
    }

    fn val_adjacent(&self, i: usize, j: usize) -> f32 {
        self.val(i, j) as f32
    }

    fn update(&mut self, disp0: &[i8], pos: &LandPosQuad) {
        let n = 8;
        for i in 0..n {
            let disp_index: usize = ((pos.x as usize & 0x7) << 13) + ((pos.y as usize & 0x7) << 5) + i*4;
            for j in 0..n {
                self.disp[i * 8 + j] = disp0[disp_index + ((j as usize) << 10)];
            }
        }
    }
}

pub fn texture_globe_provider<'a, P>(width: usize
                                     , land: &[LandPos]
                                     , params: &GlobeTextureParams
                                     , tile_provider: &'a mut P)
where P: LandTileProvider {
    let mut disp = DispProvider8::new();
    make_land_image(width, land, params, &mut disp, tile_provider);
}

pub fn texture_globe(width: usize
                     , land: &[LandPos]
                     , params: &GlobeTextureParams) -> Vec<u8> {
    let mut texture = vec![0; (8 * 8) * width * width];
    let mut provider = LandTileSliceProvider::new(&mut texture, width, 8);
    texture_globe_provider(width, land, params, &mut provider);
    texture
}
