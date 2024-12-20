use num_bigint::BigUint;
use num_traits::{One, ToPrimitive};

use crate::bits::LiteralType;
use std::cmp::Ordering;
use std::num::Wrapping;

pub type ShortType = u32;
pub const SHORT_BITS: usize = 32;

#[derive(Clone, Debug, Copy, PartialEq, Hash)]
pub struct ShortBitVec<const N: usize>(ShortType);

#[test]
fn test_max_legal_calc() {
    let k = ShortBitVec::<16>::max_legal();
    assert_eq!(k, 65535);
}

impl<const N: usize> ShortBitVec<N> {
    pub fn max_legal() -> u64 {
        (1_u64 << N) - 1
    }

    pub fn short(&self) -> ShortType {
        self.0
    }

    pub fn to_u128(&self) -> u128 {
        self.0 as u128
    }

    #[inline(always)]
    pub fn replace_bit(&self, index: usize, val: bool) -> Self {
        assert!(index < N);
        let m: ShortType = 1 << index;
        Self(if val { self.0 | m } else { self.0 & !m })
    }

    #[inline(always)]
    pub const fn get_bit(&self, index: usize) -> bool {
        self.0 & (1 << index) != 0
    }

    #[inline(always)]
    const fn bit_mask() -> ShortType {
        if N == SHORT_BITS {
            !0
        } else {
            (1 << N) - 1
        }
    }

    #[inline(always)]
    pub fn mask() -> Self {
        Self::bit_mask().into()
    }

    #[inline(always)]
    pub fn any(&self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub fn all(&self) -> bool {
        self.0 == Self::bit_mask()
    }

    #[inline(always)]
    pub fn xor(&self) -> bool {
        let mut ret = false;
        let mut x = self.0;
        for _ in 0..N {
            ret = ret ^ (x & 0x1 != 0);
            x = x >> 1;
        }
        ret
    }
}

impl<const N: usize> From<ShortType> for ShortBitVec<N> {
    #[inline(always)]
    fn from(t: ShortType) -> Self {
        Self(t & Self::bit_mask())
    }
}

impl<const N: usize> From<ShortBitVec<N>> for ShortType {
    #[inline(always)]
    fn from(t: ShortBitVec<N>) -> ShortType {
        t.0
    }
}

impl<const N: usize> std::ops::Add<ShortBitVec<N>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    #[inline(always)]
    fn add(self, rhs: ShortBitVec<N>) -> Self::Output {
        let tmp = (Wrapping(self.0) + Wrapping(rhs.0)).0;
        (tmp & ShortBitVec::<N>::bit_mask()).into()
    }
}

impl<const N: usize> std::ops::Sub<ShortBitVec<N>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    #[inline(always)]
    fn sub(self, rhs: ShortBitVec<N>) -> Self::Output {
        (Wrapping(self.0) + !Wrapping(rhs.0) + Wrapping(1_u32 as ShortType))
            .0
            .into()
    }
}

impl<const N: usize> std::ops::BitOr<ShortBitVec<N>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    #[inline(always)]
    fn bitor(self, rhs: ShortBitVec<N>) -> Self::Output {
        (self.0 | rhs.0).into()
    }
}

impl<const N: usize> std::ops::BitAnd<ShortBitVec<N>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    #[inline(always)]
    fn bitand(self, rhs: ShortBitVec<N>) -> Self::Output {
        (self.0 & rhs.0).into()
    }
}

impl<const N: usize> std::ops::BitXor<ShortBitVec<N>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    #[inline(always)]
    fn bitxor(self, rhs: ShortBitVec<N>) -> Self::Output {
        (self.0 ^ rhs.0).into()
    }
}

impl<const N: usize> std::ops::Not for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    #[inline(always)]
    fn not(self) -> Self::Output {
        (!self.0).into()
    }
}

impl<const N: usize> std::cmp::PartialOrd for ShortBitVec<N> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<const N: usize> std::ops::Shr<LiteralType> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    #[inline(always)]
    fn shr(self, rhs: LiteralType) -> Self::Output {
        let x: BigUint = self.0.into();
        let mask: BigUint = (BigUint::one() << N) - BigUint::one();
        ShortBitVec::<N>(((x >> rhs) & mask).to_u32().unwrap())
    }
}

impl<const M: usize, const N: usize> std::ops::Shr<ShortBitVec<M>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    #[inline(always)]
    fn shr(self, rhs: ShortBitVec<M>) -> Self::Output {
        let r = rhs.short() as LiteralType;
        self >> r
    }
}

impl<const N: usize> std::ops::Shl<LiteralType> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    #[inline(always)]
    fn shl(self, rhs: LiteralType) -> Self::Output {
        let x: BigUint = self.0.into();
        let mask: BigUint = (BigUint::one() << N) - BigUint::one();
        ShortBitVec::<N>(((x << rhs) & mask).to_u32().unwrap())
    }
}

impl<const M: usize, const N: usize> std::ops::Shl<ShortBitVec<M>> for ShortBitVec<N> {
    type Output = ShortBitVec<N>;

    #[inline(always)]
    fn shl(self, rhs: ShortBitVec<M>) -> Self::Output {
        let r: LiteralType = rhs.short() as LiteralType;
        self << r
    }
}
