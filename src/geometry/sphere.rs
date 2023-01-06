use cgmath::Vector3;

use crate::model::VertexModel;

pub fn gen_sphere<M: VertexModel<Vector3<f64>, u16>>(model: &mut M, ribs_h: u16, ribs_v: u16) {
    let k1 = f64::acos(-1.0) / (ribs_v as f64); // pi / ribs_v
    let k = 2.0 * f64::acos(-1.0) / (ribs_h as f64); // 2 * pi / ribs_h
    let mut last_z = f64::cos(0.0);
    let mut last_kk = 0.0;
    for j in 0..ribs_v {
        let mk = k1 * ((j+1) as f64);
        let z = f64::cos(mk);
        let z1 = f64::sin(mk);
        let kk = z1;
        let last_x = kk * f64::cos(0.0);
        let last_y = kk * f64::sin(0.0);
        let last_x1 = last_kk * f64::cos(0.0);
        let last_y1 = last_kk * f64::sin(0.0);

        let mut e1: u16 = model.push_vertex(Vector3::new(last_x1, last_y1, last_z));
        let mut e2: u16 = model.push_vertex(Vector3::new(last_x, last_y, z));

        for i in 0..ribs_h {
            let m = k * ((i+1) as f64);
            let x = kk * f64::cos(m);
            let y = kk * f64::sin(m);
            let x1 = last_kk * f64::cos(m);
            let y1 = last_kk * f64::sin(m);

            model.push_index(e1);
            model.push_index(e2);
            let e1_2 = model.push_vertex(Vector3::new(x1, y1, last_z));
            model.push_index(e1_2);
            model.push_index(e2);
            model.push_index(e1_2);
            let e1_1 = model.push_vertex(Vector3::new(x, y, z));
            model.push_index(e1_1);

            e1 = e1_2;
            e2 = e1_1;
        }
        last_z = z;
        last_kk = kk;
    }
}
