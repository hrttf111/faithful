use std::iter::Iterator;
use std::vec::Vec;

use std::marker::PhantomData;

use std::iter::{Zip, StepBy};
use std::ops::{Range, RangeFrom};
use std::slice::Chunks;

use cgmath::Vector3;


/******************************************************************************/

pub trait FromUsize {
    fn from_usize(v: usize) -> Self;
    fn to_usize(&self) -> usize;
}

pub struct MeshModel<V, I> {
    pub vertices: Vec<V>,
    pub indices: Vec<I>,
}

impl<V, I> MeshModel<V, I> {
    pub fn new() -> Self {
        MeshModel{vertices: Vec::new(), indices: Vec::new()}
    }
}

impl<V, I> Default for MeshModel<V, I> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait VertexModel<V, I> {
    fn push_vertex(&mut self, v: V) -> I;
    fn push_index(&mut self, i: I);

    fn set_vertex(&mut self, i: I, v: V);
    fn get_vertex(&self, i: I) -> &V;
}

impl<V, I> VertexModel<V, I> for MeshModel<V, I> where I: FromUsize {
    fn push_vertex(&mut self, v: V) -> I {
        let i = self.vertices.len();
        self.vertices.push(v);
        I::from_usize(i)
    }

    fn push_index(&mut self, i: I) {
        self.indices.push(i);
    }

    fn set_vertex(&mut self, i: I, v: V) {
        let index = i.to_usize();
        self.vertices[index] = v;
    }

    fn get_vertex(&self, i: I) -> &V {
        let index = i.to_usize();
        &self.vertices[index]
    }
}

pub trait TriangleModel<V, I> where Self: VertexModel<V, I> {
    fn push_triangle(&mut self, a: V, b: V, c: V) {
        self.push_vertex(a);
        self.push_vertex(b);
        self.push_vertex(c);
    }

    fn push_triangle_indexes(&mut self, a: I, b: I, c: I) {
        self.push_index(a);
        self.push_index(b);
        self.push_index(c);
    }
}

impl<V, I> TriangleModel<V, I> for MeshModel<V, I> where I: FromUsize {
}

/******************************************************************************/

pub struct Triangle<V> {
    pub a: Vector3<V>,
    pub b: Vector3<V>,
    pub c: Vector3<V>,
}

pub struct TriangleRef<'a, V> {
    pub a: &'a V,
    pub b: &'a V,
    pub c: &'a V,
}

pub trait IterTriangleModel<'a, V: 'a> {
    type Iter: Iterator<Item = (usize, TriangleRef<'a, V>)>;

    fn iter(&'a self) -> Self::Iter;
}

impl<'a, V, I> IterTriangleModel<'a, V> for MeshModel<V, I>
    where V: 'a, I: 'a + FromUsize {
    type Iter = TriangleIterator<'a, V, I, Zip<RangeFrom<usize>, StepBy<Range<usize>>>, Zip<RangeFrom<usize>, Chunks<'a, V>>>;

    fn iter(&'a self) -> Self::Iter {
        let v = (0..).zip(self.vertices.chunks(3));
        let i = (0..).zip((0..self.indices.len()).step_by(3));
        let is_indices = !self.indices.is_empty();
        TriangleIterator{ mesh: self, is_indices, iter_internal_indices: i, iter_internal_vertices: v }
    }
}

pub struct TriangleIterator<'a, V, I, K1, K2>
    where K1: Iterator<Item = (usize, usize)>,
          K2: Iterator<Item = (usize, &'a [V])> {
    pub mesh: &'a MeshModel<V, I>,
    pub is_indices: bool,
    pub iter_internal_indices: K1,
    pub iter_internal_vertices: K2,
}

impl<'a, V, I, K1, K2> Iterator for TriangleIterator<'a, V, I, K1, K2>
    where K1 : Iterator<Item = (usize, usize)>,
          K2: Iterator<Item = (usize, &'a [V])>,
          I : FromUsize {
    type Item = (usize, TriangleRef<'a, V>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_indices {
            match self.iter_internal_indices.next() {
                Some((n, i)) => {
                    if self.mesh.indices.len() < (i + 2) {
                        return None;
                    }
                    let i1 = self.mesh.indices[i].to_usize();
                    let i2 = self.mesh.indices[i + 1].to_usize();
                    let i3 = self.mesh.indices[i + 2].to_usize();
                    let t: TriangleRef<V> = TriangleRef { a: &self.mesh.vertices[i1]
                                                        , b: &self.mesh.vertices[i2]
                                                        , c: &self.mesh.vertices[i3] };
                    Some((n, t))
                },
                None => None,
            }
        } else {
            match self.iter_internal_vertices.next() {
                Some((n, c)) => {
                    if c.len() != 3 {
                        return None;
                    }
                    let t: TriangleRef<V> = TriangleRef { a: &c[0]
                                                        , b: &c[1]
                                                        , c: &c[2] };
                    Some((n, t))
                },
                None => None,
            }
        }
    }
}

pub trait RefToTriangle<V> {
    fn to(&self) -> Vector3<V>;

    fn to_triangle(t: &TriangleRef<Self>) -> Triangle<V> where Self: Sized {
        let a = t.a.to();
        let b = t.b.to();
        let c = t.c.to();
        Triangle{a, b, c}
    }
}

impl RefToTriangle<f64> for Vector3<f64> {
    fn to(&self) -> Vector3<f64> {
        *self
    }
}


impl RefToTriangle<f32> for Vector3<f32> {
    fn to(&self) -> Vector3<f32> {
        *self
    }
}

pub struct TriangleIteratorVector3<'a, V, Iter, V2>
    where Iter: Iterator<Item = (usize, TriangleRef<'a, V>)>,
          V: RefToTriangle<V2> + 'a {
    pub iter_internal: Iter,
    phantom: PhantomData<V2>,
}

impl<'a, V, Iter, V2> TriangleIteratorVector3<'a, V, Iter, V2>
    where Iter: Iterator<Item = (usize, TriangleRef<'a, V>)>,
          V: RefToTriangle<V2> + 'a {

    pub fn new(iter: Iter) -> Self {
        Self{ iter_internal: iter, phantom: PhantomData }
    }


    pub fn from_model<M>(model: &'a M) -> Self where M: IterTriangleModel<'a, V, Iter = Iter> {
        let iter = model.iter();
        Self::new(iter)
    }
}

impl<'a, V, Iter, V2> Iterator for TriangleIteratorVector3<'a, V, Iter, V2>
    where Iter: Iterator<Item = (usize, TriangleRef<'a, V>)>,
          V: RefToTriangle<V2> + 'a {
    type Item = (usize, Triangle<V2>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_internal.next().map(|(n, t)| (n, V::to_triangle(&t)))
    }
}
