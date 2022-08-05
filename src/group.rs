use itertools::Itertools;

use crate::matrix::*;

#[derive(Debug, Clone)]
pub struct Group {
    /// Number of dimensions that each elements has.
    ndim: u8,
    /// Number of initial elements that are generators (excluding the identity
    /// element).
    generator_count: u8,

    /// Matrix for each element.
    elem_matrices: Vec<Matrix<f32>>,
    /// Decomposition into generators for each element.
    elem_decompositions: Vec<Vec<GroupElement>>,
    /// For each generator, the result of composing each element with that
    /// generator.
    elem_successors: Vec<Vec<GroupElement>>,
    /// Inverse for each element.
    elem_inverses: Vec<GroupElement>,
}
impl Default for Group {
    fn default() -> Self {
        Self::new_trivial(0)
    }
}
impl Group {
    pub fn new_trivial(ndim: u8) -> Self {
        Self {
            ndim,
            generator_count: 0,
            elem_matrices: vec![Matrix::ident(ndim)],
            elem_decompositions: vec![vec![]],
            elem_successors: vec![],
            elem_inverses: vec![GroupElement(0)],
        }
    }

    pub fn from_generators(generators: &[Matrix<f32>]) -> Self {
        let ndim = generators.iter().map(|m| m.ndim()).max().unwrap_or(0);
        let mut ret = Self::new_trivial(ndim);
        ret.generator_count = generators.len() as _;
        ret.elem_successors = vec![vec![]; generators.len()];
        ret.elem_inverses = vec![GroupElement::IDENT; generators.len() + 1];

        // TODO: compute period of each generator and make sure it's smallish.

        // Find all group elements.
        let mut next_unprocessed = 0;
        while next_unprocessed < ret.order() {
            let e = GroupElement(next_unprocessed);

            for (i, generator_matrix) in generators.iter().enumerate() {
                let gen = GroupElement(i as u32 + 1);

                let m = ret.matrix(e) * generator_matrix;

                let successor_element = if m.approx_eq(&Matrix::EMPTY_IDENT) {
                    ret.elem_inverses[gen.idx()] = e;

                    // e * gen = I
                    GroupElement::IDENT
                } else if let Some((j, _)) = ret.elem_matrices[1..]
                    .iter()
                    .find_position(|old| old.approx_eq(&m))
                {
                    // e * gen = existing element
                    GroupElement(j as u32 + 1)
                } else {
                    ret.elem_matrices.push(m);

                    let decomposition = ret.decompose(e).iter().copied().chain([gen]).collect();
                    ret.elem_decompositions.push(decomposition);

                    // e * gen = new element
                    GroupElement(ret.elem_matrices.len() as u32 - 1)
                };

                ret.elem_successors[i].push(successor_element);
            }

            next_unprocessed += 1;
        }

        // TODO: error if any generator has identity as its inverse

        ret.elem_inverses
            .resize(ret.order() as _, GroupElement::IDENT);
        for elem in ret.elements().skip(ret.generator_count as usize + 1) {
            if ret.inverse(elem) == GroupElement::IDENT {
                let inv_elem = ret
                    .decompose(elem)
                    .iter()
                    .rev()
                    .fold(GroupElement::IDENT, |e, &gen| {
                        ret.compose(e, ret.inverse(gen))
                    });
                assert_ne!(inv_elem, GroupElement::IDENT, "{:?}", elem);

                ret.elem_inverses[elem.idx()] = inv_elem;
                ret.elem_inverses[inv_elem.idx()] = elem;
            }
        }

        ret
    }

    pub fn ndim(&self) -> u8 {
        self.ndim
    }
    pub fn matrix(&self, e: GroupElement) -> &Matrix<f32> {
        &self.elem_matrices[e.idx()]
    }
    pub fn decompose(&self, e: GroupElement) -> &[GroupElement] {
        &self.elem_decompositions[e.idx()]
    }
    pub fn compose(&self, e1: GroupElement, e2: GroupElement) -> GroupElement {
        self.decompose(e2)
            .iter()
            .fold(e1, |e, gen| self.elem_successors[gen.idx() - 1][e.idx()])
    }
    pub fn inverse(&self, e: GroupElement) -> GroupElement {
        self.elem_inverses[e.idx()]
    }

    pub fn order(&self) -> u32 {
        self.elem_matrices.len() as _
    }
    pub fn elements(&self) -> impl Iterator<Item = GroupElement> + ExactSizeIterator {
        (0..self.order()).map(GroupElement)
    }
    pub fn generators(&self) -> impl Iterator<Item = GroupElement> + ExactSizeIterator {
        (1..self.generator_count as u32 + 1).map(GroupElement)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GroupElement(u32);
impl GroupElement {
    pub const IDENT: Self = Self(0);

    pub fn idx(self) -> usize {
        self.0 as _
    }
}
