use std::{
    ops::RangeInclusive,
    cmp::Ordering
};

use crate::{
    Reader, Writer,
    arena::{
        Meta, MetaMut,
        PortReadGuard, PortWriteGuard, PortAllocGuard
    },
    tree::{
        Error, Bounds, Tree, SearchResult,
        Node, NodeIndex,
        Value, Color
    }
};

impl<K: Ord, V: Value> Tree<K, V> {
    #[inline]
    pub fn read(&self) -> TreeReadGuard<K, V> {
        TreeReadGuard(self.port.read())
    }
    #[inline]
    pub fn write(&mut self) -> TreeWriteGuard<K, V> {
        TreeWriteGuard(self.port.write())
    }
    #[inline]
    pub fn alloc(&mut self) -> TreeAllocGuard<K, V> {
        TreeAllocGuard(self.port.alloc())
    }
}

#[derive(Debug)]
pub struct TreeReadGuard<'a, K: Ord, V: Value>(pub(crate) PortReadGuard<'a, Node<K, V>, Bounds>);

#[derive(Debug)]
pub struct TreeWriteGuard<'a, K: Ord, V: Value>(pub(crate) PortWriteGuard<'a, Node<K, V>, Bounds>);

#[derive(Debug)]
pub struct TreeAllocGuard<'a, K: Ord, V: Value>(pub(crate) PortAllocGuard<'a, Node<K, V>, Bounds>);
impl<'a, K: Ord, V: Value> TreeAllocGuard<'a, K, V> {
    #[inline]
    pub fn downgrade(self) -> TreeWriteGuard<'a, K, V> {
        TreeWriteGuard(self.0.downgrade())
    }
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> bool {
        // SAFETY: root is a node in tree
        match unsafe { Tree::search(self.0.meta().root, &key, &self.0) } {
            SearchResult::Here(ptr) => {
                self.0[ptr].value = value;
                return false;
            },
            SearchResult::Empty => {
                let ptr = Some(self.0.insert(Node::new(key, value, Color::Black)));
                let meta = self.0.meta_mut();
                meta.root = ptr;
                meta.range = [ptr, ptr]
            },
            SearchResult::LeftOf(parent) => {
                let ptr = self.0.insert(Node::new(key, value, Color::Red));
                // SAFETY: parent is a leaf
                unsafe { Tree::insert_at::<0>(ptr, parent, &mut self.0); }
            },
            SearchResult::RightOf(parent) => {
                let ptr = self.0.insert(Node::new(key, value, Color::Red));
                // SAFETY: parent is a leaf
                unsafe { Tree::insert_at::<1>(ptr, parent, &mut self.0); }
            }
        }
        true
    }
    /// # Safety
    /// Calling this function implicitly moves the node pointer into this tree,
    /// using the same pointer in a different tree is undefined behaviour.
    #[inline]
    pub(crate) unsafe fn insert_node(&mut self, ptr: NodeIndex) -> Result<(), Error> {
        let key = &self.0[ptr].key;
        // SAFETY: root is a node in tree
        match Tree::search(self.0.meta().root, key, &self.0) {
            SearchResult::Here(_) => return Err(Error::DuplicateKey),
            SearchResult::Empty => {
                // Case 1
                let meta = self.0.meta_mut();
                meta.root = Some(ptr);
                meta.range = [Some(ptr), Some(ptr)];
            },
            SearchResult::LeftOf(parent) =>
                // SAFETY: parent is a leaf
                Tree::insert_at::<0>(ptr, parent, &mut self.0),
            SearchResult::RightOf(parent) =>
                // SAFETY: parent is a leaf
                Tree::insert_at::<1>(ptr, parent, &mut self.0)
        }
        Ok(())
    }
    #[inline]
    pub fn remove(&mut self, key: K) -> Option<V> {
        // SAFETY: root is a node in tree
        match unsafe { Tree::search(self.0.meta().root, &key, &self.0) } {
            SearchResult::Here(ptr) => {
                // SAFETY: node is the result of a search in tree
                unsafe { Tree::remove_at(ptr, &mut self.0); }
                // SAFETY: node was found, so it exists
                Some(self.0.remove(ptr).unwrap().value)
            }
            _ => None
        }
    }
    /// # Safety
    /// The node pointer has to point to a node in this tree.
    /// This will also implicitly move the node pointer out of this tree,
    /// using the pointer for anything else than inserting it into a different tree is undefined behaviour.
    #[inline(always)]
    pub(crate) unsafe fn remove_node(&mut self, ptr: NodeIndex) {
        Tree::remove_at(ptr, &mut self.0);
    }
    #[inline]
    pub fn clear(&mut self) {
        let mut ptr = self.0.meta().range[0];
        while let Some(index) = ptr {
            ptr = self.0[index].order[1];
            self.0.remove(index);
        }
        let meta = self.0.meta_mut();
        meta.root = None;
        meta.range = [None, None];
        meta.len = 0;
    }
}

