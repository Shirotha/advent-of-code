use core::slice::GetManyMutError;
use std::{
    ops::{Index as IndexRO, IndexMut},
    mem::{replace, MaybeUninit},
    intrinsics::transmute_unchecked,
    cell::SyncUnsafeCell,
    fmt::Debug, sync::Arc
};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use cc_traits::{
    Collection, CollectionRef, CollectionMut,
    Get, GetMut, Insert, Remove,
    covariant_item_ref, covariant_item_mut, Len
};
use thiserror::Error;

use super::GetManyMut;

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
    Occupied(T, [Ref; 2]),
    Free(Ref)
}
impl<T> Entry<T> {
    #[inline]
    fn prev(&self) -> Option<&Ref> {
        match self {
            Entry::Occupied(_, [prev, _]) => Some(prev),
            _ => None
        }
    }
    #[inline]
    fn next(&self) -> Option<&Ref> {
        match self {
            Entry::Occupied(_, [_, next]) => Some(next),
            _ => None
        }
    }
    #[inline]
    fn prev_mut(&mut self) -> Option<&mut Ref> {
        match self {
            Entry::Occupied(_, [prev, _]) => Some(prev),
            _ => None
        }
    }
    #[inline]
    fn next_mut(&mut self) -> Option<&mut Ref> {
        match self {
            Entry::Occupied(_, [_, next]) => Some(next),
            _ => None
        }
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
        Port(Arc::new(RwLock::new(self.into())), Arc::new(RwLock::new(PortInfo::new())))
    }
    #[inline]
    fn insert(&mut self, value: T, parent: Ref) -> Index {
        self.len += 1;
        let index = match self.free {
            Some(head) => {
                let next = replace(&mut self.items[head.0], Entry::Occupied(value, [parent, None]));
                match next {
                    Entry::Free(next) => self.free = next,
                    _ => panic!("this should never happen!")
                }
                head
            },
            None => {
                // SAFETY: even for sizeof::<T>() == 1 memory will run out before reaching usize::MAX
                let index = unsafe { Index::new_unchecked(self.items.len()) };
                self.items.push(Entry::Occupied(value, [parent, None]));
                index
            }
        };
        if let Some(parent) = parent {
            *self.items[parent.0].next_mut().unwrap() = Some(index);
        }
        index
    }
    #[inline]
    fn remove(&mut self, index: Index) -> Option<(T, [Ref; 2])> {
        if index.0 >= self.items.len() {
            return None;
        }
        let entry = &mut self.items[index.0];
        match entry {
            Entry::Occupied(..) => {
                let old = replace(entry, Entry::Free(self.free));
                self.free = Some(index);
                match old {
                    Entry::Occupied(value, bounds @ [prev, next]) => {
                        // SAFETY: only occupied entries can be pointed at
                        if let Some(prev) = prev {
                            *self.items[prev.0].next_mut().unwrap() = next;
                        }
                        if let Some(next) = next {
                            *self.items[next.0].prev_mut().unwrap() = prev;
                        }
                        Some((value, bounds))
                    },
                    _ => panic!("this should never happen!")
                }
            },
            _ => None
        }
    }
    fn clear(&mut self, mut head: Index) {
        // ASSERT: head.bounds[0] == None
        while let entry @ Entry::Occupied(..) = &mut self.items[head.0] {
            let old = replace(entry, Entry::Free(self.free));
            self.free = Some(head);
            if let Some(index) = old.next().unwrap() {
                head = *index;
            } else { return; }
        }
    }
    #[inline]
    fn get(&self, index: Index) -> Option<&T> {
        match self.items.get(index.0) {
            Some(Entry::Occupied(value, ..)) => Some(value),
            _ => None
        }
    }
    #[inline]
    fn contains(&self, index: Index) -> bool {
        matches!(self.items.get(index.0), Some(Entry::Occupied(..)))
    }
    #[inline]
    fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        match self.items.get_mut(index.0) {
            Some(Entry::Occupied(value, ..)) => Some(value),
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
                Entry::Occupied(value, ..) => _ = result.write(value),
                _ => Err(Error::NotOccupied)?
            }
        }
        // SAFETY: initialized in previous loop
        Ok(unsafe { MaybeUninit::array_assume_init(result) })
    }
}

