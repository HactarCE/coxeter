use std::ops::*;

use crate::group::*;
use crate::vector::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Shape {
    /// Number of elements of each rank.
    element_counts: Vec<usize>,

    /// Vector for each element of each rank.
    vectors: PerShapeElement<Vector<f32>>,
}
impl Shape {
    pub fn new(symmetry: &Group, facet_generators: &[Vector<f32>]) -> Self {
        let ndim = symmetry.ndim();
        let mut ret = Self::default();
        ret.vectors = PerShapeElement::new(ndim);

        let rank = ndim - 1;
        for gen in facet_generators {
            for new in symmetry
                .elements()
                .map(|e| symmetry.matrix(e).transform(gen))
            {
                if ret.elements(rank).all(|e| !new.approx_eq(ret.vector(e))) {
                    ret.vectors.push(rank, new);
                }
            }
        }

        ret
    }

    pub fn ndim(&self) -> u8 {
        self.element_counts.len() as _
    }
    pub fn elements(&self, rank: u8) -> impl Iterator<Item = ShapeElement> + ExactSizeIterator {
        (0..self.vectors.0[rank as usize].len() as _).map(move |id| ShapeElement { rank, id })
    }
    pub fn vector(&self, elem: ShapeElement) -> &Vector<f32> {
        &self.vectors[elem]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ShapeElement {
    rank: u8,
    id: u32,
}
impl ShapeElement {
    pub fn rank(self) -> u8 {
        self.rank
    }
    pub fn id(self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PerShapeElement<T>(Vec<Vec<T>>);
impl<T> Default for PerShapeElement<T> {
    fn default() -> Self {
        Self(vec![])
    }
}
impl<T: Clone> PerShapeElement<T> {
    fn new(ndim: u8) -> Self {
        Self(vec![vec![]; ndim as _])
    }

    fn push(&mut self, rank: u8, elem: T) -> ShapeElement {
        let id = self.0[rank as usize].len() as _;
        self.0[rank as usize].push(elem);
        ShapeElement { rank, id }
    }
}
impl<T> Index<ShapeElement> for PerShapeElement<T> {
    type Output = T;

    fn index(&self, e: ShapeElement) -> &Self::Output {
        &self.0[e.rank as usize][e.id as usize]
    }
}
impl<T> IndexMut<ShapeElement> for PerShapeElement<T> {
    fn index_mut(&mut self, e: ShapeElement) -> &mut Self::Output {
        &mut self.0[e.rank as usize][e.id as usize]
    }
}
