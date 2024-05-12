use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek};

use crate::pop::types::BinDeserializer;
use crate::pop::units::{UnitRaw, TribeConfigRaw};

/******************************************************************************/

pub struct LevelPaths {
    pub palette: PathBuf,
    pub disp0: PathBuf,
    pub bigf0: PathBuf,
    pub cliff0: PathBuf,
    pub fade0: PathBuf,
    pub bl320: PathBuf,
    pub bl160: PathBuf,
}

fn mk_based_path(base: &Path, s: String) -> PathBuf {
    let mut base = base.to_path_buf();
    base.push(s);
    base
}

impl LevelPaths {
    pub fn from_base(base: &Path, key: &str) -> Self {
        let key_upper = key.to_uppercase();
        Self {
            palette: mk_based_path(base, format!("pal0-{key}.dat")),
            disp0: mk_based_path(base, format!("disp0-{key}.dat")),
            bigf0: mk_based_path(base, format!("bigf0-{key}.dat")),
            cliff0: mk_based_path(base, format!("cliff0-{key}.dat")),
            fade0: mk_based_path(base, format!("fade0-{key}.dat")),
            bl320: mk_based_path(base, format!("BL320-{key_upper}.DAT")),
            bl160: mk_based_path(base, format!("BL160-{key_upper}.DAT")),
        }
    }

    pub fn from_default_dir(base: &Path, key: &str) -> Self {
        let data_dir = base.join("data");
        Self::from_base(&data_dir, key)
    }

    pub fn dat_path(base: &Path, num: u8) -> PathBuf {
        mk_based_path(base, format!("levl2{num:03}.dat"))
    }

    pub fn hdr_path(base: &Path, num: u8) -> PathBuf {
        mk_based_path(base, format!("levl2{num:03}.hdr"))
    }
}

pub struct ObjectPaths {
    pub objs0_dat: PathBuf,
    pub objs0_ver: PathBuf,
    pub pnts0: PathBuf,
    pub facs0: PathBuf,
    pub morph0: PathBuf,
    pub shapes: PathBuf,
}

impl ObjectPaths {
    pub fn from_base(base: &Path, key: &str) -> Self {
        Self {
            //objs0_dat: mk_based_path(base, format!("objs0-{key}.dat")),
            objs0_dat: mk_based_path(base, format!("OBJS0-{key}.DAT")),
            objs0_ver: mk_based_path(base, format!("objs0-{key}.ver")),
            pnts0: mk_based_path(base, format!("PNTS0-{key}.DAT")),
            facs0: mk_based_path(base, format!("FACS0-{key}.DAT")),
            morph0: mk_based_path(base, format!("morph0-{key}.dat")),
            shapes: mk_based_path(base, "SHAPES.DAT".to_string()),
        }
    }

    pub fn from_default_dir(base: &Path, key: &str) -> Self {
        let data_dir = base.join("objects");
        Self::from_base(&data_dir, key)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
pub struct Sunlight {
    pub v1: u8,
    pub v2: u8,
    pub v3: u8,
}

impl Sunlight {
    pub fn new(v1: u8, v2: u8, v3: u8) -> Self {
        Sunlight {v1, v2, v3}
    }

    pub fn from_reader<R: Read>(reader: &mut R) -> Self {
        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf).unwrap();
        Self::new(buf[0], buf[1], buf[2])
    }
}

/******************************************************************************/

pub struct LevelRes {
    pub paths: LevelPaths,
    pub params: GlobeTextureParams,
    pub landscape: Landscape<128>,
    pub tribes: Vec<TribeConfigRaw>,
    pub sunlight: Sunlight,
    pub units: Vec<UnitRaw>,
}

impl LevelRes {
    pub fn new(base: &Path, level_num: u8, level_type_opt: Option<&str>) -> LevelRes {
        let level_dir = base.join("levels");
        let (level_path, level_type) = read_level(&level_dir, level_num);

        let paths = match level_type_opt {
            Some(v) => LevelPaths::from_default_dir(base, v),
            None => LevelPaths::from_default_dir(base, &level_type),
        };

        let mut file = File::options().read(true).open(&level_path).unwrap();
        let landscape = Landscape::from_reader(&mut file);
        file.seek(std::io::SeekFrom::Start(0x8000)).unwrap();
        //read 0x4000
        file.seek(std::io::SeekFrom::Current(0x4000)).unwrap();
        //read 0x4000
        file.seek(std::io::SeekFrom::Current(0x4000)).unwrap();
        //read 0x4000 (land flags)
        file.seek(std::io::SeekFrom::Current(0x4000)).unwrap();
        let mut tribes = Vec::new();
        for _ in 0..4 {
            tribes.push(TribeConfigRaw::from_reader(&mut file).unwrap());
        }
        let sunlight = Sunlight::from_reader(&mut file);
        //read units (up to 5500 * 100)
        let units = UnitRaw::from_reader_vec(&mut file);
        //read 0x96
        file.seek(std::io::SeekFrom::Current(0x96)).unwrap();
        let params = GlobeTextureParams::from_level(&paths);
        LevelRes {
            paths,
            params,
            landscape,
            tribes,
            sunlight,
            units,
        }
    }
}

pub fn read_level(base: &Path, num: u8) -> (PathBuf, String) {
    let dat_path = LevelPaths::dat_path(base, num);
    let hdr_path = LevelPaths::hdr_path(base, num);
    let s = read_landscape_type(&hdr_path);
    (dat_path, s)
}

/******************************************************************************/

pub fn read_landscape_type(hdr_path: &Path) -> String {
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

pub fn read_bin(path: &Path) -> Vec<u8> {
    let mut f = OpenOptions::new().read(true).open(path).unwrap();
    let mut vec = Vec::new();
    f.read_to_end(&mut vec).unwrap();
    vec
}

#[allow(dead_code)]
fn read_bin16(path: &Path) -> Vec<u16> {
    let buf = read_bin(path);
    let mut vec = vec![0; buf.len() / 2];
    for (i, n) in (0..).zip(buf.chunks(2).take(vec.len())) {
        if n.len() == 2 {
            vec[i] = u16::from_le_bytes([n[0], n[1]]);
        }
    }
    vec
}

fn read_bin_i8(path: &Path) -> Vec<i8> {
    let buf = read_bin(path);
    let mut v = std::mem::ManuallyDrop::new(buf);
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();
    unsafe { Vec::from_raw_parts(p as *mut i8, len, cap) }
}

fn read_disp(path: &Path) -> Vec<i8> {
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

pub fn read_pal(paths: &LevelPaths) -> Vec<u8> {
    read_bin(&paths.palette)
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

    pub fn from_reader<R: Read>(reader: &mut R) -> Self {
        let mut s = Self::new();
        let mut buf = Vec::new();
        let _file_size = reader.read_to_end(&mut buf);
        for (i, n) in (0..).zip(buf.chunks(2).take(N*N)) {
            if n.len() == 2 {
                let val = u16::from_le_bytes([n[0], n[1]]);
                s.height[i%N][i/N] = val;
            }
        }
        s.flip();
        s
    }

    pub fn from_file(path: &Path) -> Self {
        let mut file = File::options().read(true).open(path).unwrap();
        Self::from_reader(&mut file)
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
