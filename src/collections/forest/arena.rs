use core::slice::GetManyMutError;
use std::{
    ops::{Index as IndexRO, IndexMut},
    mem::{replace, MaybeUninit},
    intrinsics::transmute_unchecked,
    sync::{Mutex, Condvar, Arc},
    cell::SyncUnsafeCell,
    fmt::Debug
};

use cc_traits::{
    Collection, CollectionRef, CollectionMut,
    Get, GetMut, Insert, Remove,
    covariant_item_ref, covariant_item_mut
};
use thiserror::Error;

use super::GetManyMut;

#[derive(Debug, Error)]
#[error("lock was poisoned")]
pub struct PoisonError;
impl<T> From<std::sync::PoisonError<T>> for PoisonError {
    #[inline(always)]
    fn from(_value: std::sync::PoisonError<T>) -> Self {
        PoisonError
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid index combination")]
    GetManyMut,
    #[error("one of the indices is invalid")]
    NotOccupied,
    #[error(transparent)]
    Poison(#[from] PoisonError),
}
impl<const N: usize> From<GetManyMutError<N>> for Error {
    #[inline]
    fn from(_value: GetManyMutError<N>) -> Self {
        Self::GetManyMut
    }
}

#[derive(Debug)]
struct Semaphore {
    readers: u32,
    writers: u32
}
#[derive(Debug)]
struct RwLock<T> {
    semaphore: Mutex<Semaphore>,
    read_cond: Condvar,
    write_cond: Condvar,
    data: SyncUnsafeCell<T>
}
impl<T> RwLock<T> {
    #[inline]
    fn new(data: T) -> Self {
        Self {
            semaphore: Mutex::new(Semaphore { readers: 0, writers: 0 }),
            read_cond: Condvar::new(),
            write_cond: Condvar::new(),
            data: data.into(),
        }
    }
    #[inline]
    // TODO: just unwrap, poisoned data should cause ff
    fn aquire_read(&self) -> Result<&T, PoisonError> {
        let mut guard = self.semaphore.lock()?;
        while guard.writers != 0 {
            guard = self.read_cond.wait(guard)?
        }
        guard.readers += 1;
        Ok(unsafe { &*self.data.get() })
    }
    #[allow(clippy::mut_from_ref)]
    #[inline]
    unsafe fn unsafe_aquire_read(&self) -> Result<&mut T, PoisonError> {
        // SAFETY: caller has to guaranty that write only occures while holding write access to inner lock
        let mut guard = self.semaphore.lock()?;
        while guard.writers != 0 {
            guard = self.read_cond.wait(guard)?
        }
        guard.readers += 1;
        Ok(unsafe { &mut *self.data.get() })
    }
    #[inline]
    fn free_read(&self) -> Result<(), PoisonError> {
        let mut guard = self.semaphore.lock()?;
        guard.readers -= 1;
        if guard.readers == 0 && guard.writers != 0 {
            self.write_cond.notify_one();
        }
        Ok(())
    }
    #[allow(clippy::mut_from_ref)]
    #[inline]
    fn aquire_write(&self) -> Result<&mut T, PoisonError> {
        let mut guard = self.semaphore.lock()?;
        guard.writers += 1;
        while guard.readers != 0 {
            guard = self.write_cond.wait(guard)?;
        }
        Ok(unsafe { &mut *self.data.get() })
    }
    #[inline]
    fn free_write(&self) -> Result<(), PoisonError> {
        let mut guard = self.semaphore.lock()?;
        guard.writers -= 1;
        if guard.writers != 0 {
            self.write_cond.notify_one();
        } else {
            self.read_cond.notify_all();
        }
        Ok(())
    }
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
    pub fn into_port(self) -> Port<T> {
        Port(Arc::new(RwLock::new(self)), Arc::new(RwLock::new(())))
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
    pub fn contains(&self, index: Index) -> bool {
        matches!(self.items.get(index.0), Some(Entry::Occupied(_)))
    }
    #[inline]
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.items.get_mut(index.0) {
            Some(Entry::Occupied(value)) => Some(value),
            _ => None
        }
    }
    #[inline]
    pub fn get_many_mut<const N: usize>(&mut self, indices: [Index; N]) -> Result<[&mut T; N], Error> {
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
impl<T> Collection for Arena<T> {
    type Item = T;
}
impl<T> CollectionRef for Arena<T> {
    type ItemRef<'a> = &'a T where Self: 'a;
    covariant_item_ref!();
}
impl<T> CollectionMut for Arena<T> {
    type ItemMut<'a> = &'a mut T where Self: 'a;
    covariant_item_mut!();
}
impl<T> Insert for Arena<T> {
    type Output = Index;
    #[inline(always)]
    fn insert(&mut self, value: Self::Item) -> Self::Output {
        self.insert(value)
    }
}
impl<T> Remove<Index> for Arena<T> {
    #[inline(always)]
    fn remove(&mut self, index: Index) -> Option<Self::Item> {
        self.remove(index)
    }
}
impl<T> Get<Index> for Arena<T> {
    #[inline(always)]
    fn get(&self, index: Index) -> Option<Self::ItemRef<'_>> {
        self.get(index)
    }
    #[inline(always)]
    fn contains(&self, index: Index) -> bool {
        self.contains(index)
    }
}
impl<T> GetMut<Index> for Arena<T> {
    #[inline(always)]
    fn get_mut(&mut self, index: Index) -> Option<Self::ItemMut<'_>> {
        self.get_mut(index)
    }
}
impl<T> GetManyMut<Index> for Arena<T> {
    type Error = Error;
    #[inline(always)]
    fn get_many_mut<const N: usize>(&mut self, indices: [Index; N]) -> Result<[Self::ItemMut<'_>; N], Self::Error> {
        self.get_many_mut(indices)
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

#[derive(Debug, Clone)]
pub(super) struct Port<T>(Arc<RwLock<Arena<T>>>, Arc<RwLock<()>>);
impl<T> Port<T> {
    #[inline]
    pub fn split(&self) -> Self {
        Port(self.0.clone(), Arc::new(RwLock::new(())))
    }
    #[inline]
    pub fn read(&self) -> Result<PortReadGuard<T>, PoisonError> {
        let arena = self.0.aquire_read()?;
        self.1.aquire_read()?;
        Ok(PortReadGuard { items: &arena.items, port_lock: &self.1, arena_lock: &self.0 })
    }
    #[inline]
    pub fn write(&self) -> Result<PortWriteGuard<T>, PoisonError> {
        // SAFETY: only access to mutable reference is to port-owned items while owning write lock to port
        let arena = unsafe { self.0.unsafe_aquire_read()? };
        self.1.aquire_write()?;
        Ok(PortWriteGuard { items: &mut arena.items, port_lock: &self.1, arena_lock: &self.0 })
    }
    #[inline]
    pub fn insert(&mut self, value: T) -> Result<Index, PoisonError> {
        let arena = self.0.aquire_write()?;
        Ok(arena.insert(value))
    }
    #[inline]
    pub fn remove(&mut self, index: Index) -> Result<Option<T>, PoisonError> {
        let arena = self.0.aquire_write()?;
        Ok(arena.remove(index))
    }
}
impl<T> Collection for Port<T> {
    type Item = T;
}
impl<T> Insert for Port<T> {
    type Output = Index;
    #[inline(always)]
    fn insert(&mut self, value: Self::Item) -> Self::Output {
        self.insert(value).unwrap()
    }
}
impl<T> Remove<Index> for Port<T> {
    #[inline(always)]
    fn remove(&mut self, index: Index) -> Option<Self::Item> {
        self.remove(index).unwrap()
    }
}

pub(super) struct PortReadGuard<'a, T> {
    items: &'a [Entry<T>],
    port_lock: &'a RwLock<()>,
    arena_lock: &'a RwLock<Arena<T>>
}
impl<'a, T> PortReadGuard<'a, T> {
    #[inline]
    pub fn get(&self, index: Index) -> Option<&T> {
        match self.items.get(index.0) {
            Some(Entry::Occupied(value)) => Some(value),
            _ => None
        }
    }
    #[inline]
    pub fn contains(&self, index: Index) -> bool {
        matches!(self.items.get(index.0), Some(Entry::Occupied(_)))
    }
}
impl<'a, T> Collection for PortReadGuard<'a, T> {
    type Item = T;
}
impl<'a, T> CollectionRef for PortReadGuard<'a, T> {
    type ItemRef<'b> = &'b T where Self: 'b;
    covariant_item_ref!();
}
impl<'a, T> Get<Index> for PortReadGuard<'a, T> {
    #[inline(always)]
    fn get(&self, index: Index) -> Option<Self::ItemRef<'_>> {
        self.get(index)
    }
    #[inline(always)]
    fn contains(&self, index: Index) -> bool {
        self.contains(index)
    }
}
impl<'a, T> IndexRO<Index> for PortReadGuard<'a, T> {
    type Output = T;
    #[inline]
    fn index(&self, index: Index) -> &Self::Output {
        match self.get(index) {
            Some(value) => value,
            _ => panic!("{} is not a valid index", index.0)
        }
    }
}
impl<'a, T> Drop for PortReadGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        _ = self.port_lock.free_read();
        _ = self.arena_lock.free_read();
    }
}

