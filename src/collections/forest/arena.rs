use core::slice::GetManyMutError;
use std::{
    ops::{Index as IndexRO, IndexMut},
    mem::{replace, MaybeUninit},
    intrinsics::transmute_unchecked,
    cell::SyncUnsafeCell,
    fmt::Debug, sync::Arc
};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard, RwLockUpgradableReadGuard};
use thiserror::Error;

use super::*;

#[derive_const(Debug, Error)]
pub enum Error {
    #[error("invalid index combination")]
    GetManyMut,
    #[error("one of the indices is invalid")]
    NotOccupied,
}
impl<const N: usize> const From<GetManyMutError<N>> for Error {
    #[inline]
    fn from(_value: GetManyMutError<N>) -> Self {
        Self::GetManyMut
    }
}

#[derive_const(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
#[cfg_attr(target_pointer_width = "64", rustc_layout_scalar_valid_range_end(0xffffffff_fffffffe))]
#[cfg_attr(target_pointer_width = "32", rustc_layout_scalar_valid_range_end(0xfffffffe))]
#[rustc_nonnull_optimization_guaranteed]
pub(super) struct Index(usize);
impl Index {
    #[inline(always)]
    const unsafe fn new_unchecked(value: usize) -> Self {
        // SAFETY: value != usize::MAX guarantied from caller
        Self(value)
    }
}

type Ref = Option<Index>;

