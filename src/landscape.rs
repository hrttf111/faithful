use std::iter::Zip;
use std::ops::RangeFrom;
use std::slice::Chunks;

use gl46::*;

use cgmath::{Vector4, Vector3, Vector2};

use crate::model::{Triangle, VertexModel, MeshModel};
use crate::default_model::DefaultBatch;

use crate::envelop::{GlModel, GlModelBatch, EModel};
use crate::opengl::vertex::GlVertexAttr;
use crate::opengl::buffer::GlBufferStatic;

pub type LandscapeModel = MeshModel<Vector2<u8>, u16>;

impl GlModel for LandscapeModel {
    type BatchType = DefaultBatch;

    fn vertex_attributes(&self) -> Vec<GlVertexAttr> {
        vec![GlVertexAttr::new(0, 2, GL_UNSIGNED_BYTE, 0)]
    }
    fn vertex_num(&self) -> usize {
        self.vertices.len()
    }

    fn vertex_buffer_size(&self) -> usize {
        self.vertices.len() * std::mem::size_of::<u8>() * 2
    }

    fn index_num(&self) -> usize {
        self.indices.len()
    }

    fn index_buffer_size(&self) -> usize {
        self.indices.len() * std::mem::size_of::<u16>()
    }

    fn index_gl_type(&self) -> GLenum {
        GL_UNSIGNED_SHORT
    }

    fn is_indexed(&self) -> bool {
        false
    }

    fn add_to_buffer(&self, offset: usize, buffer: &mut GlBufferStatic) -> usize {
        let slice = self.vertices.as_slice();
        buffer.update(offset, slice).unwrap();
        self.vertex_buffer_size()
    }

    fn add_to_buffer_elem(&self, _offset: usize, _buffer: &mut GlBufferStatic) -> usize {
        0
    }
}

impl GlModelBatch<EModel<LandscapeModel>> for DefaultBatch {
    fn vertex_attributes(_models: &[EModel<LandscapeModel>]) -> Vec<GlVertexAttr> {
        vec![GlVertexAttr::new(0, 2, GL_UNSIGNED_BYTE, 0)]
    }

    fn vertex_buffer_size(models: &[EModel<LandscapeModel>]) -> usize {
        models.iter().rfold(0, |x, e| x + e.model.vertex_buffer_size())
    }

    fn index_buffer_size(models: &[EModel<LandscapeModel>]) -> usize {
        models.iter().rfold(0, |x, e| x + e.model.index_buffer_size())
    }
}

pub struct LandscapeMesh<const N: usize> {
    vertices: Vec<Vector2<u8>>,
    step: f32,
    height_scale: f32,
    shift_x: usize,
    shift_y: usize,
    heights: [[u16; N]; N],
}

impl<const N: usize> LandscapeMesh<N> {
    fn gen_mesh() -> Vec<Vector2<u8>> {
        let vertices_num: usize = N * N * 6;
        let mut vertices = vec![Vector2{x: 0, y: 0}; vertices_num];
        for i in 0..(N-1) {
            for j in 0..(N-1) {
                let index = (i * N + j) * 6;
                let i_u8 = i as u8;
                let j_u8 = j as u8;
                vertices[index] = Vector2{x: i_u8, y: j_u8};
                vertices[index+1] = Vector2{x: i_u8, y: j_u8+1};
                vertices[index+2] = Vector2{x: i_u8+1, y: j_u8};
                vertices[index+3] = Vector2{x: i_u8+1, y: j_u8+1};
                vertices[index+4] = Vector2{x: i_u8, y: j_u8+1};
                vertices[index+5] = Vector2{x: i_u8+1, y: j_u8};
            }
        }
        vertices
    }

    fn shift_n(n: usize, max_n: usize, shift: i32) -> usize {
        let shift_n = (n as i32) + shift;
        if shift_n >= 0 {
            (shift_n as usize) % max_n
        } else {
            (((max_n as i32) + shift_n) as usize) % max_n
        }
    }

    pub fn new(step: f32, height_scale: f32) -> Self {
        let vertices = Self::gen_mesh();
        Self{vertices, step, height_scale, shift_x: 0, shift_y: 0, heights: [[0u16; N]; N]}
    }

    pub fn set_heights(&mut self, heights: &[[u16; N]; N]) {
        self.heights = *heights;
    }

    pub fn step(&self) -> f32 {
        self.step
    }

    pub fn height_scale(&self) -> f32 {
        self.height_scale
    }

