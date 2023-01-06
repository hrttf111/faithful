use num_traits::{sign::Unsigned, float::Float};
use cgmath::Vector3;

use crate::model::VertexModel;

pub fn gen_circle<M: VertexModel<Vector3<f64>, u16>>(model: &mut M, ribs: isize) {
    let k: f64 = 2.0 * f64::acos(-1.0) / (ribs as f64);
    let e0: u16 = model.push_vertex(Vector3::new(0.0, 0.0, 0.0));
    let mut e1: u16 = model.push_vertex(Vector3::new(f64::cos(0.0), f64::sin(0.0), 0.0));
    for i in 0..=ribs {
        let p = i * 9;
        let m: f64 = k * (p as f64 + 1.0);
        model.push_index(e0);
        model.push_index(e1);
        let e2: u16 = model.push_vertex(Vector3::new(f64::cos(m), f64::sin(m), 0.0));
        model.push_index(e2);
        e1 = e2;
    }
}

pub fn circle_t<M, S, I>(model: &mut M, ribs: usize)
    where M: VertexModel<Vector3<S>, I>,
          S: Float + Copy,
          I: Unsigned + Copy,
          f64: Into<S>,
          u16: Into<I>
    {
    let k: f64 = 2.0 * f64::acos(-1.0) / (ribs as f64);
    let e0 = model.push_vertex(Vector3::new(0.0.into(), 0.0.into(), 0.0.into()));
    let mut e1 = model.push_vertex(Vector3::new(f64::cos(0.0).into(), f64::sin(0.0).into(), 0.0.into()));
    for i in 0..=ribs {
        let p = i * 9;
        let m: f64 = k * (p as f64 + 1.0);
        model.push_index(e0);
        model.push_index(e1);
        let e2 = model.push_vertex(Vector3::new(f64::cos(m).into(), f64::sin(m).into(), 0.0.into()));
        model.push_index(e2);
        e1 = e2;
    }
}

/*
pub fn circle_t1<M, S, I>(model: &mut M, ribs: usize)
    where M: GModel<S, I>,
          S: Float + Copy + FromPrimitive,
          I: Unsigned + Copy + FromPrimitive
    {
    let k: f64 = 2.0 * f64::acos(-1.0) / (ribs as f64);
    let e0 = model.push_vertex_v(0.0.into(), 0.0.into(), 0.0.into());
    let mut e1 = model.push_vertex_v(f64::cos(0.0).into(), f64::sin(0.0).into(), 0.0.into());
    for i in 0..=ribs {
        let p = i * 9;
        let m: f64 = k * (p as f64 + 1.0);
        model.push_index(e0);
        model.push_index(e1);
        let e2 = model.push_vertex_v(f64::cos(m).into(), f64::sin(m).into(), 0.0.into());
        model.push_index(e2);
        e1 = e2;
    }
}
*/
