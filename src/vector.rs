use itertools::Itertools;
use num_traits::{Num, Signed, Zero};
use std::fmt;
use std::iter::Cloned;
use std::marker::PhantomData;
use std::ops::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Vector<N: Clone + Num>(Vec<N>);

pub trait VectorRef<N: Clone + Num>: Sized {
    fn ndim(&self) -> u8;

    fn get(&self, idx: u8) -> N;

    fn iter(&self) -> VectorIter<'_, N, Self> {
        VectorIter {
            range: 0..self.ndim(),
            vector: self,
            _phantom: PhantomData,
        }
    }

    fn dot<V: VectorRef<N>>(&self, rhs: V) -> N
    where
        N: Num,
    {
        self.iter()
            .zip(rhs.iter())
            .map(|(l, r)| l * r)
            .fold(N::zero(), |l, r| l + r)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VectorIter<'a, N: Clone + Num, V: VectorRef<N>> {
    range: Range<u8>,
    vector: &'a V,
    _phantom: PhantomData<N>,
}
impl<N: Clone + Num, V: VectorRef<N>> Iterator for VectorIter<'_, N, V> {
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|i| self.vector.get(i))
    }
}
impl<N: Clone + Num> VectorRef<N> for Vector<N> {
    fn ndim(&self) -> u8 {
        self.0.len() as _
    }

    fn get(&self, idx: u8) -> N {
        self.0.get(idx as usize).cloned().unwrap_or(N::zero())
    }
}

impl<N: Clone + Num, V: VectorRef<N>> VectorRef<N> for &'_ V {
    fn ndim(&self) -> u8 {
        (*self).ndim()
    }

    fn get(&self, idx: u8) -> N {
        (*self).get(idx)
    }
}

impl<N: Clone + Num + fmt::Display> fmt::Display for Vector<N> {
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
    (impl<$num:ident> $trait_name:ident for $type_name:ty { fn $fn_name:ident() }) => {
        impl<$num: Clone + Num, T: VectorRef<$num>> $trait_name<T> for $type_name {
            type Output = Vector<$num>;

            fn $fn_name(self, rhs: T) -> Self::Output {
                use itertools::Itertools;

                let result_ndim = std::cmp::max(self.ndim(), rhs.ndim());
                let lhs = self.iter().pad_using(result_ndim as _, |_| N::zero());
                let rhs = rhs.iter().pad_using(result_ndim as _, |_| N::zero());
                lhs.zip(rhs).map(|(l, r)| l.$fn_name(r)).collect()
            }
        }
    };
}
macro_rules! impl_vector_ops {
    (impl<$num:ident> for $type_name:ty) => {
        define_zero_padded_op!(impl<$num> Add for $type_name { fn add() });
        define_zero_padded_op!(impl<$num> Sub for $type_name { fn sub() });

        impl<$num: Clone + num_traits::Signed> Neg for $type_name {
            type Output = Vector<N>;

            fn neg(self) -> Self::Output {
                self.iter().map(|n| -n).collect()
            }
        }

        impl<$num: Clone + Num> Mul<$num> for $type_name {
            type Output = Vector<$num>;

            fn mul(self, rhs: $num) -> Self::Output {
                self.iter().map(|x| x * rhs.clone()).collect()
            }
        }
    };
}
impl_vector_ops!(impl<N> for Vector<N>);
impl_vector_ops!(impl<N> for &'_ Vector<N>);

impl<N: Clone + Num> Index<u8> for Vector<N> {
    type Output = N;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}
impl<N: Clone + Num> Index<u8> for &'_ Vector<N> {
    type Output = N;

    fn index(&self, index: u8) -> &Self::Output {
        &self.0[index as usize]
    }
}
impl<N: Clone + Num> IndexMut<u8> for Vector<N> {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<N: Clone + Num> Vector<N> {
    pub fn iter(&self) -> impl '_ + Iterator<Item = N> {
        self.0.iter().cloned()
    }
}
impl<N: Clone + Num> Vector<N> {
    pub const EMPTY: Self = Self(Vec::new());

    pub fn unit(axis: u8) -> Self {
        let mut ret = vector![N::zero(); axis as _];
        ret[axis] = N::one();
        ret
    }
}

impl<N: Clone + Num> IntoIterator for Vector<N> {
    type Item = N;

    type IntoIter = std::vec::IntoIter<N>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<'a, N: Clone + Num> IntoIterator for &'a Vector<N> {
    type Item = N;

    type IntoIter = Cloned<std::slice::Iter<'a, N>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().cloned()
    }
}

impl<N: Clone + Num> FromIterator<N> for Vector<N> {
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
        assert_eq!(v2 + v1, vector![-4, 2, -10]);
    }

    #[test]
    pub fn test_vector_sub() {
        let v1 = vector![1, 2, -10];
        let v2 = vector![-5];
        assert_eq!(&v1 - &v2, vector![6, 2, -10]);
        assert_eq!(v2 - &v1, vector![-6, -2, 10]);
    }

    #[test]
    pub fn test_vector_neg() {
        let v1 = vector![1, 2, -10];
        assert_eq!(-&v1, vector![-1, -2, 10]);
        assert_eq!(-v1, vector![-1, -2, 10]);
    }

    #[test]
    pub fn test_dot_product() {
        let v1 = vector![1, 2, -10];
        let v2 = vector![-5, 16];
        assert_eq!(v1.dot(v2), 27);
    }
}
