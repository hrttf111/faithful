use cgmath::{Vector2, Vector3};
use gl46::*;

use crate::model::MeshModel;
use crate::envelop::{GlModel, GlModelBatch, EModel};

use crate::opengl::vertex::GlVertexAttr;
use crate::opengl::buffer::GlBufferStatic;


/******************************************************************************/

pub struct TexVertex {
    pub coord: Vector3<f32>,
    pub uv: Vector2<f32>,
    pub tex_id: i16
}

pub type TexModel = MeshModel<TexVertex, u16>;


/******************************************************************************/

impl GlModel for TexModel {
    type BatchType = TexBatch;

    fn vertex_attributes(&self) -> Vec<GlVertexAttr> {
        let vertex_num = self.vertex_num();
        let uv_offset = vertex_num * (std::mem::size_of::<f32>() * 3);
        let tex_id_offset = uv_offset + vertex_num * (std::mem::size_of::<f32>() * 2);
        vec![GlVertexAttr::new(0, 3, GL_FLOAT, 0)
            , GlVertexAttr::new(1, 2, GL_FLOAT, uv_offset)
            , GlVertexAttr::new(2, 1, GL_SHORT, tex_id_offset)]
    }

    fn vertex_size(&self) -> usize {
        std::mem::size_of::<f32>() * 3 +
            std::mem::size_of::<f32>() * 2 +
            std::mem::size_of::<i16>()
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

    fn add_to_buffer(&self, offset: usize, total_vertices: usize, buffer: &mut GlBufferStatic) -> usize {
        if self.vertices.is_empty() {
            return 0;
        }
        let vec_vertex: Vec<Vector3<f32>> = self.vertices.iter().map(|v| v.coord).collect();
        let slice_vertex = vec_vertex.as_slice();
        buffer.update(offset * (std::mem::size_of::<f32>() * 3), slice_vertex).unwrap();

        let uv_offset = total_vertices * (std::mem::size_of::<f32>() * 3)
                        + offset * (std::mem::size_of::<f32>() * 2);
        let uv_vertex: Vec<Vector2<f32>> = self.vertices.iter().map(|v| v.uv).collect();
        let slice_uv = uv_vertex.as_slice();
        buffer.update(uv_offset, slice_uv).unwrap();

        let tex_id_offset = uv_offset + total_vertices * (std::mem::size_of::<f32>() * 2)
                          + offset * std::mem::size_of::<i16>();
        let tex_vertex: Vec<i16> = self.vertices.iter().map(|v| v.tex_id).collect();
        let slice_tex = tex_vertex.as_slice();
        buffer.update(tex_id_offset, slice_tex).unwrap();
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

pub struct TexBatch;

impl GlModelBatch<EModel<TexModel>> for TexBatch {
    fn vertex_attributes(models: &[EModel<TexModel>]) -> Vec<GlVertexAttr> {
        let vertex_num = models.iter().rfold(0, |x, e| x + e.model.vertex_num());
        let uv_offset = vertex_num * (std::mem::size_of::<f32>() * 3);
        let tex_id_offset = uv_offset + vertex_num * (std::mem::size_of::<f32>() * 2);
        vec![GlVertexAttr::new(0, 3, GL_FLOAT, 0)
            , GlVertexAttr::new(1, 2, GL_FLOAT, uv_offset)
            , GlVertexAttr::new(2, 1, GL_SHORT, tex_id_offset)]
    }

    fn vertex_buffer_size(models: &[EModel<TexModel>]) -> usize {
        models.iter().rfold(0, |x, e| x + e.model.vertex_buffer_size())
    }

    fn index_buffer_size(models: &[EModel<TexModel>]) -> usize {
        models.iter().rfold(0, |x, e| x + e.model.index_buffer_size())
    }
}
