use std::vec::Vec;

use cgmath::Vector3;
use gl46::*;

use crate::model::{FromUsize, MeshModel};
use crate::envelop::{GlModel, GlModelBatch, EModel};

use crate::opengl::vertex::GlVertexAttr;
use crate::opengl::buffer::GlBufferStatic;

/******************************************************************************/

pub type DefaultModel = MeshModel<Vector3<f32>, u16>;

impl FromUsize for u16 {
    fn from_usize(v: usize) -> Self {
        v as u16
    }
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

/******************************************************************************/

impl GlModel for DefaultModel {
    type BatchType = DefaultBatch;

    fn vertex_attributes(&self) -> Vec<GlVertexAttr> {
        vec![GlVertexAttr::new(0, 3, GL_FLOAT, 0)]
    }

    fn vertex_size(&self) -> usize {
        std::mem::size_of::<f32>() * 3
    }
    
    fn vertex_num(&self) -> usize {
        self.vertices.len()
    }

    fn vertex_buffer_size(&self) -> usize {
        self.vertices.len() * self.vertex_size()
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
        !self.indices.is_empty()
    }

    fn add_to_buffer(&self, offset: usize, _total_vertices: usize, buffer: &mut GlBufferStatic) -> usize {
        if self.vertices.is_empty() {
            return 0;
        }
        let slice = self.vertices.as_slice();
        buffer.update(offset * self.vertex_size(), slice).unwrap();
        self.vertex_buffer_size()
    }

    fn add_to_buffer_elem(&self, offset: usize, buffer: &mut GlBufferStatic) -> usize {
        if self.indices.is_empty() {
            return 0;
        }
        let slice = self.indices.as_slice();
        buffer.update(offset, slice).unwrap();
        self.index_buffer_size()
    }
}

pub struct DefaultBatch;

impl GlModelBatch<EModel<DefaultModel>> for DefaultBatch {
    fn vertex_attributes(_models: &[EModel<DefaultModel>]) -> Vec<GlVertexAttr> {
        vec![GlVertexAttr::new(0, 3, GL_FLOAT, 0)]
    }

    fn vertex_buffer_size(models: &[EModel<DefaultModel>]) -> usize {
        models.iter().rfold(0, |x, e| x + e.model.vertex_buffer_size())
    }

    fn index_buffer_size(models: &[EModel<DefaultModel>]) -> usize {
        models.iter().rfold(0, |x, e| x + e.model.index_buffer_size())
    }
}
