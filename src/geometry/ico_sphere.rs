use cgmath::InnerSpace;
use cgmath::Vector3;

use crate::model::TriangleModel;
use crate::geometry::ico;

fn gen_subdivision<M: TriangleModel<Vector3<f32>, u16>>(model: &mut M, a: u16, b: u16, c: u16, n: u16) {
    let a_v = *model.get_vertex(a);
    let b_v = *model.get_vertex(b);
    let c_v = *model.get_vertex(c);
    let a1 = model.push_vertex((a_v + b_v).normalize());
    let b1 = model.push_vertex((b_v + c_v).normalize());
    let c1 = model.push_vertex((c_v + a_v).normalize());
    if n <= 1 {
        model.push_triangle_indexes(a, a1, c1);
        model.push_triangle_indexes(b, a1, b1);
        model.push_triangle_indexes(c, b1, c1);
        model.push_triangle_indexes(a1, b1, c1);
    } else {
        gen_subdivision(model, a, a1, c1, n-1);
        gen_subdivision(model, b, a1, b1, n-1);
        gen_subdivision(model, c, b1, c1, n-1);
        gen_subdivision(model, a1, b1, c1, n-1);
    }
}

#[allow(clippy::needless_range_loop)]
pub fn gen_ico_sphere<M: TriangleModel<Vector3<f32>, u16>>(model: &mut M, n: u16) {
    let ico_verts = ico::gen_icosahedron();
    for v in ico_verts {
        model.push_vertex(v.normalize());
    }

    for i in 0..3 {
        let current_ico = i * 4;
        let next_ico = ((i + 1) % 3) * 4;
        gen_subdivision(model, current_ico, current_ico+1, next_ico+3, n);
        gen_subdivision(model, current_ico, current_ico+1, next_ico+1, n);
        gen_subdivision(model, current_ico+2, current_ico+3, next_ico+2, n);
        gen_subdivision(model, current_ico+2, current_ico+3, next_ico, n);
    }

    for i in 0..4 {
        let current_ico: u16 = i as u16;
        let vec = &ico_verts[i];
        let next_ico = {
            4 + if vec.x > 0.0 { 0 } else { 1 }
        };
        let next_ico_1 = {
            8 + if vec.z > 0.0 { 2 } else { 0 }
        };
        gen_subdivision(model, current_ico, next_ico, next_ico_1+1, n);
        gen_subdivision(model, current_ico, next_ico+2, next_ico_1, n);
    }
}
