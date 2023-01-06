use num_traits::{sign::Unsigned, float::Float};
use num_traits::identities::{zero, one};
use cgmath::Vector3;

use crate::model::TriangleModel;

pub fn cube<M, V, I>(model: &mut M)
    where M: TriangleModel<Vector3<V>, I>,
          V: Float + Copy,
          I: Unsigned + Copy
    {
    let i0 = model.push_vertex(Vector3::new(zero::<V>(), zero::<V>(), zero::<V>()));
    let i1 = model.push_vertex(Vector3::new(one::<V>(), zero::<V>(), zero::<V>()));
    let i2 = model.push_vertex(Vector3::new(zero::<V>(), one::<V>(), zero::<V>()));
    let i3 = model.push_vertex(Vector3::new(one::<V>(), one::<V>(), zero::<V>()));
    let i4 = model.push_vertex(Vector3::new(zero::<V>(), zero::<V>(), one::<V>()));
    let i5 = model.push_vertex(Vector3::new(one::<V>(), zero::<V>(), one::<V>()));
    let i6 = model.push_vertex(Vector3::new(zero::<V>(), one::<V>(), one::<V>()));
    let i7 = model.push_vertex(Vector3::new(one::<V>(), one::<V>(), one::<V>()));
    model.push_triangle_indexes(i0, i1, i2);
    model.push_triangle_indexes(i1, i2, i3);
    model.push_triangle_indexes(i0, i4, i2);
    model.push_triangle_indexes(i4, i2, i6);
    model.push_triangle_indexes(i0, i1, i4);
    model.push_triangle_indexes(i1, i4, i5);
    model.push_triangle_indexes(i2, i3, i6);
    model.push_triangle_indexes(i3, i6, i7);
    model.push_triangle_indexes(i4, i6, i5);
    model.push_triangle_indexes(i6, i5, i7);
    model.push_triangle_indexes(i1, i5, i3);
    model.push_triangle_indexes(i5, i3, i7);
}
