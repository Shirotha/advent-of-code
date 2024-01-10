/*
use std::{
    cmp::Ordering,
    ops::Range,
    mem::transmute
};

use smallvec::SmallVec;

/* self (A)  both {B}  other [C]
 *     <anti-symmetrical>                            LeftLess LeftEqual LeftGreater RightLess RightEqual RightGreater Overlap Symmetric
 * ( A ) [     B     ]         Left                      x                              x
 *                             LeftConnect               x                              x                                         x
 * (  A  [     B     }         FitLeftOuter              x                                        x                      x
 * (  A  [  B  )  C  ]         OverlapLeftOuter          x                              x                                x
 *       {  B  )  C  ]         FitLeftInner                      x                      x                                x
 *        < mirror >
 *       [  C  (  B  }         FitRightInner                                 x                    x                      x
 *       [  C  (  B  ]  A  )   OverlapRightOuter                             x                                x          x
 *       {     B     ]  A  )   FitRightOuter                     x                                            x          x
 *                             RightConnect                                  x                                x                   x
 *       [     B     ] ( A )   Right                                         x                                x
 * 
 *       <symmetrical>
 *       [ C ( B ) C ]         Inner                                         x          x                                x        x
 *       {     B     }         Equal                             x                                x                      x        x
 * (  A  [     B     ]  A  )   Outer                     x                                                    x          x        x
 */
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Overlap {
    Left              = 0x90,
    LeftConnect       = 0x91,
    FitLeftOuter      = 0x8a,
    OverlapLeftOuter  = 0x92,
    FitLeftInner      = 0x52,
    FitRightInner     = 0x2a,
    OverlapRightOuter = 0x26,
    FitRightOuter     = 0x46,
    RightConnect      = 0x25,
    Right             = 0x24,
    Inner             = 0x33,
    Equal             = 0x4b,
    Outer             = 0x87,
}
impl Overlap {
    #[inline] pub const fn is_symmetric(&self) -> bool { (*self as u8) & 0x03 == 0x03 }
    #[inline] pub const fn is_overlapping(&self) -> bool { (*self as u8) & 0x02 == 0x02 }
    #[inline] pub const fn reverse(&self) -> Self {
        let bits = *self as u8;
        if bits & 0x01 == 0x01 { return *self; }
        let swap = (bits >> 3) | ((bits & 0x1c) << 3);
        unsafe { transmute((swap & 0x48 | ((swap & 0x90) >> 2) | ((swap & 0x24) << 2)) | (bits & 0x03)) }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Influence<T> {
    Neither(Interval<T>),
    Left(Interval<T>),
    Right(Interval<T>),
    Both(Interval<T>)
}
impl<T> From<Influence<T>> for Interval<T> {
    #[inline]
    fn from(value: Influence<T>) -> Self {
        match value {
            Influence::Neither(value)
            | Influence::Left(value)
            | Influence::Right(value)
            | Influence::Both(value)
            => value,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Interval<T> {
    min: T,
    max: T
}
impl<T> Interval<T> {
    #[inline]
    pub const fn new(min: T, max: T) -> Self {
        // ASSERT: min <= max
        Self { min, max }
    }
    #[inline]
    pub const fn min(&self) -> &T {
        &self.min
    }
    #[inline]
    pub const fn max(&self) -> &T {
        &self.max
    }
}
impl<T: Ord> Interval<T> {
    #[inline]
    pub fn contains(&self, value: &T) -> bool {
        &self.min <= value && value < &self.max
    }
    #[inline]
    pub fn compare(&self, value: &T) -> Ordering {
        if value < &self.min { Ordering::Less }
        else if value >= &self.max { Ordering::Greater }
        else { Ordering::Equal }
    }
    #[inline]
    pub fn overlap(&self, other: &Self) -> Overlap {
        match self.partial_cmp(other) {
            Some(Ordering::Less) => if self.max == other.min { Overlap::LeftConnect } else { Overlap::Left },
            Some(Ordering::Equal) => Overlap::Equal,
            Some(Ordering::Greater) => if other.max == self.min { Overlap::RightConnect } else { Overlap::Right },
            None => {
                let (left, right) = (self.min.cmp(&other.min), self.max.cmp(&other.max));
                unsafe { transmute::<u8, Overlap>(match left {
                    Ordering::Less => 0x80,
                    Ordering::Equal => 0x40,
                    Ordering::Greater => 0x20
                } | match right {
                    Ordering::Less => 0x10,
                    Ordering::Equal => 0x08,
                    Ordering::Greater => 0x04
                } | if left == right.reverse() { 0x03 } else { 0x02 }) }
            }
        }
    }
}
impl<T: Ord + Clone + Default> Interval<T> {
    #[inline]
    pub fn iter_segments(&self, other: &Self) -> impl Iterator<Item = Influence<T>> {
        match self.overlap(other) {
            Overlap::Left => SmallVec::from_buf([
                Influence::Left(self.clone()),
                Influence::Neither(Interval::new(self.max.clone(), other.min.clone())),
                Influence::Right(other.clone())
            ]),
            Overlap::LeftConnect => SmallVec::from_buf_and_len([
                Influence::Left(self.clone()),
                Influence::Right(other.clone()),
                Influence::Neither(Interval::default())
            ], 2),
            Overlap::FitLeftOuter => SmallVec::from_buf_and_len([
                Influence::Left(Interval::new(self.min.clone(), other.min.clone())),
                Influence::Both(other.clone()),
                Influence::Neither(Interval::default())
            ], 2),
            Overlap::OverlapLeftOuter => SmallVec::from_buf([
                Influence::Left(Interval::new(self.min.clone(), other.min.clone())),
                Influence::Both(Interval::new(other.min.clone(), self.max.clone())),
                Influence::Right(Interval::new(self.max.clone(), other.max.clone()))
            ]),
            Overlap::FitLeftInner => SmallVec::from_buf_and_len([
                Influence::Both(self.clone()),
                Influence::Right(Interval::new(self.max.clone(), other.max.clone())),
                Influence::Neither(Interval::default())
            ], 2),
            Overlap::FitRightInner => SmallVec::from_buf_and_len([
                Influence::Right(Interval::new(other.min.clone(), self.min.clone())),
                Influence::Both(self.clone()),
                Influence::Neither(Interval::default())
            ], 2),
            Overlap::OverlapRightOuter => SmallVec::from_buf([
                Influence::Right(Interval::new(other.min.clone(), self.min.clone())),
                Influence::Both(Interval::new(self.min.clone(), other.max.clone())),
                Influence::Left(Interval::new(other.max.clone(), self.max.clone()))
            ]),
            Overlap::FitRightOuter => SmallVec::from_buf_and_len([
                Influence::Both(other.clone()),
                Influence::Left(Interval::new(other.max.clone(), self.max.clone())),
                Influence::Neither(Interval::default())
            ], 2),
            Overlap::RightConnect => SmallVec::from_buf_and_len([
                Influence::Right(other.clone()),
                Influence::Left(other.clone()),
                Influence::Neither(Interval::default())
            ], 2),
            Overlap::Right => SmallVec::from_buf([
                Influence::Right(other.clone()),
                Influence::Neither(Interval::new(other.max.clone(), self.min.clone())),
                Influence::Left(self.clone())
            ]),
            Overlap::Inner => SmallVec::from_buf([
                Influence::Right(Interval::new(other.min.clone(), self.min.clone())),
                Influence::Both(self.clone()),
                Influence::Right(Interval::new(self.max.clone(), other.max.clone()))
            ]),
            Overlap::Equal => SmallVec::from_buf_and_len([
                Influence::Both(self.clone()),
                Influence::Neither(Interval::default()),
                Influence::Neither(Interval::default())
            ], 1),
            Overlap::Outer => SmallVec::from_buf([
                Influence::Left(Interval::new(self.min.clone(), other.min.clone())),
                Influence::Both(other.clone()),
                Influence::Left(Interval::new(other.max.clone(), self.max.clone()))
            ]),
        }.into_iter()
    }
}
impl<T: Ord> PartialEq for Interval<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.min == other.min && self.max == other.max
    }
}
impl<T: Ord> Eq for Interval<T> {}
impl<T: Ord> PartialOrd for Interval<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.max <= other.min { Some(Ordering::Less) }
        else if self.min >= other.max { Some(Ordering::Greater) }
        else if self == other { Some(Ordering::Equal) }
        else { None }
    }
}
impl<T> From<Range<T>> for Interval<T> {
    #[inline]
    fn from(value: Range<T>) -> Self {
        Self::new(value.start, value.end)
    }
}
impl<T> From<Interval<T>> for Range<T> {
    #[inline]
    fn from(value: Interval<T>) -> Self {
        value.min..value.max
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::bool_assert_comparison)]
    use super::*;
    use duplicate::duplicate;

    #[test]
    fn overlap_symmetric() {
        use Overlap::*;
        duplicate! {[
            value               result;
            [Left]              [false];
            [FitLeftOuter]      [false];
            [OverlapLeftOuter]  [false];
            [FitLeftInner]      [false];
            [OverlapRightOuter] [false];
            [FitRightOuter]     [false];
            [Right]             [false];
            [Inner]             [true];
            [Equal]             [true];
            [Outer]             [true];
        ]
            assert_eq!(value.is_symmetric(), result);
        }
    }

    #[test]
    fn overlap_overlapping() {
        use Overlap::*;
        duplicate! {[
            value               result;
            [Left]              [false];
            [FitLeftOuter]      [true];
            [OverlapLeftOuter]  [true];
            [FitLeftInner]      [true];
            [OverlapRightOuter] [true];
            [FitRightOuter]     [true];
            [Right]             [false];
            [Inner]             [true];
            [Equal]             [true];
            [Outer]             [true];
        ]
            assert_eq!(value.is_overlapping(), result);
        }
    }

    #[test]
    fn overlap_reverse() {
        use Overlap::*;
        duplicate! {[
            left                right;
            [Left]              [Right];
            [FitLeftOuter]      [FitRightOuter];
            [OverlapLeftOuter]  [OverlapRightOuter];
            [FitLeftInner]      [FitRightInner];
        ]
            assert_eq!(left.reverse(), right);
            assert_eq!(left, right.reverse());
        }
    }

    #[test]
    fn interval_contains() {
        duplicate! {[
            min max value result;
            [1] [3] [0]   [false];
            [1] [3] [1]   [true];
            [1] [3] [2]   [true];
            [1] [3] [3]   [false];
            [1] [3] [4]   [false];
        ]
            assert_eq!(Interval::new(min, max).contains(&value), result);
        }
    }

    #[test]
    fn interval_overlap() {
        use Overlap::*;
        duplicate! {[
            left_min left_max right_min right_max result;
            [0]      [1]      [2]       [6]       [Left];
            [0]      [6]      [2]       [6]       [FitLeftOuter];
            [0]      [4]      [2]       [6]       [OverlapLeftOuter];
            [2]      [4]      [2]       [6]       [FitLeftInner];
            [4]      [6]      [2]       [6]       [FitRightInner];
            [4]      [8]      [2]       [6]       [OverlapRightOuter];
            [2]      [8]      [2]       [6]       [FitRightOuter];
            [7]      [8]      [2]       [6]       [Right];
            [3]      [5]      [2]       [6]       [Inner];
            [2]      [6]      [2]       [6]       [Equal];
            [0]      [8]      [2]       [6]       [Outer];
        ]
            assert_eq!(Interval::new(left_min, left_max).overlap(&Interval::new(right_min, right_max)), result);
        }
    }
}
 */