//! Coxeter diagrams for puzzle symmetry groups.

// #![warn(missing_docs)]

#[macro_use]
mod vector;
#[macro_use]
mod matrix;
mod coxeter;
mod group;
mod shape;
mod util;

pub use coxeter::*;
pub use group::*;
pub use matrix::*;
pub use shape::*;
pub use vector::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_facets() {
        let cubic_symmetry = CoxeterDiagram::with_edges(vec![4, 3]).group();

        let cube = Shape::new(&cubic_symmetry, &vec![Vector::unit(0)]);
        assert_eq!(cube.elements(2).len(), 6);

        let octahedron = Shape::new(&cubic_symmetry, &vec![vector![1.0, 1.0, 1.0]]);
        assert_eq!(octahedron.elements(2).len(), 8);
    }

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
        let group = CoxeterDiagram::with_edges(edges).group();
        assert_eq!(group.order(), expected);
    }
}
