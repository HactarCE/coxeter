use num_traits::Num;
use std::ops::*;

use crate::util::f32_approx_eq;
use crate::vector::{Vector, VectorRef};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Matrix<N: Clone + Num> {
    /// Number of dimensions in the matrix.
    ndim: u8,
    /// Elements stored in **column-major** order.
    elems: Vec<N>,
}
impl<N: Clone + Num> Matrix<N> {
    pub const EMPTY_IDENT: Self = Matrix {
        ndim: 0,
        elems: vec![],
    };

    pub fn zero(ndim: u8) -> Self {
        Self {
            ndim,
            elems: vec![N::zero(); ndim as usize * ndim as usize],
        }
    }
    pub fn ident(ndim: u8) -> Self {
        let mut ret = Self::zero(ndim);
        for i in 0..ndim {
            *ret.get_mut(i, i) = N::one();
        }
        ret
    }
    pub fn from_elems(elems: Vec<N>) -> Self {
        let ndim = (elems.len() as f64).sqrt() as u8;
        assert_eq!(ndim as usize * ndim as usize, elems.len());
        Matrix { ndim, elems }
    }
    pub fn from_cols<I>(cols: impl IntoIterator<IntoIter = I>) -> Self
    where
        I: ExactSizeIterator,
        I::Item: VectorRef<N>,
    {
        let cols = cols.into_iter();
        let ndim = cols.len() as u8;
        Self {
            ndim,
            elems: cols
                .flat_map(|col| (0..ndim).map(move |i| col.get(i)))
                .collect(),
        }
    }

    pub fn ndim(&self) -> u8 {
        self.ndim
    }

    pub fn get(&self, col: u8, row: u8) -> N {
        let ndim = self.ndim();
        if col < ndim && row < ndim {
            self.elems[col as usize * ndim as usize + row as usize].clone()
        } else if col == row {
            N::one()
        } else {
            N::zero()
        }
    }
    pub fn get_mut(&mut self, col: u8, row: u8) -> &mut N {
        let ndim = self.ndim();
        assert!(col < ndim);
        assert!(row < ndim);
        &mut self.elems[col as usize * ndim as usize + row as usize]
    }
    pub fn row(&self, row: u8) -> MatrixRow<'_, N> {
        MatrixRow { matrix: self, row }
    }
    pub fn col(&self, col: u8) -> MatrixCol<'_, N> {
        MatrixCol { matrix: self, col }
    }

    pub fn rows(&self) -> impl Iterator<Item = MatrixRow<'_, N>> {
        (0..self.ndim()).map(|i| self.row(i))
    }
    pub fn cols(&self) -> impl Iterator<Item = MatrixCol<'_, N>> {
        (0..self.ndim()).map(|i| self.col(i))
    }

    #[must_use]
    pub fn scale(mut self, scalar: N) -> Self {
        for elem in &mut self.elems {
            *elem = elem.clone() * scalar.clone();
        }
        self
    }
}
impl<N: Clone + Num> FromIterator<N> for Matrix<N> {
    fn from_iter<T: IntoIterator<Item = N>>(iter: T) -> Self {
        Self::from_elems(iter.into_iter().collect())
    }
}

macro_rules! matrix {
    ($([$($n:expr),* $(,)?]),* $(,)?) => {
        Matrix::from_elems(vec![$($($n),*),*])
    };
}

pub struct MatrixCol<'a, N: Clone + Num> {
    matrix: &'a Matrix<N>,
    col: u8,
}
impl<N: Clone + Num> VectorRef<N> for MatrixCol<'_, N> {
    fn ndim(&self) -> u8 {
        self.matrix.ndim()
    }

    fn get(&self, row: u8) -> N {
        self.matrix.get(self.col, row)
    }
}

pub struct MatrixRow<'a, N: Clone + Num> {
    matrix: &'a Matrix<N>,
    row: u8,
}
impl<N: Clone + Num> VectorRef<N> for MatrixRow<'_, N> {
    fn ndim(&self) -> u8 {
        self.matrix.ndim()
    }

    fn get(&self, col: u8) -> N {
        self.matrix.get(col, self.row)
    }
}

impl_vector_ops!(impl<N> for MatrixCol<'_, N>);
impl_vector_ops!(impl<N> for MatrixRow<'_, N>);

impl<'a, N: Clone + Num + std::fmt::Debug> Mul for &'a Matrix<N> {
    type Output = Matrix<N>;

    fn mul(self, rhs: Self) -> Self::Output {
        let new_ndim = std::cmp::max(self.ndim(), rhs.ndim());
        let mut new_matrix = Matrix::zero(new_ndim);

        for (i, self_col) in self.cols().enumerate() {
            for x in 0..new_ndim {
                let rhs_elem = rhs.get(x, i as _);
                for y in 0..new_ndim {
                    let self_elem = self_col.get(y);
                    *new_matrix.get_mut(x, y) =
                        new_matrix.get(x, y) + self_elem.clone() * rhs_elem.clone();
                }
            }
        }

        new_matrix
    }
}
impl<N: Clone + Num, V: VectorRef<N>> Mul<V> for Matrix<N> {
    type Output = Vector<N>;

    fn mul(self, rhs: V) -> Self::Output {
        (0..self.ndim()).map(|i| self.row(i).dot(&rhs)).collect()
    }
}

impl Matrix<f32> {
    pub fn approx_eq(&self, other: &Self) -> bool {
        let ndim = std::cmp::max(self.ndim(), other.ndim());
        (0..ndim).all(|x| (0..ndim).all(|y| f32_approx_eq(self.get(x, y), other.get(x, y))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply() {
        let m1 = matrix![[1, 2, 0, 0], [0, 1, 1, 0], [1, 1, 1, 0], [0, 0, 0, -3]];
        let m2 = matrix![[1, 2, 4], [2, 3, 2], [1, 1, 2]];
        assert_eq!(
            &m1 * &m2,
            matrix![[5, 8, 6, 0], [4, 9, 5, 0], [3, 5, 3, 0], [0, 0, 0, -3]]
        );
    }
}
