use core::slice::GetManyMutError;
use std::{
    ops::{Index as IndexRO, IndexMut},
    mem::{replace, MaybeUninit},
    intrinsics::transmute_unchecked
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManyMutError<const N: usize> {
    #[error(transparent)]
    GetManyMutError(#[from] GetManyMutError<N>),
    #[error("one of the indices is invalid")]
    NotOccupied
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
#[cfg_attr(target_pointer_width = "64", rustc_layout_scalar_valid_range_end(0xffffffff_fffffffe))]
#[cfg_attr(target_pointer_width = "32", rustc_layout_scalar_valid_range_end(0xfffffffe))]
#[rustc_nonnull_optimization_guaranteed]
pub(super) struct Index(usize);
impl Index {
    #[inline(always)]
    pub const unsafe fn new_unchecked(value: usize) -> Self {
        // SAFETY: value != usize::MAX guarantied from caller
        Self(value)
    }
    #[inline]
    pub const fn new(value: usize) -> Option<Self> {
        if value != usize::MAX {
            // SAFETY: value if bounds checked
            Some(unsafe { Self(value) })
        } else { None }
    }
}

type Ref = Option<Index>;

#[derive(Debug)]
enum Entry<T> {
    Occupied(T),
    Free(Ref)
}

// ASSERT: user is responsible for dangling references
#[derive(Debug)]
pub(super) struct Arena<T> {
    items: Vec<Entry<T>>,
    free: Ref,
    len: usize
}
impl<T> Arena<T> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self { items: Vec::new(), free: None, len: 0 }
    }
    #[inline]
    pub fn insert(&mut self, value: T) -> Index {
        self.len += 1;
        match self.free {
            Some(head) => {
                let next = replace(&mut self.items[head.0], Entry::Occupied(value));
                match next {
                    Entry::Free(next) => self.free = next,
                    _ => panic!("this should never happen!")
                }
                head
            },
            None => {
                // SAFETY: even for sizeof::<T>() == 1 memory will run out before reaching usize::MAX
                let index = unsafe { Index::new_unchecked(self.items.len()) };
                self.items.push(Entry::Occupied(value));
                index
            }
        }
    }
    #[inline]
    pub fn remove(&mut self, index: Index) -> Option<T> {
        if index.0 >= self.items.len() {
            return None;
        }
        let entry = &mut self.items[index.0];
        match entry {
            Entry::Occupied(_) => {
                let old = replace(entry, Entry::Free(self.free));
                self.free = Some(index);
                match old {
                    Entry::Occupied(value) => Some(value),
                    _ => panic!("this should never happen!")
                }
            },
            _ => None
        }
    }
    #[inline]
    pub fn get(&self, index: Index) -> Option<&T> {
        match self.items.get(index.0) {
            Some(Entry::Occupied(value)) => Some(value),
            _ => None
        }
    }
    #[inline]
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.items.get_mut(index.0) {
            Some(Entry::Occupied(value)) => Some(value),
            _ => None
        }
    }
    #[inline]
    pub fn get_many_mut<const N: usize>(&mut self, indices: [Index; N]) -> Result<[&mut T; N], ManyMutError<N>> {
        // SATEFY: Index is guarantied to have the same memory layout as usize
        let indices = unsafe { transmute_unchecked(indices) };
        let entries = self.items.get_many_mut(indices)?;
        let mut result = MaybeUninit::uninit_array();
        for (result, entry) in result.iter_mut().zip(entries) {
            match entry {
                Entry::Occupied(value) => _ = result.write(value),
                _ => Err(ManyMutError::NotOccupied)?
            }
        }
        // SAFETY: initialized in previous loop
        Ok(unsafe { MaybeUninit::array_assume_init(result) })
    }
}

impl<T> IndexRO<Index> for Arena<T> {
    type Output = T;
    #[inline]
    fn index(&self, index: Index) -> &Self::Output {
        match self.get(index) {
            Some(value) => value,
            None => panic!("{} is not a valid index", index.0)
        }
    }
}

impl<T> IndexMut<Index> for Arena<T> {
    #[inline]
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(value) => value,
            None => panic!("{} is not a valid index", index.0)
        }
    }
}