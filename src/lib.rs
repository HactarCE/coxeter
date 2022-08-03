//! Coxeter diagrams for puzzle symmetry groups.

#![allow(unused_imports, dead_code)]
// #![warn(missing_docs)]

use itertools::Itertools;
use num_traits::{Num, Signed};
use std::collections::VecDeque;
use std::fmt;
use std::ops::*;

#[macro_use]
mod vector;
#[macro_use]
mod matrix;
mod util;

use matrix::*;
use vector::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coxeter_generators() {
        // Tetrahedron
        assert_group_order(vec![3, 3], 24);
        // Cube
        assert_group_order(vec![4, 3], 48);
        // Icosahedron
        assert_group_order(vec![5, 3], 120);

        // Hundredagonal duoprism
        assert_group_order(vec![100, 2, 4], 1600);

        // // 120-cell
        // assert_group_order(vec![5, 3, 3], 14400);

        // // 6-simplex
        // assert_group_order(vec![3; 5], 5040);
    }

    fn assert_group_order(edges: Vec<usize>, expected: usize) {
        let cd = CoxeterDiagram { edges };
        let generators: Vec<_> = cd.mirrors().into_iter().map(Matrix::from).collect();
        assert_eq!(generate_group(generators).len(), expected);
    }
}

pub fn generate_group(generators: Vec<Matrix<f32>>) -> Vec<Matrix<f32>> {
    let ndim = generators.iter().map(|m| m.ndim()).max().unwrap_or(1);
    let mut ret: Vec<Matrix<f32>> = vec![];
    let mut queue: VecDeque<_> = vec![Matrix::ident(ndim)].into();
    while let Some(next) = queue.pop_front() {
        if ret.iter().all(|old| !old.approx_eq(&next)) {
            for gen in &generators {
                queue.push_back(&next * gen);
            }
            ret.push(next);
        }
    }
    ret
}

/// Linear Coxeter diagram with unlabeled vertices.
pub struct CoxeterDiagram {
    edges: Vec<usize>,
}
impl CoxeterDiagram {
    /// Number of dimensions described by the Coxeter diagram's group.
    pub fn ndim(&self) -> u8 {
        self.edges.len() as u8 + 1
    }

    pub fn mirrors(&self) -> Vec<Mirror> {
        let mut ret = vec![];
        let mut last = Vector::unit(0);
        for (i, &edge) in self.edges.iter().enumerate() {
            ret.push(Mirror(last.clone()));
            let q = last[i as u8];
            let y = (std::f32::consts::PI / edge as f32).cos() / q;
            let z = (1.0 - y * y).sqrt();
            last = Vector::EMPTY;
            last[i as u8] = y;
            last[i as u8 + 1] = z;
        }
        ret.push(Mirror(last));
        ret
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirrorGenerator {
    mirrors: Vec<Mirror>,
}
impl From<MirrorGenerator> for Matrix<f32> {
    fn from(gen: MirrorGenerator) -> Self {
        gen.mirrors
            .into_iter()
            .map(Matrix::from)
            .reduce(|a, b| &a * &b)
            .expect("empty mirror generator not allowed")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mirror(Vector<f32>);
impl From<Mirror> for Matrix<f32> {
    fn from(mirror: Mirror) -> Self {
        let ndim = mirror.0.ndim();
        let mut ret = Matrix::ident(ndim);
        for x in 0..ndim {
            for y in 0..ndim {
                *ret.get_mut(x, y) = ret.get(x, y) - 2.0 * mirror.0[x] * mirror.0[y];
            }
        }
        ret
    }
}
