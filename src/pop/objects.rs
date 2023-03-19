use std::path::Path;
use std::fs::File;
use std::io::Read;
use core::mem::size_of;
use core::slice::Iter;

use crate::pop::level::ObjectPaths;

/******************************************************************************/

pub trait BinReaderDeserializer {
    fn from_reader<R: Read>(reader: &mut R) -> Vec<Self> where Self: Sized;

    fn from_file(path: &Path) -> Vec<Self> where Self: Sized {
        let mut file = File::options().read(true).open(path).unwrap();
        Self::from_reader(&mut file)
    }
}

fn from_reader<T, const S: usize, R: Read>(reader: &mut R) -> Vec<T> where T: Copy {
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

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct ObjectRaw {
    flags: u16,
    facs_num: u16,
    pnts_num: u16,
    f1: u8,
    morph_index: u8,
    f2: u32,
    coord_scale: u32,
    facs_ptr: u32,
    facs_ptr_end: u32,
    pnts_ptr: u32,
    pnts_ptr_end: u32,
    f4: i16,
    f5: i16,
    f6: i16,
    f7: u16,
    f8: u16,
    f9: u16,
    shapes_index: u8,
    u1: u8,
    f10: u16,
    f11: u16,
    f12: u16,
    f13: u16,
}

impl BinReaderDeserializer for ObjectRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Vec<Self> {
        from_reader::<ObjectRaw, {size_of::<ObjectRaw>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Shape {
    x1: u8,
    y1: u8,
    x2: u8,
    y2: u8,
    unknown: [u8; 40],
    ptr: u32,
}

impl BinReaderDeserializer for Shape {
    fn from_reader<R: Read>(reader: &mut R) -> Vec<Self> {
        from_reader::<Self, {size_of::<Self>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct PointRaw {
    x: i16,
    y: i16,
    z: i16,
}

impl BinReaderDeserializer for PointRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Vec<Self> {
        from_reader::<Self, {size_of::<Self>()}, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct FaceRaw {
    f0: u16,
    tex_index: i16,
    flags1: i16,
    num_points: u8,
    f11: u8,
    point_1_u: u32,
    point_1_v: u32,
    point_2_u: u32,
    point_2_v: u32,
    point_3_u: u32,
    point_3_v: u32,
    point_4_u: u32,
    point_4_v: u32,
    point_1: u16,
    point_2: u16,
    point_3: u16,
    point_4: u16,
    f6: u16,
    ff1: u16,
    ff2: u16,
    ff3: u16,
    ff4: u16,
    f8: u8,
    flags2: u8,
}

impl BinReaderDeserializer for FaceRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Vec<Self> {
        from_reader::<Self, {size_of::<Self>()}, R>(reader)
    }
}

/******************************************************************************/

const XYZ_SCALE: f32 = 1.0 / 300.0;
const UV_SCALE: f32 = 4.768372e-7;

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
}

impl Vertex {
    pub fn new() -> Self {
        Vertex{x: 0.0, y: 0.0, z: 0.0, u: 0.0, v: 0.0}
    }

    pub fn from_point(&mut self, point: &PointRaw, u: u32, v: u32) {
        self.x = point.x as f32 * XYZ_SCALE;
        self.y = point.y as f32 * XYZ_SCALE;
        self.z = point.z as f32 * XYZ_SCALE;
        self.u = u as f32 * UV_SCALE;
        self.v = v as f32 * UV_SCALE;
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Face {
    pub texture_index: i16,
    pub vertex_num: usize,
    pub vertex: [Vertex; 4],
}

impl Face {
    pub fn new(texture_index: i16, vertex_num: usize) -> Self {
        Face{texture_index, vertex_num, vertex: [Vertex::default(); 4]}
    }
}

/******************************************************************************/

#[derive(Debug)]
pub struct Object3D {
    object: ObjectRaw,
    faces: Vec<FaceRaw>,
    points: Vec<PointRaw>,
}

impl Object3D {
    pub fn create(object: &ObjectRaw, faces: &[FaceRaw], points: &[PointRaw]) -> Self {
        let mut object_3d = Object3D{object: *object, faces: Vec::new(), points: Vec::new()};
        for i in object.pnts_ptr..object.pnts_ptr_end {
            object_3d.points.push(points[i as usize-1]);
        }
        for i in object.facs_ptr..object.facs_ptr_end {
            object_3d.faces.push(faces[i as usize-1]);
        }
        object_3d
    }

    pub fn create_objects(objects: &[ObjectRaw], faces: &[FaceRaw], points: &[PointRaw]) -> Vec<Self> {
        let mut objects_3d = Vec::new();
        for object in objects {
            if object.facs_num > 0 {
                objects_3d.push(Self::create(object, faces, points));
            }
        }
        objects_3d
    }

    pub fn from_file(base: &Path, bank_num: &str) -> Vec<Self> {
        let paths = ObjectPaths::from_base(base, bank_num);
        let objects = ObjectRaw::from_file(&paths.objs0_dat);
        let points = PointRaw::from_file(&paths.pnts0);
        let faces = FaceRaw::from_file(&paths.facs0);
        Self::create_objects(&objects, &faces, &points)
    }

    pub fn iter_face(&self) -> FaceIter<Iter<FaceRaw>> {
        FaceIter{iter: self.faces.iter(), points: &self.points}
    }

    pub fn coord_scale(&self) -> f32 {
        self.object.coord_scale as f32
    }
}

/******************************************************************************/

pub struct FaceIter<'a, I> where I: Iterator<Item = &'a FaceRaw> {
    iter: I,
    points: &'a [PointRaw],
}

impl<'a, I> Iterator for FaceIter<'a, I> where I: Iterator<Item = &'a FaceRaw> {
    type Item = Face;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(f) => {
                let num_points = std::cmp::min(f.num_points as usize, 4);
                let mut face_3d = Face::new(f.tex_index, num_points);
                face_3d.vertex[0].from_point(&self.points[f.point_1 as usize], f.point_1_u, f.point_1_v);
                face_3d.vertex[1].from_point(&self.points[f.point_2 as usize], f.point_2_u, f.point_2_v);
                face_3d.vertex[2].from_point(&self.points[f.point_3 as usize], f.point_3_u, f.point_3_v);
                if num_points == 4 {
                    face_3d.vertex[3].from_point(&self.points[f.point_4 as usize], f.point_4_u, f.point_4_v);
                }
                Some(face_3d)
            },
            None => None,
        }
    }
}
