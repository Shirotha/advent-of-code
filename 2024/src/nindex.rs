use itertools::izip;
use num_traits::{NumCast, PrimInt, ToPrimitive, cast};
use std::ops::{Add, AddAssign, Deref, Mul, Not, Sub, SubAssign};

pub trait CheckedAdd<Rhs = Self> {
    type Output;

    fn checked_add(self, rhs: Rhs) -> Option<Self::Output>;
}
pub trait CheckedSub<Rhs = Self> {
    type Output;

    fn checked_sub(self, rhs: Rhs) -> Option<Self::Output>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NIndex<'a, const N: usize> {
    index: [usize; N],
    size: &'a [usize; N],
}
impl<const N: usize> NIndex<'_, N> {
    pub const fn zero(size: &[usize; N]) -> NIndex<N> {
        NIndex {
            index: [0; N],
            size,
        }
    }
}
impl<const N: usize> Deref for NIndex<'_, N> {
    type Target = [usize; N];

    fn deref(&self) -> &Self::Target {
        &self.index
    }
}
impl<const N: usize> AddAssign<&[usize; N]> for NIndex<'_, N> {
    fn add_assign(&mut self, rhs: &[usize; N]) {
        for (x, s, r) in izip!(self.index.iter_mut(), self.size.iter(), rhs.iter()) {
            *x = (*x + r) % s;
        }
    }
}
impl<const N: usize> Add<&[usize; N]> for NIndex<'_, N> {
    type Output = Self;

    fn add(mut self, rhs: &[usize; N]) -> Self::Output {
        self += rhs;
        self
    }
}
impl<const N: usize> CheckedAdd<&[usize; N]> for NIndex<'_, N> {
    type Output = Self;

    fn checked_add(mut self, rhs: &[usize; N]) -> Option<Self::Output> {
        for (x, &s, &r) in izip!(self.index.iter_mut(), self.size.iter(), rhs.iter()) {
            *x = x.checked_add(r)?;
            if *x >= s {
                return None;
            }
        }
        Some(self)
    }
}
impl<const N: usize> Add<usize> for NIndex<'_, N> {
    type Output = Option<Self>;

    fn add(mut self, rhs: usize) -> Self::Output {
        if N == 0 {
            return None;
        }
        self.index[0] += rhs;
        let mut i = 1;
        while i < N {
            if let Some(overflow) = self.index[i - 1].checked_sub(self.size[i - 1]) {
                self.index[i - 1] = overflow % self.size[i - 1];
                self.index[i] += 1 + overflow / self.size[i - 1];
            } else {
                return Some(self);
            }
            i += 1;
        }
        if self.index[i - 1] < self.size[i - 1] {
            Some(self)
        } else {
            None
        }
    }
}
impl<const N: usize> SubAssign<&[usize; N]> for NIndex<'_, N> {
    fn sub_assign(&mut self, rhs: &[usize; N]) {
        for (x, &s, &r) in izip!(self.index.iter_mut(), self.size.iter(), rhs.iter()) {
            *x = (*x + s - r % s) % s;
        }
    }
}
impl<const N: usize> Sub<&[usize; N]> for NIndex<'_, N> {
    type Output = Self;

    fn sub(mut self, rhs: &[usize; N]) -> Self::Output {
        self -= rhs;
        self
    }
}
impl<const N: usize> CheckedSub<&[usize; N]> for NIndex<'_, N> {
    type Output = Self;

    fn checked_sub(mut self, rhs: &[usize; N]) -> Option<Self::Output> {
        for (x, &r) in self.index.iter_mut().zip(rhs) {
            *x = x.checked_sub(r)?;
        }
        Some(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Sign {
    Zero     = 0,
    Positive = 1,
    Negative = 2,
}
impl Sign {
    pub fn try_from_primitive(value: impl ToPrimitive) -> Option<Self> {
        match value.to_u8()? {
            0 | 3 => Some(Sign::Zero),
            1 => Some(Sign::Positive),
            2 => Some(Sign::Negative),
            _ => None,
        }
    }
}
impl ToPrimitive for Sign {
    fn to_i64(&self) -> Option<i64> {
        Some(*self as i64)
    }

    fn to_u64(&self) -> Option<u64> {
        Some(*self as u64)
    }
}
impl NumCast for Sign {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        Self::try_from_primitive(n)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NDir<T: PrimInt, const N: usize>(T);
impl<T: PrimInt, const N: usize> NDir<T, N> {
    pub fn new(dirs: [Sign; N]) -> Self {
        assert!(size_of::<T>() * 4 >= N, "container type is too small");
        Self(
            dirs.into_iter()
                .rev()
                .fold(T::zero(), |value, dir| value << 2 | cast(dir).unwrap()),
        )
    }
    /// # Safety
    /// bits has to be a valid N-digit binary coded ternary number.
    pub const unsafe fn from_bits_unchecked(bits: T) -> Self {
        Self(bits)
    }
    pub fn dir(&self, index: usize) -> Sign {
        assert!(index < N, "index out of bounds");
        // SAFETY: any T can be trimmed to u8
        let dir = cast::<T, u8>(self.0 >> (2 * index)).unwrap() & 3;
        // SAFETY: all 2 bit values are valid
        Sign::try_from_primitive(dir).unwrap()
    }
    pub fn index(&self) -> usize {
        (0..N).fold(0, |result, index| result * 3 + self.dir(index) as usize)
    }
}
impl<T: PrimInt, const N: usize> Not for NDir<T, N> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
impl<T: PrimInt, const N: usize> AddAssign<NDir<T, N>> for NIndex<'_, N> {
    fn add_assign(&mut self, rhs: NDir<T, N>) {
        for (i, (x, &s)) in self.index.iter_mut().zip(self.size).enumerate() {
            match rhs.dir(i) {
                Sign::Zero => (),
                Sign::Positive => {
                    *x = (*x + 1) % s;
                }
                Sign::Negative => {
                    *x = (*x + s - 1) % s;
                }
            }
        }
    }
}
impl<T: PrimInt, const N: usize> Add<NDir<T, N>> for NIndex<'_, N> {
    type Output = Self;

    fn add(mut self, rhs: NDir<T, N>) -> Self::Output {
        self += rhs;
        self
    }
}
impl<T: PrimInt, const N: usize> CheckedAdd<NDir<T, N>> for NIndex<'_, N> {
    type Output = Self;

    fn checked_add(mut self, rhs: NDir<T, N>) -> Option<Self::Output> {
        for (i, (x, &s)) in self.index.iter_mut().zip(self.size).enumerate() {
            match rhs.dir(i) {
                Sign::Zero => (),
                Sign::Positive if *x + 1 < s => {
                    *x += 1;
                }
                Sign::Negative if *x != 0 => {
                    *x -= 1;
                }
                _ => return None,
            }
        }
        Some(self)
    }
}
impl<T: PrimInt, const N: usize> SubAssign<NDir<T, N>> for NIndex<'_, N> {
    fn sub_assign(&mut self, rhs: NDir<T, N>) {
        *self += !rhs;
    }
}
impl<T: PrimInt, const N: usize> Sub<NDir<T, N>> for NIndex<'_, N> {
    type Output = Self;

    fn sub(mut self, rhs: NDir<T, N>) -> Self::Output {
        self -= rhs;
        self
    }
}
impl<T: PrimInt, const N: usize> CheckedSub<NDir<T, N>> for NIndex<'_, N> {
    type Output = Self;

    fn checked_sub(self, rhs: NDir<T, N>) -> Option<Self::Output> {
        self.checked_add(!rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Directed<T: PrimInt, const N: usize> {
    len: usize,
    dir: NDir<T, N>,
}
impl<T: PrimInt, const N: usize> Not for Directed<T, N> {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        self.dir = !self.dir;
        self
    }
}
impl<T: PrimInt, const N: usize> Mul<usize> for NDir<T, N> {
    type Output = Directed<T, N>;

    fn mul(self, rhs: usize) -> Self::Output {
        Directed {
            len: rhs,
            dir: self,
        }
    }
}
impl<T: PrimInt, const N: usize> AddAssign<Directed<T, N>> for NIndex<'_, N> {
    fn add_assign(&mut self, rhs: Directed<T, N>) {
        for (i, (x, &s)) in self.index.iter_mut().zip(self.size).enumerate() {
            match rhs.dir.dir(i) {
                Sign::Zero => (),
                Sign::Positive => {
                    *x = (*x + rhs.len) % s;
                }
                Sign::Negative => {
                    *x = (*x + s - rhs.len % s) % s;
                }
            }
        }
    }
}
impl<T: PrimInt, const N: usize> Add<Directed<T, N>> for NIndex<'_, N> {
    type Output = Self;

    fn add(mut self, rhs: Directed<T, N>) -> Self::Output {
        self += rhs;
        self
    }
}
impl<T: PrimInt, const N: usize> CheckedAdd<Directed<T, N>> for NIndex<'_, N> {
    type Output = Self;

    fn checked_add(mut self, rhs: Directed<T, N>) -> Option<Self::Output> {
        for (i, (x, &s)) in self.index.iter_mut().zip(self.size).enumerate() {
            match rhs.dir.dir(i) {
                Sign::Zero => (),
                Sign::Positive if *x + rhs.len < s => {
                    *x += rhs.len;
                }
                Sign::Negative if *x >= rhs.len => {
                    *x -= rhs.len;
                }
                _ => return None,
            }
        }
        Some(self)
    }
}
impl<T: PrimInt, const N: usize> SubAssign<Directed<T, N>> for NIndex<'_, N> {
    fn sub_assign(&mut self, rhs: Directed<T, N>) {
        *self += !rhs;
    }
}
impl<T: PrimInt, const N: usize> Sub<Directed<T, N>> for NIndex<'_, N> {
    type Output = Self;

    fn sub(mut self, rhs: Directed<T, N>) -> Self::Output {
        self -= rhs;
        self
    }
}
impl<T: PrimInt, const N: usize> CheckedSub<Directed<T, N>> for NIndex<'_, N> {
    type Output = Self;

    fn checked_sub(self, rhs: Directed<T, N>) -> Option<Self::Output> {
        self.checked_add(!rhs)
    }
}
