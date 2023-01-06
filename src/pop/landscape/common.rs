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
    pub c_4: u8,
    pub ph: u8,
    pub ph_2: u8,
    pub land_adj: bool,
}

impl LandPos {
    fn default() -> LandPos {
        LandPos{flags: 0, height: 0, b_2: 0, c: 0, c_1: 0, c_2: 0, c_3: 0, c_4: 0, ph: 0, ph_2: 0, land_adj: false}
    }

    pub fn from_landscape<const N: usize>(landscape: &Landscape<N>) -> Vec<LandPos> {
        let size = N * N;
        let default_pos = Self::default();
        let mut v = vec![default_pos; size];
        for i in 0..N {
            let p = i * N;
            for j in 0..N {
                v[p+j].c_1 = 0;
                v[p+j].c_4 = 0x80;
                v[p+j].height = landscape.height[i][j];
                v[p+j].land_adj = landscape.is_land_adj(i, j);
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

pub struct LandInc
{
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

/******************************************************************************/
