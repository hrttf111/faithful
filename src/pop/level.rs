use std::fs::{File, OpenOptions};
use std::io::Read;

/******************************************************************************/

pub struct LevelPaths {
    pub palette: String,
    pub disp0: String,
    pub bigf0: String,
    pub cliff0: String,
    pub fade0: String,
}

impl LevelPaths {
    pub fn from_base(base: &str, key: &str) -> Self {
        Self {
            palette: format!("{base}/pal0-{key}.dat"),
            disp0: format!("{base}/disp0-{key}.dat"),
            bigf0: format!("{base}/bigf0-{key}.dat"),
            cliff0: format!("{base}/cliff0-{key}.dat"),
            fade0: format!("{base}/fade0-{key}.dat"),
        }
    }

    pub fn dat_path(base: &str, num: u8) -> String {
        format!("{base}/levl2{num:03}.dat")
    }

    pub fn hdr_path(base: &str, num: u8) -> String {
        format!("{base}/levl2{num:03}.hdr")
    }
}

/******************************************************************************/

pub fn read_landscape_type(hdr_path: &str) -> String {
    let hdr_data = read_bin(hdr_path);
    if hdr_data.len() < 70 {
        panic!("Hdr is too small {}", hdr_data.len())
    }
    let type_int = hdr_data[96];
    match type_int {
        0 ..= 9 => {
            let v = 0x30 + type_int;
            std::char::from_u32(v as u32).unwrap().to_string().to_lowercase()
        },
        i if i < 36 => {
            let v = 0x41 + (type_int - 10);
            std::char::from_u32(v as u32).unwrap().to_string().to_lowercase()
        },
        _ => panic!("Wrong landscape type {type_int:?}")
    }
}

/******************************************************************************/

pub fn read_bin(path: &str) -> Vec<u8> {
    let mut f = OpenOptions::new().read(true).open(path).unwrap();
    let mut vec = Vec::new();
    f.read_to_end(&mut vec).unwrap();
    vec
}

#[allow(dead_code)]
fn read_bin16(path: &str) -> Vec<u16> {
    let buf = read_bin(path);
    let mut vec = vec![0; buf.len() / 2];
    for (i, n) in (0..).zip(buf.chunks(2).take(vec.len())) {
        if n.len() == 2 {
            vec[i] = u16::from_le_bytes([n[0], n[1]]);
        }
    }
    vec
}

fn read_bin_i8(path: &str) -> Vec<i8> {
    let buf = read_bin(path);
    let mut v = std::mem::ManuallyDrop::new(buf);
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    unsafe { Vec::from_raw_parts(p as *mut i8, len, cap) }
}

fn read_disp(path: &str) -> Vec<i8> {
    let mut disp = read_bin_i8(path);
    let width = 256;
    for i in 0..width {
        for j in 0..(width/2 - 1) {
            let n = i*width + j;
            let n1 = i*width + (width-1) - j;
            disp.swap(n, n1);
        }
    }
    disp
}

/******************************************************************************/

pub struct GlobeTextureParams {
    pub disp0: Vec<i8>,
    pub cliff0: Vec<u8>,
    pub bigf0: Vec<u8>,
    pub fade0: Vec<u8>,
    pub static_landscape_array: Vec<u16>,
    pub palette: Vec<u8>,
}

impl GlobeTextureParams {
    pub fn from_level(paths: &LevelPaths) -> Self {
        Self {
            bigf0: read_bin(&paths.bigf0),
            cliff0: read_bin(&paths.cliff0),
            disp0: read_disp(&paths.disp0),
            fade0: read_bin(&paths.fade0),
            static_landscape_array: Self::make_static_array(),
            palette: read_bin(&paths.palette),
        }
    }

    pub fn make_static_array() -> Vec<u16> {
        let mut v = vec![0; 1152];
        for (i, elem) in v.iter_mut().enumerate() {
            if i < 128 {
                *elem = 0x140;
            } else if i < 362 {
                *elem = (0xd3d - (1152 - i) * 3) as u16;
            } else {
                *elem = 0x400;
            }
        }
        v
    }
}

/******************************************************************************/

pub struct Landscape<const N: usize> {
    pub height: [[u16; N]; N],
}

impl<const N: usize> Landscape<N> {
    pub fn new() -> Self {
        Self{height: [[0u16; N]; N]}
    }

    pub fn land_size(&self) -> usize {
        N
    }

    fn flip(&mut self) {
        let width = N;
        for i in 0..width {
            for j in 0..(width/2 - 1) {
                let n1 = (width-1) - j;
                let v = self.height[j][i];
                self.height[j][i] = self.height[n1][i];
                self.height[n1][i] = v;
            }
        }
    }

    pub fn from_file(path: &str) -> Self {
        let mut file = File::options().read(true).open(path).unwrap();
        let mut s = Self::new();
        let mut buf = Vec::new();
        let _file_size = file.read_to_end(&mut buf);
        for (i, n) in (0..).zip(buf.chunks(2).take(N*N)) {
            if n.len() == 2 {
                let val = u16::from_le_bytes([n[0], n[1]]);
                s.height[i%N][i/N] = val;
            }
        }
        s.flip();
        s
    }

    pub fn is_land_adj(&self, i: usize, j: usize) -> bool {
        if self.height[i][j] > 0 {
            return false;
        }
        let i_u = (i+1) % N;
        let j_u = (j+1) % N;
        let i_d = if i == 0 { N-1 } else { i - 1 };
        let j_d = if j == 0 { N-1 } else { j - 1 };
        (self.height[i][j_d] > 0) ||
               (self.height[i][j_u] > 0) ||
               (self.height[i_d][j] > 0) ||
               (self.height[i_u][j] > 0) ||
               (self.height[i_u][j_d] > 0) ||
               (self.height[i_u][j_u] > 0) ||
               (self.height[i_d][j_d] > 0) ||
               (self.height[i_d][j_u] > 0)
    }

    pub fn make_shores(&self) -> Self{
        let mut output = Self{height: self.height};
        for i in 0..N {
            for j in 0..N {
                if self.height[i][j] == 0 && self.is_land_adj(i, j) {
                    output.height[i][j] = 1;
                }
            }
        }
        output
    }

    pub fn to_vec(&self) -> Vec<u32> {
        let mut vec = vec![0u32; N*N];
        for i in 0..N {
            for j in 0..N {
                vec[i*N + j] = self.height[i][j] as u32;
            }
        }
        vec
    }
}

impl<const N: usize> Default for Landscape<N> {
    fn default() -> Self {
        Self::new()
    }
}

/******************************************************************************/