pub(super) struct PortWriteGuard<'a, T> {
    items: &'a mut [Entry<T>],
    port_lock: &'a RwLock<()>,
    arena_lock: &'a RwLock<Arena<T>>
}
impl<'a, T> PortWriteGuard<'a, T> {
    #[inline]
    pub fn get(&self, index: Index) -> Option<&T> {
        match self.items.get(index.0) {
            Some(Entry::Occupied(value)) => Some(value),
            _ => None
        }
    }
    #[inline]
    pub fn contains(&self, index: Index) -> bool {
        matches!(self.items.get(index.0), Some(Entry::Occupied(_)))
    }
    #[inline]
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.items.get_mut(index.0) {
            Some(Entry::Occupied(value)) => Some(value),
            _ => None
        }
    }
    #[inline]
    pub fn get_many_mut<const N: usize>(&mut self, indices: [Index; N]) -> Result<[&mut T; N], Error> {
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
impl<'a, T> Collection for PortWriteGuard<'a, T> {
    type Item = T;
}
impl<'a, T> CollectionRef for PortWriteGuard<'a, T> {
    type ItemRef<'b> = &'b T where Self: 'b;
    covariant_item_ref!();
}
impl<'a, T> CollectionMut for PortWriteGuard<'a, T> {
    type ItemMut<'b> = &'b mut T where Self: 'b;
    covariant_item_mut!();
}
impl<'a, T> Get<Index> for PortWriteGuard<'a, T> {
    #[inline(always)]
    fn get(&self, index: Index) -> Option<Self::ItemRef<'_>> {
        self.get(index)
    }
    #[inline(always)]
    fn contains(&self, index: Index) -> bool {
        self.contains(index)
    }
}
impl<'a, T> GetMut<Index> for PortWriteGuard<'a, T> {
    #[inline(always)]
    fn get_mut(&mut self, index: Index) -> Option<Self::ItemMut<'_>> {
        self.get_mut(index)
    }
}
impl<'a, T> GetManyMut<Index> for PortWriteGuard<'a, T> {
    type Error = Error;
    #[inline(always)]
    fn get_many_mut<const N: usize>(&mut self, indices: [Index; N]) -> Result<[Self::ItemMut<'_>; N], Self::Error> {
        self.get_many_mut::<N>(indices)
    }
}
impl<'a, T> IndexRO<Index> for PortWriteGuard<'a, T> {
    type Output = T;
    #[inline]
    fn index(&self, index: Index) -> &Self::Output {
        match self.get(index) {
            Some(value) => value,
            _ => panic!("{} is not a valid index", index.0)
        }
    }
}
impl<'a, T> IndexMut<Index> for PortWriteGuard<'a, T> {
    #[inline]
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        match self.get_mut(index) {
            Some(value) => value,
            _ => panic!("{} is not a valid index", index.0)
        }
    }
}
impl<'a, T> Drop for PortWriteGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        _ = self.port_lock.free_write();
        _ = self.arena_lock.free_read();
    }
}