use std::rc::Rc;
use std::cell::RefCell;
use std::ffi::c_void;

use cgmath::{Vector3, Matrix4, Rad, Deg};

use gl46::*;

use crate::model::{IterTriangleModel, TriangleIteratorVector3};
use crate::opengl::gl::{GlCtx, gl_error_panic};
use crate::opengl::buffer::GlBufferStatic;
use crate::opengl::vertex::{GlVao, GlVertexAttr};
use crate::opengl::uniform::{GlUniform1, GlUniform};
use crate::intersect::intersect_iter;

pub trait GlModel {
    type BatchType;
    fn vertex_attributes(&self) -> Vec<GlVertexAttr>;
    fn vertex_size(&self) -> usize;

    fn vertex_num(&self) -> usize;
    fn vertex_buffer_size(&self) -> usize;

    fn index_num(&self) -> usize;
    fn index_buffer_size(&self) -> usize;
    fn index_gl_type(&self) -> GLenum;
    fn is_indexed(&self) -> bool;

    fn add_to_buffer(&self, offset: usize, total_vertices: usize, buffer: &mut GlBufferStatic) -> usize;
    fn add_to_buffer_elem(&self, offset: usize, buffer: &mut GlBufferStatic) -> usize;
}

pub trait GlModelBatch<M> {
    fn vertex_attributes(models: &[M]) -> Vec<GlVertexAttr>;
    fn vertex_buffer_size(models: &[M]) -> usize;
    fn index_buffer_size(models: &[M]) -> usize;
}

pub enum RenderType {
    Triangles,
    Lines,
}

pub struct EModel<M> {
    pub model: M,
    pub location: Vector3<f32>,
    pub angles: Vector3<f32>,
    pub scale: f32,
    render: RenderType,
}

impl<M> EModel<M> {
    pub fn new(model: M, render: RenderType) -> Self {
        let location = Vector3{x: 0.0, y: 0.0, z: 0.0};
        let angles = Vector3{x: 0.0, y: 0.0, z: 0.0};
        let scale = 1.0;
        Self{model, location, angles, scale, render}
    }

    pub fn transform(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.location)
                      * Matrix4::from_angle_x(Rad::from(Deg(self.angles.x)))
                      * Matrix4::from_angle_y(Rad::from(Deg(self.angles.y)))
                      * Matrix4::from_angle_z(Rad::from(Deg(self.angles.z)))
                      * Matrix4::from_scale(self.scale)
    }
}

pub type UniformMvp = Rc<RefCell<GlUniform1<Matrix4<f32>>>>;

pub type ModelInit<M> = (RenderType, M);

pub struct ModelEnvelop<M> {
    gl: GlCtx,
    models: Vec<EModel<M>>,
    vao: GlVao,
    array_buffer: GlBufferStatic,
    element_buffer: GlBufferStatic,
    mvp: UniformMvp,
}

fn make_array_buffer<M>(gl: &GlCtx, models: &[EModel<M>]) -> GlBufferStatic
    where M: GlModel, M::BatchType: GlModelBatch<EModel<M>> {
    let total_size = M::BatchType::vertex_buffer_size(models);
    GlBufferStatic::new(gl, total_size, GL_ARRAY_BUFFER).unwrap()
}

fn make_element_buffer<M>(gl: &GlCtx, models: &[EModel<M>]) -> GlBufferStatic
    where M: GlModel, M::BatchType: GlModelBatch<EModel<M>> {
    let index_buffer_size = M::BatchType::index_buffer_size(models);
    GlBufferStatic::new(gl, index_buffer_size, GL_ELEMENT_ARRAY_BUFFER).unwrap()
}

impl<M> ModelEnvelop<M> where M: GlModel, M::BatchType: GlModelBatch<EModel<M>> {
    pub fn new(gl: &GlCtx, mvp: &UniformMvp, models: Vec<ModelInit<M>>) -> Self {
        let mut models_e: Vec<EModel<M>> = models.into_iter()
                                             .map(|(render, model)| EModel::new(model, render))
                                             .collect();
        let mut vao = GlVao::new(gl).unwrap();
        vao.bind();
        let total_vertices = models_e.iter().rfold(0, |x, e| x + e.model.vertex_num());
        let mut array_buffer = make_array_buffer::<M>(gl, &models_e);
        let mut element_buffer = make_element_buffer::<M>(gl, &models_e);
        models_e.iter().fold(0, |offset, e| {
            e.model.add_to_buffer(offset, total_vertices, &mut array_buffer);
            offset + e.model.vertex_num()
        });
        models_e.iter_mut().fold(0, |offset, e| {
            offset + e.model.add_to_buffer_elem(offset, &mut element_buffer)
        });
        vao.set_attrs(M::BatchType::vertex_attributes(&models_e)).unwrap();
        vao.unbind();
        array_buffer.unbind();
        element_buffer.unbind();
        ModelEnvelop{gl: gl.clone(), models: models_e, vao, array_buffer, element_buffer, mvp: mvp.clone()}
    }

    pub fn draw(&self, index: i32) {
        let gl = &self.gl;
        self.vao.bind();
        let mut offset: usize = 0;
        let mut index_offset = 0;
        for e in &self.models {
            self.mvp.borrow_mut().set(e.transform());
            self.mvp.borrow().bind(gl, index);
            unsafe {
                let render_type = match e.render {
                    RenderType::Triangles => GL_TRIANGLES,
                    RenderType::Lines => GL_LINES,
                };
                if e.model.is_indexed() {
                    gl.DrawElementsBaseVertex(render_type
                                              , e.model.index_num() as i32
                                              , e.model.index_gl_type()
                                              , index_offset as *const c_void
                                              , offset as i32);
                } else {
                    gl.DrawArrays(render_type, offset as i32, e.model.vertex_num() as i32);
                }
                gl_error_panic(gl, "ModelEnvelop draw failed");
            }
            offset += e.model.vertex_num();
            index_offset += e.model.index_num();
        }
        self.vao.unbind();
    }

    pub fn get(&mut self, index: usize) -> Option<&mut EModel<M>> {
        if index >= self.models.len() {
            return None;
        }
        Some(&mut self.models[index])
    }

    pub fn update_model(&mut self, index: usize) {
        if index >= self.models.len() {
            return;
        }
        let total_vertices = self.models.iter().rfold(0, |x, e| x + e.model.vertex_num());
        let (offset, offset_indices) = self.models.iter().take(index).fold((0, 0), |(offset, offset_indices), m| {
            (offset + m.model.vertex_buffer_size(), offset_indices + m.model.index_buffer_size())
        });
        self.models[index].model.add_to_buffer(offset, total_vertices, &mut self.array_buffer);
        self.models[index].model.add_to_buffer_elem(offset_indices, &mut self.element_buffer);
    }
}

pub fn intersect_models<'a, M>(envelop: &'a ModelEnvelop<M>, vec_s: Vector3<f32>, vec_e: Vector3<f32>) -> Option<usize> 
    where M: IterTriangleModel<'a, Vector3<f32>> {
    let r = envelop.models.iter().rfold(None, |r, e| {
        let mvp = e.transform();
        let iter = TriangleIteratorVector3::from_model(&e.model);
        match (r, intersect_iter(iter, &mvp, vec_s, vec_e)) {
            (r1@Some((_, t1)), r2@Some((_, t2))) =>
                  if t1 > t2 {
                      r1
                  } else {
                      r2
                  }
            (None, r2) => r2,
            (r1, _) => r1,
        }
    });
    r.map(|(index, _)| index)
}
