use crate::pop::level::GlobeTextureParams;
use crate::pop::types::{ImageInfo, ImageStorage, ImageTileSource, Image, TiledComposer, ImageSourceComposed};
use crate::pop::landscape::common::{LandTile, LandTileQuad, LandPosQuad, LandPosQ, LandscapeFull, DispProvider};

struct DispProvider32<'a> {
    x: usize,
    y: usize,
    disp: &'a [i8],
}

impl<'a> DispProvider32<'a> {
    pub fn new(disp: &'a [i8]) -> Self {
        Self{x: 0, y: 0, disp}
    }
}

impl<'a> DispProvider for DispProvider32<'a> {
    fn val(&self, i: usize, j: usize) -> i8 {
        let y = (self.y + i) & 0xff;
        let x = ((self.x + j) & 0xff) << 8;
        self.disp[x + y]
    }

    fn val_adjacent(&self, i: usize, j: usize) -> f32 {
        let disp_val = self.val(i, j);
        let id = if j == 31 { 0 } else { 1 };
        let disp_val_2 = self.val(i+id, j+1);
        (disp_val_2 - disp_val) as f32
    }

    fn update(&mut self, _disp0: &[i8], pos: &LandPosQuad) {
        self.x = (pos.x as usize & 0x7) * 32;
        self.y = (pos.y as usize & 0x7) * 32;
    }
}

pub fn render_land_tile<T, D, I>(params: &GlobeTextureParams
                                 , land_tile: &T
                                 , disp: &D
                                 , image_tile: &mut I)
    where I: ImageInfo + ImageStorage, D: DispProvider, T: LandTile {
    let w = land_tile.tile_width();
    let h = land_tile.tile_height();

    for i in 0..w {
        for j in 0..h {
            let height: i32 = land_tile.height(i, j) as i32;
            let height_x256: i32 = height * 256;

            let brightness = land_tile.brightness(i, j);
            let b_dist = (disp.val_adjacent(i, j) / 4.0 + brightness) as i32;
            let b_dist = b_dist.clamp(0, 255);

            let c1 = land_tile.c1(i, j) / 4.0;
            let c1: usize = c1.max(0.0) as usize;

            let disp_val = disp.val(i, j);
            let static_component: i32 = params.static_landscape_array[height as usize] as i32 * (disp_val as i32);
            let static_component = unsafe {
                let k = std::mem::transmute::<i32, u32>(static_component) & 0xfffffc03;
                std::mem::transmute::<u32, i32>(k) >> 2
            };
            let height_component = height_x256 & 0x7fffff00;
            let big_index = (static_component + height_component + b_dist) as usize;
            if big_index > params.bigf0.len() {
                panic!("{height_component:} | {static_component:?} | {disp_val:?} | {b_dist:?} | {big_index:?}");
            }
            let big_component: usize = (params.bigf0[big_index]).into();
            let val = params.cliff0[big_component + c1 * 0x80];
            image_tile.set_pixel(j, i, val);
        }
    }
}

pub fn render_landscape<'a, I, D, P>(land_iter: &mut I
                                     , params: &GlobeTextureParams
                                     , disp_provider: &'a mut D
                                     , tile_source: &'a mut P)
where I: Iterator<Item=LandPosQ<'a>>, D: DispProvider, P: ImageTileSource {
    for pos in land_iter {
        disp_provider.update(&params.disp0, &pos.2);
        let image_tile = tile_source.next_tile(pos.0, pos.1);
        let land_tile = LandTileQuad::new(image_tile.width(), &pos.2);
        render_land_tile(params, &land_tile, disp_provider, image_tile);
    }
}

pub fn texture_land_provider<'a, P>(land: &LandscapeFull
                                    , params: &GlobeTextureParams
                                    , tile_source: &'a mut P)
where P: ImageTileSource {
    let mut disp = DispProvider32::new(&params.disp0);
    render_landscape(&mut land.iter_quad(), params, &mut disp, tile_source);
}

pub fn texture_land(width: usize
                    , land: &LandscapeFull
                    , params: &GlobeTextureParams) -> Image {
    let mut tile_source = {
        let n = 32;
        let image = Image::alloc(width * n, width * n);
        let composer = TiledComposer::new(width * n, width * n, n, n);
        ImageSourceComposed::new(composer, image)
    };
    texture_land_provider(land, params, &mut tile_source);
    tile_source.get_image()
}