macro_rules! impl_Reader {
    ( $type:ident ) => {
        impl<'a, K: Ord, V: Value> Reader<&K> for $type <'a, K, V> {
            type Item = V;
            #[inline]
            fn get(&self, key: &K) -> Option<&V> {
                // SAFETY: root is a node in tree
                let ptr = unsafe { Tree::search(self.0.meta().root, key, &self.0) }
                    .into_here()?;
                Some(&self.0[ptr].value)
            }
            #[inline]
            fn contains(&self, key: &K) -> bool {
                // SAFETY: root is a node in tree
                unsafe { Tree::search(self.0.meta().root, key, &self.0) }
                    .is_here()
            }
        }
    };
}
impl_Reader!(TreeReadGuard);
impl_Reader!(TreeWriteGuard);
impl_Reader!(TreeAllocGuard);

macro_rules! impl_Writer {
    ( $type:ident ) => {
        impl<'a, K: Ord, V: Value> Writer<&K, Error> for $type <'a, K, V> {
            #[inline]
            fn get_mut(&mut self, key: &K) -> Option<&mut V> {
                // SAFETY: root is a node in tree
                let ptr = unsafe { Tree::search(self.0.meta().root, key, &self.0) }
                    .into_here()?;
                Some(&mut self.0[ptr].value)
            }
            #[inline]
            fn get_pair_mut(&mut self, a: &K, b: &K) -> Result<[Option<&mut V>; 2], Error> {
                if a == b {
                    return Err(Error::KeyAlias);
                }
                let root = self.0.meta().root;
                // SAFETY: root is a node in tree
                let a = unsafe { Tree::search(root, a, &self.0) };
                let b = unsafe { Tree::search(root, b, &self.0) };
                match (a, b) {
                    (SearchResult::Here(a), SearchResult::Here(b)) => {
                        // SAFETY: a and b are checked before this
                        let [a, b] = self.0.get_pair_mut(a, b).unwrap();
                        Ok([
                            a.map( |a| &mut a.value ),
                            b.map( |b| &mut b.value )
                        ])
                    },
                    (SearchResult::Here(a), _) => Ok([Some(&mut self.0[a].value), None]),
                    (_, SearchResult::Here(b)) => Ok([None, Some(&mut self.0[b].value)]),
                    _ => Ok([None, None])
                }
            }
            #[inline]
            fn get_mut_with<const N: usize>(&mut self, key: &K, others: [Option<&K>; N]) -> Result<(Option<&mut V>, [Option<&V>; N]), Error> {
                if others.iter().any( |k| k.is_some_and( |k| k == key ) ) {
                    return Err(Error::KeyAlias)
                }
                let root = self.0.meta().root;
                // SAFETY: root is a node in tree
                if let SearchResult::Here(x) = unsafe { Tree::search(root, key, &self.0) } {
                    let others = others.map( |k| k.and_then( |k|
                        unsafe { Tree::search(root, k, &self.0) }
                            .into_here()
                    ) );
                    // SAFETY: all keys are checked before this
                    let (x, others) = self.0.get_mut_with(x, others).unwrap();
                    Ok((
                        x.map( |x| &mut x.value ),
                        others.map( |x| x.map( |x| &x.value ) )
                    ))
                } else {
                    Ok((None, others.map( |k| k.and_then( |k| self.get(k) ) )))
                }
            }
        }
    };
}
impl_Writer!(TreeWriteGuard);
impl_Writer!(TreeAllocGuard);

macro_rules! impl_ReadOnly {
    ( $type:ident ) => {
        impl<'a, K: Ord, V: Value> $type <'a, K, V> {
            #[inline(always)]
            pub fn len(&self) -> usize {
                self.0.meta().len
            }
            #[inline(always)]
            pub fn is_empty(&self) -> bool {
                self.0.meta().len == 0
            }
            #[inline]
            pub fn min(&self) -> Option<&K> {
                let index = self.0.meta().range[0]?;
                Some(&self.0[index].key)
            }
            #[inline]
            pub fn max(&self) -> Option<&K> {
                let index = self.0.meta().range[1]?;
                Some(&self.0[index].key)
            }
            #[inline]
            pub fn range(&self) -> Option<RangeInclusive<&K>> {
                let [Some(min), Some(max)] = self.0.meta().range else { return None };
                Some((&self.0[min].key)..=(&self.0[max].key))
            }
            /// Searches the tree using the given comparison function.
            /// The tree has to be sorted by compare or the result of this are meaningless.
            ///
            /// This behavious like the standard libary binary search for slices.
            #[inline]
            pub fn search_by<F>(&self, compare: F) -> SearchResult<&K>
                where F: Fn(&K, &V) -> Ordering
            {
                // SAFETY: root is part of tree
                unsafe {
                    Tree::search_by(
                        self.0.meta().root,
                        |node| compare(&node.key, &node.value) ,
                        &self.0
                    ).map( |index| &self.0[index].key )
                }
            }
        }
    };
}
impl_ReadOnly!(TreeReadGuard);
impl_ReadOnly!(TreeWriteGuard);
impl_ReadOnly!(TreeAllocGuard);