#[derive(Debug)]
enum Entry<T> {
    Occupied(T),
    Free(Ref)
}
impl<T> Entry<T> {
    fn into_value(self) -> Option<T> {
        let Self::Occupied(value) = self else { return None };
        Some(value)
    }
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
    const fn new() -> Self {
        Self { items: Vec::new(), free: None, len: 0 }
    }
    #[inline]
    fn into_port(self) -> Port<T> {
        Port(Arc::new(RwLock::new(self.into())), Arc::new(RwLock::new(())))
    }
    #[inline]
    fn insert_within_capacity(&mut self, value: T) -> Result<Index, T> {
        // ASSERT: there is enough capacity available
        self.len += 1;
        match self.free {
            Some(head) => {
                let next = replace(&mut self.items[head.0], Entry::Occupied(value));
                match next {
                    Entry::Free(next) => self.free = next,
                    _ => panic!("this should never happen!")
                }
                Ok(head)
            },
            None => {
                // SAFETY: even for sizeof::<T>() == 1 memory will run out before reaching usize::MAX
                let index = unsafe { Index::new_unchecked(self.items.len()) };
                self.items.push_within_capacity(Entry::Occupied(value))
                    .map( |_| index)
                    .map_err( |e| e.into_value().unwrap() )
            }
        }
    }
    #[inline]
    fn reserve(&mut self) {
        if self.free.is_none() {
            self.items.reserve(1);
        }
    }
    #[inline]
    fn is_full(&self) -> bool {
        self.free.is_none() && self.items.len() == self.items.capacity()
    }
    #[inline]
    fn remove(&mut self, index: Index) -> Option<T> {
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
    fn get(&self, index: Index) -> Option<&T> {
        match self.items.get(index.0) {
            Some(Entry::Occupied(value)) => Some(value),
            _ => None
        }
    }
    #[inline]
    fn contains(&self, index: Index) -> bool {
        matches!(self.items.get(index.0), Some(Entry::Occupied(_)))
    }
    #[inline]
    fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.items.get_mut(index.0) {
            Some(Entry::Occupied(value)) => Some(value),
            _ => None
        }
    }
    #[inline]
    fn get_many_mut<const N: usize>(&mut self, indices: [Index; N]) -> Result<[&mut T; N], Error> {
        // SATEFY: Index is guarantied to have the same memory layout as usize
        let indices: [usize; N] = unsafe { transmute_unchecked(indices) };
        let entries = self.items.get_many_mut(indices)?;
        let mut result = MaybeUninit::uninit_array();
        for (result, entry) in result.iter_mut().zip(entries) {
            match entry {
                Entry::Occupied(value) => _ = result.write(value),
                _ => Err(Error::NotOccupied)?
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

#[derive(Debug)]
pub(super) struct Port<T>(Arc<RwLock<SyncUnsafeCell<Arena<T>>>>, Arc<RwLock<()>>);
impl<T> Port<T> {
    #[inline]
    pub fn split(&self) -> Self {
        Port(self.0.clone(), Arc::new(RwLock::new(())))
    }
    #[inline]
    pub fn read(&self) -> PortReadGuard<T> {
        let arena = self.0.read();
        let port = self.1.read();
        PortReadGuard { arena, _port: port }
    }
    #[inline]
    pub fn write(&self) -> PortWriteGuard<T> {
        // SAFETY: only access to mutable reference is to port-owned items while owning write lock to port
        let arena = self.0.read();
        let port = self.1.write();
        PortWriteGuard { arena, _port: port }
    }
    #[inline]
    pub fn alloc(&self) -> PortAllocGuard<T> {
        // SAFETY: only access to mutable reference is to port-owned items while owning write lock to port
        let arena = self.0.upgradable_read();
        let port = self.1.write();
        PortAllocGuard { arena, port }
    }
}

#[derive(Debug)]
pub(super) struct PortReadGuard<'a, T> {
    arena: RwLockReadGuard<'a, SyncUnsafeCell<Arena<T>>>,
    _port: RwLockReadGuard<'a, ()>
}
impl<'a, T> PortReadGuard<'a, T> {
    #[inline]
    fn arena(&self) -> &Arena<T> {
        // SAFETY: arena is not null
        unsafe { self.arena.get().as_ref().unwrap() }
    }
}
impl<'a, T> Reader<Index> for PortReadGuard<'a, T> {
    type Item = T;
    #[inline]
    fn get(&self, index: Index) -> Option<&T> {
        self.arena().get(index)
    }
    #[inline]
    fn contains(&self, index: Index) -> bool {
        self.arena().contains(index)
    }
}
impl_Index_for_Reader!(Index, PortReadGuard<'a, T>);

#[derive(Debug)]
pub(super) struct PortWriteGuard<'a, T> {
    arena: RwLockReadGuard<'a, SyncUnsafeCell<Arena<T>>>,
    _port: RwLockWriteGuard<'a, ()>
}
impl<'a, T> PortWriteGuard<'a, T> {
    #[inline]
    fn arena(&self) -> &Arena<T> {
        // SAFETY: arena is not null
        unsafe { self.arena.get().as_ref().unwrap() }
    }
    #[inline]
    fn arena_mut(&mut self) -> &mut Arena<T> {
        // SAFETY: arena is not null
        unsafe { self.arena.get().as_mut().unwrap() }
    }
}
impl<'a, T> Reader<Index> for PortWriteGuard<'a, T> {
    type Item = T;
    #[inline]
    fn get(&self, index: Index) -> Option<&T> {
        self.arena().get(index)
    }
    #[inline]
    fn contains(&self, index: Index) -> bool {
        self.arena().contains(index)
    }
}
impl_Index_for_Reader!(Index, PortWriteGuard<'a, T>);
impl<'a, T> Writer<Index, Error> for PortWriteGuard<'a, T> {
    #[inline]
    fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        self.arena_mut().get_mut(index)
    }
    #[inline]
    fn get_many_mut<const N: usize>(&mut self, indices: [Index; N]) -> Result<[&mut T; N], Error> {
        self.arena_mut().get_many_mut(indices)
    }
}
impl_IndexMut_for_Writer!(Index, PortWriteGuard<'a, T>);

#[derive(Debug)]
pub(super) struct PortAllocGuard<'a, T> {
    arena: RwLockUpgradableReadGuard<'a, SyncUnsafeCell<Arena<T>>>,
    port: RwLockWriteGuard<'a, ()>
}
impl<'a, T> PortAllocGuard<'a, T> {
    #[inline]
    fn downgrade(self) -> PortWriteGuard<'a, T> {
        let arena = RwLockUpgradableReadGuard::downgrade(self.arena);
        PortWriteGuard { arena, _port: self.port }
    }
    #[inline]
    fn arena(&self) -> &Arena<T> {
        // SAFETY: arena is not null
        unsafe { self.arena.get().as_ref().unwrap() }
    }
    #[inline]
    fn arena_mut(&mut self) -> &mut Arena<T> {
        // SAFETY: arena is not null
        unsafe { self.arena.get().as_mut().unwrap() }
    }
    #[inline]
    pub fn insert(&mut self, value: T) -> Index {
        if self.arena().is_full() {
            self.arena.with_upgraded( |arena|
                arena.get_mut().reserve()
            );
        }
        // SAFETY: space was reserved in advance
        unsafe { self.arena_mut().insert_within_capacity(value).unwrap_unchecked() }
    }
    #[inline]
    pub fn remove(&mut self, index: Index) -> Option<T> {
        // SAFETY: there can only be one upgradable lock, so this has exclusive access to the free list
        self.arena_mut().remove(index)
    }
}
impl<'a, T> Reader<Index> for PortAllocGuard<'a, T> {
    type Item = T;
    #[inline]
    fn get(&self, index: Index) -> Option<&T> {
        self.arena().get(index)
    }
    #[inline]
    fn contains(&self, index: Index) -> bool {
        self.arena().contains(index)
    }
}
impl_Index_for_Reader!(Index, PortAllocGuard<'a, T>);
impl<'a, T> Writer<Index, Error> for PortAllocGuard<'a, T> {
    #[inline]
    fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        self.arena_mut().get_mut(index)
    }
    #[inline]
    fn get_many_mut<const N: usize>(&mut self, indices: [Index; N]) -> Result<[&mut T; N], Error> {
        self.arena_mut().get_many_mut(indices)
    }
}
impl_IndexMut_for_Writer!(Index, PortAllocGuard<'a, T>);