use itertools::{Itertools, Permutations};
use num_traits::{Num, Signed};
use std::ops::*;

use crate::util::{f32_approx_eq, permutation_parity};
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

    pub fn from_outer_product(u: impl VectorRef<N>, v: impl VectorRef<N>) -> Self {
        let dim = std::cmp::max(u.ndim(), v.ndim());
        let u = &u;
        let v = &v;
        Self::from_elems(
            (0..dim)
                .flat_map(|i| (0..dim).map(move |j| u.get(i) * v.get(j)))
                .collect(),
        )
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

    pub fn transform(&self, v: impl VectorRef<N>) -> Vector<N> {
        let ndim = std::cmp::max(self.ndim(), v.ndim());
        (0..ndim)
            .map(|i| {
                (0..ndim)
                    .map(|j| self.get(j, i) * v.get(j))
                    .fold(N::zero(), |a, b| a + b)
            })
            .collect()
    }

    pub fn determinant(&self) -> N
    where
        N: Signed,
    {
        (0..self.ndim)
            .permutations(self.ndim as _)
            .enumerate()
            .map(|(i, p)| {
                let parity = match permutation_parity(i) {
                    true => -N::one(),
                    false => N::one(),
                };
                p.into_iter()
                    .enumerate()
                    .map(|(j, k)| self.get(j as _, k))
                    .fold(N::one(), |x, y| x * y)
                    * parity
            })
            .fold(N::zero(), |x, y| x + y)
    }

    pub fn inverse(&self) -> Matrix<N>
    where
        N: Signed,
        N: Clone,
    {
        let determinant = self.determinant();
        let det = &determinant;
        Matrix::from_elems(
            (0..self.ndim)
                .flat_map(|j| {
                    (0..self.ndim).map(move |i| {
                        let mut a = self.clone();
                        for k in 0..self.ndim {
                            *a.get_mut(i, k) = N::zero();
                        }
                        *a.get_mut(i, j) = N::one();
                        a.determinant() / det.clone()
                    })
                })
                .collect(),
        )
    }

    pub fn transpose(&self) -> Matrix<N> {
        Matrix::from_cols(self.rows().collect::<Vec<_>>())
    }
}
impl<N: Clone + Num> FromIterator<N> for Matrix<N> {
    fn from_iter<T: IntoIterator<Item = N>>(iter: T) -> Self {
        Self::from_elems(iter.into_iter().collect())
    }
}

#[macro_export]
macro_rules! matrix {
    ($([$($n:expr),* $(,)?]),* $(,)?) => {
        Matrix::from_elems(vec![$($($n),*),*])
    };
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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
impl<'a, N: Clone + Num + std::fmt::Debug> Add for &'a Matrix<N> {
    type Output = Matrix<N>;

    fn add(self, rhs: Self) -> Self::Output {
        let new_ndim = std::cmp::max(self.ndim(), rhs.ndim());
        Matrix::from_elems(
            (0..new_ndim)
                .flat_map(|i| (0..new_ndim).map(move |j| self.get(i, j) + rhs.get(i, j)))
                .collect(),
        )
    }
}
impl<'a, N: Clone + Num + std::fmt::Debug> Sub for &'a Matrix<N> {
    type Output = Matrix<N>;

    fn sub(self, rhs: Self) -> Self::Output {
        let new_ndim = std::cmp::max(self.ndim(), rhs.ndim());
        Matrix::from_elems(
            (0..new_ndim)
                .flat_map(|i| (0..new_ndim).map(move |j| self.get(i, j) - rhs.get(i, j)))
                .collect(),
        )
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

    #[test]
    fn test_determinant() {
        // let m = matrix![[-2, -1, 2], [2, 1, 4], [-3, 3, -1]];
        // assert_eq!(m.determinant(), 54);
        //let m = matrix![[3, 7], [1, -4]];
        //assert_eq!(m.determinant(), -19);

        let m = matrix![[1, 2, 3, 4], [5, 6, 8, 7], [-10, 3, 6, 2], [3, 1, 4, 1]];
        assert_eq!(m.determinant(), -402);
    }

    #[test]
    fn test_inverse() {
        let m = matrix![[1., 0., 4.], [1., 1., 6.], [-3., 0., -10.]];
        assert_eq!(&m * &m.inverse(), Matrix::ident(3));
    }

    #[test]
    fn test_transpose() {
        let m = matrix![[1, 2, 3], [4, 5, 6], [7, 8, 9]].transpose();
        assert_eq!(m, matrix![[1, 4, 7], [2, 5, 8], [3, 6, 9]])
    }
}
