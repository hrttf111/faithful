use cgmath::{Vector3, Matrix4, SquareMatrix};
use num_traits::Float;
use num_traits::cast::NumCast;
use num_traits::identities::{zero, one};

use crate::model::{Triangle, IterTriangleModel, TriangleIteratorVector3, RefToTriangle};

pub fn intersect_triangle<V>(vs: &Vector3<V>, ve: &Vector3<V>, t1: &Triangle<V>) -> (bool, V) where V: Float {
    let t = t1;
    let a = t.a.x - t.b.x;
    let b = t.a.y - t.b.y;
    let c = t.a.z - t.b.z;
    let d = t.a.x - t.c.x;
    let e = t.a.y - t.c.y;
    let f = t.a.z - t.c.z;
    let g = ve.x;
    let h = ve.y;
    let i = ve.z;
    let j = t.a.x - vs.x;
    let k = t.a.y - vs.y;
    let l = t.a.z - vs.z;

    let m = a*(e*i - h*f) + b*(g*f - d*i) + c*(d*h - e*g);
    let t1 = -(f*(a*k - j*b) + e*(j*c - a*l) + d*(b*l - k*c)) / m;
    let bt = (j*(e*i - h*f) + k*(g*f - d*i) + l*(d*h - e*g)) / m;
    let gt = (i*(a*k - j*b) + h*(j*c - a*l) + g*(b*l - k*c)) / m;
    if gt > zero::<V>() && bt > zero::<V>() && gt < one::<V>() && bt < (one::<V>() - gt) {
        return (true, t1);
    }
    (false, zero::<V>())
}

fn mul<V>(vec: &Vector3<V>, m: &Matrix4<f32>) -> Vector3<V> where V: Float {
    let vec_1: Vector3<f32> = Vector3{x: <f32 as NumCast>::from(vec.x).unwrap()
                                     , y: <f32 as NumCast>::from(vec.y).unwrap()
                                     , z: <f32 as NumCast>::from(vec.z).unwrap()};
    let vec_2 = (m * vec_1.extend(one::<f32>())).truncate();
    Vector3{x: <V as NumCast>::from(vec_2.x).unwrap(), y: <V as NumCast>::from(vec_2.y).unwrap(), z: <V as NumCast>::from(vec_2.z).unwrap()}
}

pub fn intersect<'a, M, V>(model: &'a M, mvp: &Matrix4<f32>, vec_s: Vector3<V>, vec_e: Vector3<V>) -> Option<(usize, V)>
    where M: IterTriangleModel<'a, Vector3<V>>,
          V: Float + 'a,
          Vector3<V>: RefToTriangle<V> {
    let iter = TriangleIteratorVector3::from_model(model);
    intersect_iter(iter, mvp, vec_s, vec_e)
}

pub fn intersect_iter<I, V>(iter: I, mvp: &Matrix4<f32>, vec_s: Vector3<V>, vec_e: Vector3<V>) -> Option<(usize, V)>
    where I: Iterator<Item = (usize, Triangle<V>)>,
          V: Float {
    let mvp_t = mvp.invert().unwrap();
    let vec_s1 = mul::<V>(&vec_s, &mvp_t);
    let vec_e1 = mul::<V>(&vec_e, &mvp_t);
    let mut res: Option<(usize, V)> = None;
    for (n, t) in iter {
        let (b, r) = intersect_triangle(&vec_s1, &vec_e1, &t);
        if b {
            res = match res {
                ro@Some((_, r_nearest)) => {
                    if r < r_nearest {
                        Some((n, r))
                    } else {
                        ro
                    }
                },
                None => Some((n, r)),
            }
        }
    }
    res
}