#[derive(Debug)]
struct PortInfo {
    bounds: [Ref; 2],
    len: usize
}
impl PortInfo {
    fn new() -> Self {
        Self { bounds: [None, None], len: 0 }
    }
}
// TODO: for splitting trees, need to be able to tranfer nodes to other port !! transfering nodes one-by-one will cause split to be O(n) !!
#[derive(Debug, Clone)]
pub(super) struct Port<T>(Arc<RwLock<SyncUnsafeCell<Arena<T>>>>, Arc<RwLock<PortInfo>>);
impl<T> Port<T> {
    #[inline]
    pub fn split(&self) -> Self {
        Port(self.0.clone(), Arc::new(RwLock::new(PortInfo::new())))
    }
    #[inline]
    pub fn read(&self) -> PortReadGuard<T> {
        let arena = self.0.read();
        let port = self.1.read();
        PortReadGuard { arena, port }
    }
    #[inline]
    pub fn write(&self) -> PortWriteGuard<T> {
        // SAFETY: only access to mutable reference is to port-owned items while owning write lock to port
        let arena = self.0.read();
        let port = self.1.write();
        PortWriteGuard { arena, port }
    }
    #[inline]
    pub fn insert(&mut self, value: T) -> Index {
        let mut arena = self.0.write();
        // SAFETY: can only be here while arena write lock is held
        let mut port = unsafe { self.1.make_write_guard_unchecked() };
        port.len += 1;
        let index = arena.get_mut().insert(value, port.bounds[1]);
        port.bounds[1] = Some(index);
        if port.bounds[0].is_none() {
            port.bounds[0] = Some(index);
        }
        index
    }
    #[inline]
    pub fn remove(&mut self, index: Index) -> Option<T> {
        let mut arena = self.0.write();
        let (value, [prev, next]) = arena.get_mut().remove(index)?;
        // SAFETY: can only be here while arena write lock is held
        let mut port = unsafe { self.1.make_write_guard_unchecked() };
        port.len -= 1;
        if prev.is_none() {
            port.bounds[0] = next;
        }
        if next.is_none() {
            port.bounds[1] = prev;
        }
        Some(value)
    }
    pub fn clear(&mut self) {
        let mut arena = self.0.write();
        // SAFETY: can only be here while arena write lock is held
        let mut port = unsafe { self.1.make_write_guard_unchecked() };
        if port.len == 0 { return; }
        // SAFETY: non-empty port will have a non-null head
        arena.get_mut().clear(port.bounds[0].unwrap());
        port.len = 0;
        port.bounds = [None, None];
    }
    #[inline]
    pub fn len(&self) -> usize {
        let _arena = self.0.read();
        let port = self.1.read();
        port.len
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        let _arena = self.0.read();
        let port = self.1.read();
        port.len == 0
    }
}
impl<T> const Collection for Port<T> {
    type Item = T;
}
impl<T> Insert for Port<T> {
    type Output = Index;
    #[inline(always)]
    fn insert(&mut self, value: Self::Item) -> Self::Output {
        self.insert(value)
    }
}
impl<T> Remove<Index> for Port<T> {
    #[inline(always)]
    fn remove(&mut self, index: Index) -> Option<Self::Item> {
        self.remove(index)
    }
}
impl<T> Len for Port<T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[derive(Debug)]
pub(super) struct PortReadGuard<'a, T> {
    arena: RwLockReadGuard<'a, SyncUnsafeCell<Arena<T>>>,
    port: RwLockReadGuard<'a, PortInfo>
}
impl<'a, T> PortReadGuard<'a, T> {
    #[inline]
    fn arena(&self) -> &'a Arena<T> {
        // SAFETY: arena is not null
        unsafe { self.arena.get().as_ref().unwrap() }
    }
    #[inline]
    pub fn get(&self, index: Index) -> Option<&'a T> {
        self.arena().get(index)
    }
    #[inline]
    pub fn contains(&self, index: Index) -> bool {
        self.arena().contains(index)
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.port.len
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.port.len == 0
    }
}
impl<'a, T> const Collection for PortReadGuard<'a, T> {
    type Item = T;
}
impl<'a, T> const CollectionRef for PortReadGuard<'a, T> {
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
impl<'a, T> Len for PortReadGuard<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[derive(Debug)]
pub(super) struct PortWriteGuard<'a, T> {
    arena: RwLockReadGuard<'a, SyncUnsafeCell<Arena<T>>>,
    port: RwLockWriteGuard<'a, PortInfo>
}
impl<'a, T> PortWriteGuard<'a, T> {
    #[inline]
    fn arena(&self) -> &'a Arena<T> {
        // SAFETY: arena is not null
        unsafe { self.arena.get().as_ref().unwrap() }
    }
    #[inline]
    fn arena_mut(&mut self) -> &'a mut Arena<T> {
        // SAFETY: arena is not null
        unsafe { self.arena.get().as_mut().unwrap() }
    }
    #[inline]
    pub fn get(&self, index: Index) -> Option<&'a T> {
        self.arena().get(index)
    }
    #[inline]
    pub fn contains(&self, index: Index) -> bool {
        self.arena().contains(index)
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.port.len
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.port.len == 0
    }
    #[inline]
    pub fn get_mut(&mut self, index: Index) -> Option<&'a mut T> {
        self.arena_mut().get_mut(index)
    }
    #[inline]
    pub fn get_many_mut<const N: usize>(&mut self, indices: [Index; N]) -> Result<[&'a mut T; N], Error> {
        self.arena_mut().get_many_mut(indices)
    }
}
impl<'a, T> const Collection for PortWriteGuard<'a, T> {
    type Item = T;
}
impl<'a, T> const CollectionRef for PortWriteGuard<'a, T> {
    type ItemRef<'b> = &'b T where Self: 'b;
    covariant_item_ref!();
}
impl<'a, T> const CollectionMut for PortWriteGuard<'a, T> {
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
impl<'a, T> Len for PortWriteGuard<'a, T> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}