// TODO: credit Andrey for algorithm to generate shape

use std::ops::*;

use crate::group::*;
use crate::matrix::*;
use crate::vector::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Shape {
    base_faces: Vec<BaseFacetMesh>,
}
#[derive(Debug, Default, Clone, PartialEq)]
struct BaseFacetMesh {
    verts: Vec<Vector<f32>>,
    segs: Vec<[u32; 2]>,
    tris: Vec<[u32; 3]>,

    faces: Vec<Matrix<f32>>,
}
impl BaseFacetMesh {
    fn simplify(self) -> Self {
        Self {
            verts: (),
            segs: (),
            tris: (),
            faces: self.faces,
        }
    }
}

impl Shape {
    pub fn new(symmetry: &Group, base_facets: &[Vector<f32>]) -> Self {
        let cube =

        let ndim = symmetry.ndim();
        let mut ret = Self::default();
        ret.vectors = PerShapeElement::new(ndim);

        let rank = ndim - 1;

        struct ShapeElementSuccessors<'a> {
            group: &'a Group,
            /// For each generator: for each element: where that
            /// facet ends up when the generator is applied to it.
            successors: Vec<PerShapeElement<ShapeElement>>,
        }
        impl<'a> ShapeElementSuccessors<'a> {
            fn new(group: &'a Group) -> Self {
                Self {
                    group,
                    successors: vec![PerShapeElement::new(group.ndim()); group.generators().len()],
                }
            }
            fn apply_group_element(
                &self,
                shape_element: ShapeElement,
                group_element: GroupElement,
            ) -> ShapeElement {
                let mut e = shape_element;
                for gen in self.group.decompose(group_element) {
                    e = self.successors[gen.idx() - 1][e];
                }
                e
            }
        }
        let mut successors = ShapeElementSuccessors::new(symmetry);

        let mut shape_elem_gens = vec![];

        let mut next_unprocessed = 0;
        for facet_generator in facet_generators {
            if let Some(existing) = ret.element_at_vector(rank, facet_generator) {
                // TODO: report error
                panic!("duplicate facet generator {existing:?}");
            }

            let e = ret.vectors.push(rank, facet_generator.clone());
            shape_elem_gens.push(e);

            while next_unprocessed < ret.facets().len() {
                let e = ShapeElement {
                    rank,
                    id: next_unprocessed as _,
                };

                for (i, gen) in symmetry.generators().enumerate() {
                    let new_vector = symmetry.matrix(gen).transform(&ret.vectors[e]);
                    let successor_element =
                        if let Some(existing) = ret.element_at_vector(rank, &new_vector) {
                            existing
                        } else {
                            ret.vectors.push(rank, new_vector)
                        };
                    successors.successors[i].push(rank, successor_element);
                }

                next_unprocessed += 1;
            }
        }

        // Find all elements of this rank.

        // Start at ridges (ndim-2) and go down to vertices (0).
        for rank in (0..ndim - 1).rev() {
            let mut children_gens = vec![];
            for e in shape_elem_gens {}
        }

        // // Find all group elements.
        // let mut next_unprocessed = 0;
        // while next_unprocessed < ret.facets().len() {
        //     ret.vectors.push(rank, elem)

        //     let e = GroupElement(next_unprocessed);

        //     for (i, generator_matrix) in generators.iter().enumerate() {
        //         let gen = GroupElement(i as u32 + 1);

        //         let m = ret.matrix(e) * generator_matrix;

        //         let successor_element = if m.approx_eq(&Matrix::EMPTY_IDENT) {
        //             ret.elem_inverses[gen.idx()] = e;

        //             // e * gen = I
        //             GroupElement::IDENT
        //         } else if let Some((j, _)) = ret.elem_matrices[1..]
        //             .iter()
        //             .find_position(|old| old.approx_eq(&m))
        //         {
        //             // e * gen = existing element
        //             GroupElement(j as u32 + 1)
        //         } else {
        //             ret.elem_matrices.push(m);

        //             let decomposition = ret.decompose(e).iter().copied().chain([gen]).collect();
        //             ret.elem_decompositions.push(decomposition);

        //             // e * gen = new element
        //             GroupElement(ret.elem_matrices.len() as u32 - 1)
        //         };

        //         ret.elem_successors[i].push(successor_element);
        //     }

        //     next_unprocessed += 1;
        // }

        let mut element_successors = ShapeElementSuccessors::new(symmetry);

        let rank = ndim - 1;
        // For each facet type: for each facet: a group action.
        let mut facet_symmetries: Vec<Vec<GroupElement>> = vec![];
        for gen in facet_generators {
            let mut this_facet_symmetries = vec![];
            for group_elem in symmetry.elements() {
                let new = symmetry.matrix(group_elem).transform(&gen);
                if ret.elements(rank).all(|e| !new.approx_eq(ret.vector(e))) {
                    ret.vectors.push(rank, new);
                    this_facet_symmetries.push(group_elem);
                }
            }
            facet_symmetries.push(this_facet_symmetries);
        }

        ret
    }

    pub fn ndim(&self) -> u8 {
        self.element_counts.len() as _
    }
    pub fn elements(&self, rank: u8) -> impl Iterator<Item = ShapeElement> + ExactSizeIterator {
        (0..self.vectors.0[rank as usize].len() as _).map(move |id| ShapeElement { rank, id })
    }
    pub fn facets(&self) -> impl Iterator<Item = ShapeElement> + ExactSizeIterator {
        self.elements(self.ndim() - 1)
    }
    pub fn vector(&self, elem: ShapeElement) -> &Vector<f32> {
        &self.vectors[elem]
    }

    pub fn element_at_vector(&self, rank: u8, vector: impl VectorRef<f32>) -> Option<ShapeElement> {
        // TODO: be smart and only search elements of the same type
        self.elements(rank)
            .find(|&e| self.vectors[e].approx_eq(&vector))
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
