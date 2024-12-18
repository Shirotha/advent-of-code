use std::ops::{
    Bound,
    Range,
    RangeBounds,
    RangeFrom,
    RangeFull,
    RangeInclusive,
    RangeTo,
    RangeToInclusive,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RangeAny<T> {
    Range(Range<T>),
    RangeFrom(RangeFrom<T>),
    RangeFull(RangeFull),
    RangeInclusive(RangeInclusive<T>),
    RangeTo(RangeTo<T>),
    RangeToInclusive(RangeToInclusive<T>),
    Single(T),
}
impl<T> RangeBounds<T> for RangeAny<T> {
    fn start_bound(&self) -> Bound<&T> {
        match self {
            RangeAny::Range(range) => range.start_bound(),
            RangeAny::RangeFrom(range_from) => range_from.start_bound(),
            RangeAny::RangeFull(range_full) => range_full.start_bound(),
            RangeAny::RangeInclusive(range_inclusive) => range_inclusive.start_bound(),
            RangeAny::RangeTo(range_to) => range_to.start_bound(),
            RangeAny::RangeToInclusive(range_to_inclusive) => range_to_inclusive.start_bound(),
            RangeAny::Single(x) => Bound::Included(x),
        }
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        match self {
            RangeAny::Range(range) => range.end_bound(),
            RangeAny::RangeFrom(range_from) => range_from.end_bound(),
            RangeAny::RangeFull(range_full) => range_full.end_bound(),
            RangeAny::RangeInclusive(range_inclusive) => range_inclusive.end_bound(),
            RangeAny::RangeTo(range_to) => range_to.end_bound(),
            RangeAny::RangeToInclusive(range_to_inclusive) => range_to_inclusive.end_bound(),
            RangeAny::Single(x) => Bound::Included(x),
        }
    }
}
// TODO: generalize this
impl RangeAny<usize> {
    pub const fn start(&self) -> Option<usize> {
        match self {
            RangeAny::Range(range) => Some(range.start),
            RangeAny::RangeFrom(range_from) => Some(range_from.start),
            RangeAny::RangeInclusive(range_inclusive) => Some(*range_inclusive.start()),
            RangeAny::Single(x) => Some(*x),
            _ => None,
        }
    }
    pub const fn end(&self) -> Option<usize> {
        match self {
            RangeAny::Range(range) => range.end.checked_sub(1),
            RangeAny::RangeInclusive(range_inclusive) => Some(*range_inclusive.end()),
            RangeAny::RangeTo(range_to) => range_to.end.checked_sub(1),
            RangeAny::RangeToInclusive(range_to_inclusive) => Some(range_to_inclusive.end),
            RangeAny::Single(x) => Some(*x),
            _ => None,
        }
    }
}
macro_rules! impl_from_range {
    ($type: ident) => {
        impl<T> From<$type<T>> for RangeAny<T> {
            fn from(value: $type<T>) -> Self {
                Self::$type(value)
            }
        }
    };
}
impl_from_range!(Range);
impl_from_range!(RangeFrom);
impl_from_range!(RangeInclusive);
impl_from_range!(RangeTo);
impl_from_range!(RangeToInclusive);
impl<T> From<RangeFull> for RangeAny<T> {
    fn from(value: RangeFull) -> Self {
        Self::RangeFull(value)
    }
}
// TODO: generalize this
impl From<usize> for RangeAny<usize> {
    fn from(value: usize) -> Self {
        Self::Single(value)
    }
}
