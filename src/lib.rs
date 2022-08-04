//! Coxeter diagrams for puzzle symmetry groups.

// #![allow(unused_imports, dead_code)]
// #![warn(missing_docs)]

#[macro_use]
mod vector;
#[macro_use]
mod matrix;
mod coxeter;
mod group;
mod util;

pub use coxeter::*;
pub use group::*;
pub use matrix::*;
pub use vector::*;

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

    fn assert_group_order(edges: Vec<usize>, expected: u32) {
        let cd = CoxeterDiagram::with_edges(edges);
        let generators: Vec<_> = cd.mirrors().into_iter().map(Matrix::from).collect();
        // println!("{:#?}", Group::from_generators(&generators));
        assert_eq!(Group::from_generators(&generators).order(), expected);
    }
}
