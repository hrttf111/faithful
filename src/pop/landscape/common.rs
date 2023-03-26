use crate::pop::level::Landscape;

/******************************************************************************/

#[derive(Clone)]
pub struct LandPos
{
    pub flags: u32,
    pub height: u16,
    pub b_2: u16,
    pub c: u16,
    pub c_1: u8,
    pub c_2: u8,
    pub c_3: u8,
    pub brightness: u8,
    pub ph: u8,
    pub ph_2: u8,
    pub land_adj: bool,
}

impl LandPos {
    fn default() -> LandPos {
        LandPos{flags: 0, height: 0, b_2: 0, c: 0, c_1: 0, c_2: 0, c_3: 0, brightness: 0, ph: 0, ph_2: 0, land_adj: false}
    }

    pub fn from_landscape<const N: usize>(landscape: &Landscape<N>) -> Vec<LandPos> {
        let size = N * N;
        let default_pos = Self::default();
        let mut v = vec![default_pos; size];
        for i in 0..N {
            let p = i * N;
            for j in 0..N {
                v[p+j].c_1 = 0;
                v[p+j].brightness = 0x80;
                v[p+j].height = landscape.height[i][j];
                v[p+j].land_adj = landscape.is_land_adj(i, j);
            }
        }
        v
    }

    pub fn from_landscape_sun<const N: usize>(landscape: &Landscape<N>) -> Vec<LandPos> {
        let sunlight_var_1 = 0x93;
        let sunlight_var_2 = 0x93;
        let sunlight_var_3 = 0x93;
        let mut v = Self::from_landscape(landscape);
        for i in 0..N {
            let p = i * N;
            for j in 0..N {
                let ch: i32 = landscape.height[i][j] as i32;
                let h1: i32 = landscape.height[(i+1)%N][j] as i32;
                let h2: i32 = landscape.height[i][(j+1)%N] as i32;
                let b = sunlight_var_3 + (h1 - ch) * sunlight_var_2 - (ch - h2) * sunlight_var_1;
                let b = (b as f64) / (0x15e as f64) + (v[p+j].brightness as f64);
                let b = b.clamp(0.0, 255.0) as u8;
                v[p+j].brightness = b;
            }
        }
        v
    }
}

/******************************************************************************/

/*
 * p1 - p2
 * |    |
 * p3 - p4
 *
 * p1 - ul
 * p2 - ur
 * p3 - bl
 * p4 - br
 */
pub struct LandPosQuad<'a> {
    pub x: u16, // horizontal
    pub y: u16, // vertical
    pub p1: &'a LandPos,
    pub p2: &'a LandPos,
    pub p3: &'a LandPos,
    pub p4: &'a LandPos,
}

pub fn get_height(pos: &LandPos) -> u16 {
    if pos.c_1 != 0 {
        core::cmp::min(pos.height + 0x96, 0x3fe)
    } else if pos.height > 0x0 || pos.land_adj {
        core::cmp::min(pos.height + 0x96, 0x400)
    } else {
        core::cmp::min(pos.height + 0x4b, 0x400)
    }
}

pub struct LandInc {
    start: f32,
    inc_vert: f32, // increments for start
    inc_start: f32,
    inc_horz: f32, // increments for each line
}

impl LandInc
{
    pub fn mk_land_inc(p1: f32, p2: f32, p3: f32, p4: f32, n: f32) -> Self {
        let start = p1;
        let inc_vert = (p3 - p1) / n;
        let inc_start = (p2 - p1) / n;
        let inc_horz = ((p4 - p3) / n - inc_start) / n;

        Self { start, inc_vert, inc_start, inc_horz }
    }

    pub fn mk_land_inc8(p1: u8, p2: u8, p3: u8, p4: u8, n: f32) -> Self {
        Self::mk_land_inc(p1 as f32, p2 as f32, p3 as f32, p4 as f32, n)
    }

    pub fn inc(&self, i: usize) -> (f32, f32) {
        let start = self.start + self.inc_vert * (i as f32);
        let inc_line = self.inc_start + self.inc_horz * (i as f32);
        (start, inc_line)
    }

    pub fn inc_line(&self, i: usize, j: usize) -> f32 {
        let (start, inc_line) = self.inc(i);
        start + inc_line * (j as f32)
    }
}

pub trait LandTile {
    fn tile_width(&self) -> usize;
    fn set_texel(&mut self, i: usize, j: usize, val: u8);
}

pub struct LandTileSlice<'a> {
    texture: &'a mut[u8],
    start: usize,
    line_width: usize,
    tile_width: usize,
}

impl<'a> LandTileSlice<'a> {
    pub fn new(texture: &'a mut[u8], start: usize, line_width: usize, tile_width: usize) -> Self {
        Self{texture, start, line_width, tile_width}
    }

}

impl<'a> LandTile for LandTileSlice<'a> {
    fn tile_width(&self) -> usize {
        self.tile_width
    }

    fn set_texel(&mut self, i: usize, j: usize, val: u8) {
        let index: usize = self.start + self.line_width * i;
        self.texture[index + j] = val;
    }
}

/******************************************************************************/

pub trait LandTileProvider {
    type Tile: LandTile;

    fn next_tile(&mut self, i: usize, j: usize) -> &mut Self::Tile;
}

/******************************************************************************/

pub struct LandTileSliceProvider<'a> {
    texture: &'a mut[u8],
    start: usize,
    line_width: usize,
    tile_width: usize,
    tile_h_num: usize,
}

impl<'a> LandTileSliceProvider<'a> {
    pub fn new(texture: &'a mut[u8], tile_h_num: usize, tile_width: usize) -> Self {
        let line_width = tile_h_num * tile_width;
        Self{texture, start: 0, line_width, tile_width, tile_h_num}
    }
}

impl<'a> LandTile for LandTileSliceProvider<'a> {
    fn tile_width(&self) -> usize {
        self.tile_width
    }

    fn set_texel(&mut self, i: usize, j: usize, val: u8) {
        let index: usize = self.start + self.line_width * i;
        self.texture[index + j] = val;
    }
}

impl<'a> LandTileProvider for LandTileSliceProvider<'a> {
    type Tile = Self;

    fn next_tile(&mut self, i: usize, j: usize) -> &mut Self::Tile {
        self.start = i * self.tile_h_num * (self.tile_width * self.tile_width) + j * self.tile_width;
        self
    }
}

/******************************************************************************/

pub trait DispProvider {
    fn val(&self, i: usize, j: usize) -> i8;
    fn val_adjacent(&self, i: usize, j: usize) -> f32;
    fn update(&mut self, disp0: &[i8], pos: &LandPosQuad);
}

/******************************************************************************/