    pub fn shift_x(&mut self, shift: i32) -> usize {
        self.shift_x = Self::shift_n(self.shift_x, N, shift);
        self.shift_x
    }

    pub fn shift_y(&mut self, shift: i32) -> usize {
        self.shift_y = Self::shift_n(self.shift_y, N, shift);
        self.shift_y
    }

    pub fn get_shift_vector(&self) -> Vector4<f32> {
        Vector4::new(self.shift_x as f32, self.shift_y as f32, 0.0, 0.0)
    }

    pub fn to_model(&self, m: &mut LandscapeModel) {
        for v2 in &self.vertices {
            m.push_vertex(*v2);
        }
    }

    pub fn iter(&self) -> LandscapeTriangleIterator<N> {
        let iter_internal = (0..).zip(self.vertices.chunks(3));
        LandscapeTriangleIterator{landscape: self, iter_internal}
    }

    fn make_vec3(&self, v: &Vector2<u8>) -> Vector3<f32> {
        let x = v.x as f32 * self.step;
        let y = v.y as f32 * self.step;
        let index_x = (v.x as usize + self.shift_x) % N;
        let index_y = (v.y as usize + self.shift_y) % N;
        let z = self.heights[index_y][index_x] as f32 * self.height_scale;
        Vector3{x, y, z}
    }
}

pub struct LandscapeTriangleIterator<'a, const N: usize> {
    landscape: &'a LandscapeMesh<N>,
    iter_internal: Zip<RangeFrom<usize>, Chunks<'a, Vector2<u8>>>,
}

impl<'a, const N: usize> Iterator for LandscapeTriangleIterator<'a, N> {
    type Item = (usize, Triangle<f32>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter_internal.next() {
            Some((n, c)) => {
                if c.len() != 3 {
                    return None;
                }
                let a = self.landscape.make_vec3(&c[0]);
                let b = self.landscape.make_vec3(&c[1]);
                let c = self.landscape.make_vec3(&c[2]);
                let t: Triangle<f32> = Triangle{a, b, c};
                Some((n, t))
            },
            None => None,
        }
    }
}

/*
struct LandscapeProp {
    pub offset_x: usize,
    pub offset_y: usize,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl LandscapeProp {
    pub fn index_x<const N: usize>(&self, index: usize) -> usize {
        let x = (index + self.offset_x) % N;
        if self.flip_x {
            return (N-1) - x;
        } else {
            return x;
        }
    }

    pub fn index_y<const N: usize>(&self, index: usize) -> usize {
        let y = (index + self.offset_y) % N;
        if self.flip_y {
            return (N-1) - y;
        } else {
            return y;
        }
    }
}

pub fn landscape_to_mesh<const N: usize, M: GModel<f32, u16>>(land: &Landscape<N>, m: &mut M, step: f32, h_scale: f32, prop: &LandscapeProp) {
    let mut lines = [[0u16; N]; 2];
    let &mut mut line_prev = &mut lines[1];
    let &mut mut line_curr = &mut lines[0];
    for i in 0..N {
        let ip = prop.index_x::<N>(i);
        let jp = prop.index_y::<N>(0);
        let ix = i as f32;
        let iy = 0.0;
        line_curr[i] = m.push_vertex_v(ix * step, iy * step, land.height[jp][ip] as f32 * h_scale);
    }
    line_prev = line_curr;

    for j in 1..N {
        for i in 0..N-1 {
            let ip = prop.index_x::<N>(i);
            let jp = prop.index_y::<N>(j);
            let ix = i as f32;
            let iy = j as f32;
            line_curr[i] = m.push_vertex_v(ix * step, iy * step, land.height[jp][ip] as f32 * h_scale);
            m.push_triangle_i(line_prev[i], line_curr[i], line_prev[i+1]);
            if i > 0 {
                m.push_triangle_i(line_prev[i], line_curr[i], line_curr[i-1]);
            }
        }
        let ip = prop.index_x::<N>(N-1);
        let jp = prop.index_y::<N>(j);
        let ix = (N-1) as f32;
        let iy = j as f32;
        line_curr[N-1] = m.push_vertex_v(ix * step, iy * step, land.height[jp][ip] as f32 * h_scale);
        m.push_triangle_i(line_prev[N-1], line_curr[N-1], line_curr[N-2]);
        let buf = line_prev;
        line_prev = line_curr;
        line_curr = buf;
    }
}
*/
