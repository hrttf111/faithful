use std::marker::PhantomData;
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

pub struct LandPosPoint<'a> {
    pub x: usize, // horizontal
    pub y: usize, // vertical
    pub pos: &'a LandPos,
}

pub struct LandscapeFull {
    width: usize,
    data: Vec<LandPos>,
}

pub type LandPosQ<'a> = (usize, usize, LandPosQuad<'a>);

impl LandscapeFull {
    pub fn new(width: usize, data: Vec<LandPos>) -> Self {
        Self{width, data}
    }

    pub fn iter(&self) -> LandPosIterator<LandPosPoint> {
        LandPosIterator::new(self)
    }

    pub fn iter_quad(&self) -> LandPosIterator<LandPosQ> {
        LandPosIterator::new(self)
    }
}

pub struct LandPosIterator<'a, T> {
    pos_width: usize,
    pos_height: usize,
    landscape: &'a LandscapeFull,
    phantom: PhantomData<T>,
}

impl<'a, T> LandPosIterator<'a, T> {
    pub fn new(landscape: &'a LandscapeFull) -> Self {
        LandPosIterator{pos_width: 0, pos_height: 0, landscape, phantom: PhantomData}
    }
}

impl<'a> Iterator for LandPosIterator<'a, LandPosPoint<'a>> {
    type Item = LandPosPoint<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos_height >= self.landscape.width {
            return None;
        }
        let pos = &self.landscape.data[self.pos_height * self.landscape.width + self.pos_width];
        let ret = Some(LandPosPoint{x: self.pos_width, y: self.pos_height, pos});
        self.pos_width += 1;
        if self.pos_width >= self.landscape.width {
            self.pos_width = 0;
            self.pos_height += 1;
        }
        ret
    }
}

impl<'a> Iterator for LandPosIterator<'a, LandPosQ<'a>> {
    type Item = LandPosQ<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos_height >= self.landscape.width {
            return None;
        }
        let width = self.landscape.width;
        let i = self.pos_height;
        let j = self.pos_width;
        let index_1 = i * width + j;
        let index_2 = i * width + ((j + 1) % width);
        let index_3 = ((i + 1) % width) * width + j;
        let index_4 = ((i + 1) % width) * width + ((j + 1) % width);
        // Set i+1 to align with texture in pop3
        let pos = LandPosQuad {x: (j & 0x7) as u16, y: ((i+1) & 0x7) as u16
            , p1: &self.landscape.data[index_1]
            , p2: &self.landscape.data[index_2]
            , p3: &self.landscape.data[index_3]
            , p4: &self.landscape.data[index_4]
        };
        let ret = Some((self.pos_width, self.pos_height, pos));
        self.pos_width += 1;
        if self.pos_width >= self.landscape.width {
            self.pos_width = 0;
            self.pos_height += 1;
        }
        ret
    }
}

/******************************************************************************/

pub trait LandTile {
    fn tile_width(&self) -> usize;
    fn tile_height(&self) -> usize;

    fn c1(&self, x: usize, y: usize) -> f32;
    fn brightness(&self, x: usize, y: usize) -> f32;
    fn height(&self, x: usize, y: usize) -> f32;
}

pub struct LandTileQuad<'a> {
    width: usize,
    pos: &'a LandPosQuad<'a>,
    c1_inc: LandInc,
    brightness_inc: LandInc,
    height_inc: LandInc,
}

impl<'a> LandTileQuad<'a> {
    pub fn new(n: usize, pos: &'a LandPosQuad<'a>) -> LandTileQuad {
        let c1_inc = pos.c1_inc(n);
        let brightness_inc = pos.brightness_inc(n);
        let height_inc = pos.height_inc(n);
        Self{width: n, pos, c1_inc, brightness_inc, height_inc}
    }

    pub fn coord_x(&self) -> usize {
        self.pos.x as usize
    }

    pub fn coord_y(&self) -> usize {
        self.pos.y as usize
    }
}

impl<'a> LandTile for LandTileQuad<'a> {
    fn tile_width(&self) -> usize {
        self.width
    }

    fn tile_height(&self) -> usize {
        self.width
    }

    fn c1(&self, x: usize, y: usize) -> f32 {
        self.c1_inc.inc_line(x, y)
    }

    fn brightness(&self, x: usize, y: usize) -> f32 {
        self.brightness_inc.inc_line(x, y)
    }

    fn height(&self, x: usize, y: usize) -> f32 {
        self.height_inc.inc_line(x, y)
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

impl<'a> LandPosQuad<'a> {
    pub fn c1_inc(&self, n: usize) -> LandInc {
        LandInc::mk_land_inc8(self.p1.c_1, self.p2.c_1, self.p3.c_1, self.p4.c_1, n as f32)
    }

    pub fn brightness_inc(&self, n: usize) -> LandInc {
        LandInc::mk_land_inc8(self.p1.brightness
                              , self.p2.brightness
                              , self.p3.brightness
                              , self.p4.brightness
                              , n as f32)
    }

    pub fn height_inc(&self, n: usize) -> LandInc {
        let height_1 = get_height(self.p1) as f32;
        let height_2 = get_height(self.p2) as f32;
        let height_3 = get_height(self.p3) as f32;
        let height_4 = get_height(self.p4) as f32;
        LandInc::mk_land_inc(height_1, height_2, height_3, height_4, n as f32)
    }
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

impl LandInc {
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

/******************************************************************************/

pub trait DispProvider {
    fn val(&self, i: usize, j: usize) -> i8;
    fn val_adjacent(&self, i: usize, j: usize) -> f32;
    fn update(&mut self, disp0: &[i8], pos: &LandPosQuad);
}

/******************************************************************************/
