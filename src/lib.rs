//! Coxeter diagrams for puzzle symmetry groups.

#![allow(unused_imports, dead_code)]
// #![warn(missing_docs)]

use itertools::Itertools;
use num_traits::{Num, Signed};
use std::fmt;
use std::ops::*;

#[macro_use]
mod vector;
#[macro_use]
mod matrix;

use matrix::Matrix;
use vector::Vector;

/// Linear Coxeter diagram with unlabeled vertices.
pub struct CoxeterDiagram {
    edges: Vec<usize>,
}
impl CoxeterDiagram {
    /// Number of dimensions described by the Coxeter diagram's group.
    pub fn ndim(&self) -> u8 {
        self.edges.len() as u8 + 1
    }
}

pub struct Mirrors {
    generators: Vec<MirrorGenerator>,
}
pub struct MirrorGenerator {
    mirrors: Vec<Mirror>,
}
pub struct Mirror(Vector<f32>);
