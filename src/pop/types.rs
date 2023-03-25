use std::path::Path;
use std::fs::File;
use std::io::Read;

/******************************************************************************/

pub trait BinDeserializer {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> where Self: Sized;

    fn from_reader_vec<R: Read>(reader: &mut R) -> Vec<Self> where Self: Sized {
        let mut res = Vec::new();
        while let Some(obj) = Self::from_reader(reader) {
            res.push(obj);
        }
        res
    }

    fn from_file_vec(path: &Path) -> Vec<Self> where Self: Sized {
        let mut file = File::options().read(true).open(path).unwrap();
        Self::from_reader_vec(&mut file)
    }

    fn from_file(path: &Path) -> Option<Self> where Self: Sized {
        let mut file = File::options().read(true).open(path).unwrap();
        Self::from_reader(&mut file)
    }
}

pub fn from_reader<T, const S: usize, R: Read>(reader: &mut R) -> Option<T> where T: Copy {
    let mut data = [0u8; S];
    if let Ok(()) = reader.read_exact(&mut data) {
        return Some(unsafe {
            *(data.as_ptr() as *const T)
        });
    }
    None
}

pub fn from_reader_vec<T, const S: usize, R: Read>(reader: &mut R) -> Vec<T> where T: Copy {
    let mut items = Vec::new();
    let mut data = [0u8; S];
    while let Ok(()) = reader.read_exact(&mut data) {
        items.push(unsafe {
            *(data.as_ptr() as *const T)
        });
    }
    items
}

/******************************************************************************/

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

/******************************************************************************/
