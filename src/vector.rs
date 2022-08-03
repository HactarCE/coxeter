use itertools::Itertools;
use num_traits::{Num, Signed};
use std::fmt;
use std::ops::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Vector<N>(Vec<N>);

impl<N: fmt::Display> fmt::Display for Vector<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut iter = self.0.iter();
        if let Some(first) = iter.next() {
            write!(f, "{first}")?;
            for elem in iter {
                write!(f, ", {elem}")?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

#[macro_export]
macro_rules! vector {
    [$($tok:tt)*] => {
        Vector(vec![$($tok)*])
    };
}

macro_rules! define_zero_padded_op {
    (impl $trait_name:ident { fn $fn_name:ident() }) => {
        impl<'a, N: Clone + Num> $trait_name for &'a Vector<N> {
            type Output = Vector<N>;

            fn $fn_name(self, rhs: Self) -> Self::Output {
                let result_ndim = std::cmp::max(self.ndim(), rhs.ndim());
                let lhs = self.iter().pad_using(result_ndim as _, |_| N::zero());
                let rhs = rhs.iter().pad_using(result_ndim as _, |_| N::zero());
                Vector(lhs.zip(rhs).map(|(l, r)| l.$fn_name(r)).collect())
            }
        }
    };
}
define_zero_padded_op!(impl Add { fn add() });
define_zero_padded_op!(impl Sub { fn sub() });

impl<'a, N: Clone + Signed> Neg for &'a Vector<N> {
    type Output = Vector<N>;

    fn neg(self) -> Self::Output {
        self.iter().map(|n| -n).collect()
    }
}

impl<N> Index<u8> for Vector<N> {
    type Output = N;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}
impl<N> IndexMut<u8> for Vector<N> {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<N: Num + Clone> Vector<N> {
    pub const EMPTY: Self = Self(Vec::new());

    pub fn ndim(&self) -> u8 {
        self.0.len() as _
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = N> {
        self.0.iter().cloned()
    }

    pub fn unit(axis: u8) -> Self {
        let mut ret = vector![N::zero(); axis as _];
        ret[axis] = N::one();
        ret
    }
}

impl<N: Clone> IntoIterator for Vector<N> {
    type Item = N;

    type IntoIter = std::vec::IntoIter<N>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<N> FromIterator<N> for Vector<N> {
    fn from_iter<T: IntoIterator<Item = N>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_vector_add() {
        let v1 = vector![1, 2, -10];
        let v2 = vector![-5];
        assert_eq!(&v1 + &v2, vector![-4, 2, -10]);
        assert_eq!(&v2 + &v1, vector![-4, 2, -10]);
    }

    #[test]
    pub fn test_vector_sub() {
        let v1 = vector![1, 2, -10];
        let v2 = vector![-5];
        assert_eq!(&v1 - &v2, vector![6, 2, -10]);
        assert_eq!(&v2 - &v1, vector![-6, -2, 10]);
    }

    #[test]
    pub fn test_vector_neg() {
        let v1 = vector![1, 2, -10];
        assert_eq!(-&v1, vector![-1, -2, 10]);
    }
}
