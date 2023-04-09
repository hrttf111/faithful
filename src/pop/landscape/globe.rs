use crate::pop::level::GlobeTextureParams;
use crate::pop::types::{ImageTileSource, Image, TiledComposer, ImageSourceComposed};
use crate::pop::landscape::common::{LandPosQuad, LandscapeFull, DispProvider};
use crate::pop::landscape::land::render_landscape;

struct DispProvider8<'a> {
    x: usize,
    y: usize,
    disp: &'a [i8],
}

impl<'a> DispProvider8<'a> {
    pub fn new(disp: &'a [i8]) -> Self {
        Self{x: 0, y: 0, disp}
    }
}

impl<'a> DispProvider for DispProvider8<'a> {
    fn val(&self, i: usize, j: usize) -> i8 {
        let disp_index: usize = ((self.x as usize & 0x7) << 13) + ((self.y as usize & 0x7) << 5) + i*4;
        self.disp[disp_index + ((j as usize) << 10)]
    }

    fn val_adjacent(&self, i: usize, j: usize) -> f32 {
        self.val(i, j) as f32
    }

    fn update(&mut self, _disp0: &[i8], pos: &LandPosQuad) {
        self.x = pos.x as usize;
        self.y = pos.y as usize;
    }
}

pub fn texture_globe_provider<'a, P>(land: &LandscapeFull
                                     , params: &GlobeTextureParams
                                     , tile_source: &'a mut P)
where P: ImageTileSource {
    let mut disp = DispProvider8::new(&params.disp0);
    render_landscape(&mut land.iter_quad(), params, &mut disp, tile_source);
}

pub fn texture_globe(width: usize
                     , land: &LandscapeFull
                     , params: &GlobeTextureParams) -> Image {
    let mut tile_source = {
        let n = 8;
        let image = Image::alloc(width * n, width * n);
        let composer = TiledComposer::new(width * n, width * n, n, n);
        ImageSourceComposed::new(composer, image)
    };
    texture_globe_provider(land, params, &mut tile_source);
    tile_source.get_image()
}
