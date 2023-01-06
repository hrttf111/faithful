use cgmath::Vector3;

use crate::model::TriangleModel;

pub fn gen_icosahedron() -> [Vector3<f32>; 12] {
    let phi = -(1.0 - f32::sqrt(5.0)) / 2.0;
    [
        //X
        Vector3 { x: -1.0, y: 0.0, z: phi } 
        , Vector3 { x: -1.0, y: 0.0, z: -phi }
        , Vector3 { x: 1.0, y: 0.0, z: phi }
        , Vector3 { x: 1.0, y: 0.0, z: -phi }
        //Y
        , Vector3 { x: phi, y: -1.0, z: 0.0 }
        , Vector3 { x: -phi, y: -1.0, z: 0.0 }
        , Vector3 { x: phi, y: 1.0, z: 0.0 }
        , Vector3 { x: -phi, y: 1.0, z: 0.0 }
        //Z
        , Vector3 { x: 0.0, y: phi, z: -1.0 }
        , Vector3 { x: 0.0, y: -phi, z: -1.0 }
        , Vector3 { x: 0.0, y: phi, z: 1.0 }
        , Vector3 { x: 0.0, y: -phi, z: 1.0 }
    ]
}


pub fn gen_ico_skeleton<M: TriangleModel<Vector3<f32>, u16>>(model: &mut M) {
    let ico_verts = gen_icosahedron();
    for vector in ico_verts {
        model.push_vertex(vector);
    }
    for i in 0..3 {
        let s = i * 4;
        model.push_triangle_indexes(s, s+1, s+2);
        model.push_triangle_indexes(s+2, s+3, s+1);
    }
}

#[allow(clippy::needless_range_loop)]
pub fn gen_ico<M: TriangleModel<Vector3<f32>, u16>>(model: &mut M) {
    let ico_verts = gen_icosahedron();
    for vector in ico_verts {
        model.push_vertex(vector);
    }
    for i in 0..3 {
        let current_ico = i * 4;
        let next_ico = ((i + 1) % 3) * 4;
        model.push_triangle_indexes(current_ico, current_ico+1, next_ico+3);
        model.push_triangle_indexes(current_ico, current_ico+1, next_ico+1);
        model.push_triangle_indexes(current_ico+2, current_ico+3, next_ico+2);
        model.push_triangle_indexes(current_ico+2, current_ico+3, next_ico);
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
        model.push_triangle_indexes(current_ico, next_ico, next_ico_1+1);
        model.push_triangle_indexes(current_ico, next_ico+2, next_ico_1);
    }
}
