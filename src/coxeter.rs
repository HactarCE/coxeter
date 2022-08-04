use crate::{group::*, matrix::*, vector::*};

/// Linear Coxeter diagram with unlabeled vertices.
pub struct CoxeterDiagram {
    edges: Vec<usize>,
}
impl CoxeterDiagram {
    pub fn with_edges(edges: Vec<usize>) -> Self {
        Self { edges }
    }

    /// Number of dimensions described by the Coxeter diagram's group.
    pub fn ndim(&self) -> u8 {
        self.edges.len() as u8 + 1
    }

    pub fn mirrors(&self) -> Vec<Mirror> {
        let mut ret = vec![];
        let mut last = Vector::unit(0);
        for (i, &edge) in self.edges.iter().enumerate() {
            ret.push(Mirror(last.clone()));
            // The final mirror vectors will look like this, with each row as a
            // vector:
            //
            // ```
            // [ ? 0 0 0 0 ]
            // [ ? ? 0 0 0 ]
            // [ 0 ? ? 0 0 ]
            // [ 0 0 ? ? 0 ]
            // [ 0 0 0 ? ? ]
            // ```
            //
            // Each mirror vector is perpendicular to all the others except its
            // neighbors.
            //
            // So to compute each next mirror vector, we only need to consider
            // the previous one. Consider the third mirror vector:
            //
            // ```
            // [ 0 ? ? 0 0 ]
            // ```
            //
            // Only two axes are nonzero, and their values could be anything.
            // The first nonzero axis is irrelevant, because that axis will be
            // zero in the next vector. Let `q` be the value of the second
            // nonzero axis.
            let q = last[i as u8];
            // `dot` is what we want the dot product of the new vector with the
            // previous one to be.
            let dot = (std::f32::consts::PI / edge as f32).cos();
            // Since there's only one axis shared between the last vector and
            // the new one, only that axis will affect the dot product.
            let y = dot / q;
            // Compute the other nonzero axis of the new vector such that the
            // vector will be normalized.
            let z = (1.0 - y * y).sqrt();
            // Actually construct that vector.
            last = Vector::EMPTY;
            last[i as u8] = y;
            last[i as u8 + 1] = z;
        }
        ret.push(Mirror(last));
        ret
    }

    pub fn group(self) -> Group {
        let gens: Vec<_> = self.mirrors().into_iter().map(|m| m.into()).collect();
        Group::from_generators(&gens)
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
pub struct Mirror(pub Vector<f32>);
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
