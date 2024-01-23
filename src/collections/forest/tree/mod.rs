use std::{
    cmp::Ordering,
    ops::{Index, IndexMut},
    mem::MaybeUninit
};

use cc_traits::{
    Collection, CollectionRef, CollectionMut, Keyed, KeyedRef,
    Get, GetMut, GetKeyValue, GetKeyValueMut, MapInsert, Remove, Len,
    covariant_item_ref, covariant_item_mut, covariant_key_ref
};

mod concurrent;
mod cursor;
mod iter;

pub use concurrent::*;
pub use cursor::*;
pub use iter::*;

use crate::*;
use super::*;

#[derive_const(Debug, Error)]
pub enum Error {
    #[error("key already exists")]
    DuplicateKey,
    #[error("invalid key combination")]
    GetManyMut,
    #[error(transparent)]
    Arena(#[from] arena::Error)
}

#[derive(Debug)]
enum SearchResult<T> {
    Empty,
    LeftOf(T),
    Here(T),
    RightOf(T)
}

#[derive(Debug)]
pub struct Tree<K, V> {
    pub(super) port: Port<Node<K, V>>,
    pub(super) root: NodeRef,
    pub(super) bounds: [NodeRef; 2],
}

impl<K, V> Tree<K, V> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.port.len()
    }
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.port.is_empty()
    }
    #[inline]
    fn rotate<const I: usize>(ptr: NodeIndex,
        root: &mut NodeIndex, nodes: &mut PortWriteGuard<Node<K, V>>
    ) where [(); 1 - I]: {
        // ASSERT: node has a non-null right child
        let node = &nodes[ptr];
        let parent = node.parent;
        unwrap! { (): {
            // SAFETY: guarantied by caller
            let other = node.children[1 - I]?;
            let child_other = nodes[other].children[I];
            discard! {
                nodes[child_other?].parent = Some(ptr)
            };
            if let Some(parent) = parent {
                let parent_node = &mut nodes[parent];
                if parent_node.children[I].is_some_and( |child| child == ptr ) {
                    parent_node.children[I] = Some(other);
                } else {
                    parent_node.children[1 - I] = Some(other);
                }
            } else {
                *root = other;
            }
        } }
    }
}

impl<K: Ord, V> Tree<K, V>
{
    #[inline]
    fn search<C>(mut ptr: NodeRef, key: &K,
        nodes: &C
    ) -> SearchResult<NodeIndex>
        where C: Index<NodeIndex, Output = Node<K, V>>
    {
        let (mut parent, mut left) = (None, false);
        while let Some(valid) = ptr {
            parent = ptr;
            let node = &nodes[valid];
            match node.key.cmp(key) {
                Ordering::Greater => {
                    left = true;
                    ptr = node.children[0];
                },
                Ordering::Equal => return SearchResult::Here(valid),
                Ordering::Less => {
                    left = false;
                    ptr = node.children[1];
                }
            }
        }
        if let Some(parent) = parent {
            if left {
                SearchResult::LeftOf(parent)
            } else {
                SearchResult::RightOf(parent)
            }
        } else { SearchResult::Empty }
    }
    #[inline]
    pub fn contains(&self, key: &K) -> bool {
        if self.is_empty() { return false; }
        let nodes = self.port.read();
        matches!(Self::search(self.root, key, &nodes), SearchResult::Here(_))
    }
    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> bool {
        let mut nodes = self.port.write();
        match Self::search(self.root, &key, &nodes) {
            SearchResult::Here(ptr) => {
                nodes[ptr].value = value;
                return false;
            },
            SearchResult::Empty => {
                drop(nodes);
                let ptr = Some(self.port.insert(Node::new(key, value)));
                self.root = ptr;
                self.bounds = [ptr, ptr]
            },
            SearchResult::LeftOf(parent) => {
                drop(nodes);
                let ptr = self.port.insert(Node::new(key, value));
                let mut nodes = self.port.write();
                Self::insert_at::<0>(ptr, parent, &mut self.root.unwrap(), &mut self.bounds, &mut nodes);
            },
            SearchResult::RightOf(parent) => {
                drop(nodes);
                let ptr = self.port.insert(Node::new(key, value));
                let mut nodes = self.port.write();
                Self::insert_at::<1>(ptr, parent, &mut self.root.unwrap(), &mut self.bounds, &mut nodes);
            }
        }
        true
    }
    #[inline]
    pub(super) fn insert_node(&mut self, ptr: NodeIndex) -> Result<(), Error> {
        // ASSERT: node is not part of another tree
        let mut nodes = self.port.write();
        let key = &nodes[ptr].key;
        match Self::search(self.root, key, &nodes) {
            SearchResult::Here(_) => return Err(Error::DuplicateKey),
            SearchResult::Empty => {
                // Case 1
                self.root = Some(ptr);
                self.bounds = [Some(ptr), Some(ptr)];
            },
            SearchResult::LeftOf(parent) =>
                // SAFETY: search was succesful, so tree cannot be empty
                Self::insert_at::<0>(ptr, parent, &mut self.root.unwrap(), &mut self.bounds, &mut nodes),
            SearchResult::RightOf(parent) =>
                // SAFETY: search was succesful, so tree cannot be empty
                Self::insert_at::<1>(ptr, parent, &mut self.root.unwrap(), &mut self.bounds, &mut nodes)
        }
        Ok(())
    }
    #[inline]
    fn insert_at<const I: usize>(ptr: NodeIndex, parent: NodeIndex,
        root: &mut NodeIndex, bounds: &mut [NodeRef; 2],
        nodes: &mut PortWriteGuard<Node<K, V>>
    ) where [(); 1 - I]: {
        // ASSERT: child I is null
        let mut order = [None, None];
        order[I] = nodes[parent].order[I];
        order[1 - I] = Some(parent);
        let node = &mut nodes[ptr];
        node.parent = Some(parent);
        node.order = order;
        match order[I] {
            Some(far) => nodes[far].order[1 - I] = Some(ptr),
            None => bounds[I] = Some(ptr)
        }
        let parent_node = &mut nodes[parent];
        parent_node.children[I] = Some(ptr);
        parent_node.order[I] = Some(ptr);
        if parent_node.parent.is_some() {
            Self::fix_insert(ptr, root, nodes)
        }
    }
    #[inline]
    fn fix_insert(mut ptr: NodeIndex,
        root: &mut NodeIndex, nodes: &mut PortWriteGuard<Node<K, V>>
    ) {
        // ASSERT: node has a non-null grand-parent
        #[inline]
        fn helper<const I: usize, const J: usize, K, V>(mut ptr: NodeIndex, parent: NodeIndex, grandparent: NodeIndex,
            root: &mut NodeIndex, nodes: &mut PortWriteGuard<Node<K, V>>
        ) -> Option<NodeIndex>
            where [(); 1 - I]:, [(); 1 - J]:, [(); 1 - (1 - I)]:
        {
            let grandparent_node = &nodes[grandparent];
            // SAFETY: tree is balanced, so nodes on parent level cannot be null
            let uncle = grandparent_node.children[I]?;
            let uncle_node = &mut nodes[uncle];
            if uncle_node.is_red() {
                // Case 3.1
                uncle_node.color = Color::Black;
                nodes[parent].color = Color::Black;
                nodes[grandparent].color = Color::Red;
                ptr = grandparent;
            } else {
                if I == J {
                    // Case 3.2.2
                    ptr = parent;
                    Tree::rotate::<{1 - I}>(ptr, root, nodes);
                }
                // Case 3.2.1
                let parent = nodes[ptr].parent?;
                let parent_node = &mut nodes[parent];
                parent_node.color = Color::Black;
                // SAFETY: guarantied by caller
                let grandparent = parent_node.parent?;
                nodes[grandparent].color = Color::Red;
                Tree::rotate::<I>(grandparent, root, nodes);
            }
            Some(ptr)
        }
        
        unwrap! { (): loop {
            let node = &nodes[ptr];
            // SAFETY: node cannot be the root
            let parent = node.parent?;
            let parent_node = &nodes[parent];
            if parent_node.is_black() {
                // Case 2
                break;
            }
            let is_left = parent_node.children[0].is_some_and( |left| left == ptr );
            // SAFETY: guarantied by caller
            let grandparent = parent_node.parent?;
            // SAFETY: tree is balanced, so nodes on parent level cannot be null
            ptr = if nodes[grandparent].children[1]? == parent {
                if is_left {
                    helper::<0, 0, K, V>(ptr, parent, grandparent, root, nodes)?
                } else {
                    helper::<0, 1, K, V>(ptr, parent, grandparent, root, nodes)?
                }
            } else if is_left {
                helper::<1, 0, K, V>(ptr, parent, grandparent, root, nodes)?
            } else {
                helper::<1, 1, K, V>(ptr, parent, grandparent, root, nodes)?
            };
            if ptr == *root { break }
        } };
        nodes[*root].color = Color::Black
    }
    #[inline]
    pub fn remove(&mut self, key: K) -> Option<V> {
        let mut nodes = self.port.write();
        match Self::search(self.root, &key, &nodes) {
            SearchResult::Here(ptr) => {
                Self::remove_at(ptr, &mut self.root, &mut self.bounds, &mut nodes);
                drop(nodes);
                // SAFETY: node was found, so it exists
                Some(self.port.remove(ptr).unwrap().value)
            }
            _ => None
        }
    }
    #[inline]
    pub(super) fn remove_at(ptr: NodeIndex,
        root: &mut NodeRef, bounds: &mut [NodeRef; 2],
        nodes: &mut PortWriteGuard<Node<K, V>>,
    ) {
        // ASSERT: root is the root of node
        unwrap! { (): {
            let node = &nodes[ptr];
            let mut color = node.color;
            let [prev, next] = node.order;
            let fix = if node.children[0].is_none() {
                // SAFETY: 
                let fix = node.children[1];
                Self::transplant(ptr, fix, root, nodes);
                fix
            } else if node.children[1].is_none() {
                let fix = node.children[0];
                Self::transplant(ptr, fix, root, nodes);
                fix
            } else {
                // SAFETY: node has a right child, so has to have a succsesor
                let min = nodes[ptr].order[1]?;
                let min_node = &nodes[min];
                color = min_node.color;
                let fix = min_node.children[1];
                if min_node.parent.is_some_and( |parent| parent == ptr ) {
                    // SAFETY: node has both children in this branch
                    nodes[fix?].parent = Some(min);
                } else {
                    Self::transplant(min, nodes[min].children[1], root, nodes);
                    let right = nodes[ptr].children[1];
                    nodes[min].children[1] = right;
                    // SAFETY: node has both children in this branch
                    nodes[right?].parent = Some(min);
                }
                Self::transplant(ptr, Some(min), root, nodes);
                let node = &nodes[ptr];
                let left = node.children[0];
                let color = node.color;
                let min_node = &mut nodes[min];
                min_node.children[0] = left;
                min_node.color = color;
                // SAFETY: node has both children in this branch
                nodes[left?].parent = Some(min);
                fix
            };
            match prev {
                Some(prev) => nodes[prev].order[1] = next,
                None => bounds[0] = next
            }
            match next {
                Some(next) => nodes[next].order[0] = prev,
                None => bounds[1] = prev
            }
            // SAFETY: node was searched before
            // FIXME: use new system
            //let node = nodes.remove(ptr)?;
            if let (Some(fix), Color::Black) = (fix, color) {
                // SAFETY: search was successful, so tree cannot be empty
                Self::fix_remove(fix, root.as_mut().unwrap(), nodes)
            }
        } };
    }
    #[inline]
    fn transplant(ptr: NodeIndex, child: NodeRef,
        root: &mut NodeRef, nodes: &mut PortWriteGuard<Node<K, V>>
    ) {
        let parent = nodes[ptr].parent;
        discard! {
            nodes[child?].parent = parent
        };
        if let Some(parent) = parent {
            let parent_node = &mut nodes[parent];
            if parent_node.children[0].is_some_and( |left| left == ptr ) {
                parent_node.children[0] = child;
            } else {
                parent_node.children[1] = child;
            }
        } else {
            *root = child;
        }
    }
    #[inline]
    fn fix_remove(mut ptr: NodeIndex,
        root: &mut NodeIndex, nodes: &mut PortWriteGuard<Node<K, V>>
    ) {
        // ASSERT: node is black
        #[inline]
        fn helper<const I: usize, K, V>(mut ptr: NodeIndex, mut parent: NodeIndex,
            root: &mut NodeIndex, nodes: &mut PortWriteGuard<Node<K, V>>
        ) -> NodeIndex
            where [(); 1 - I]:, [(); 1 - (1 - I)]:
        {
            let parent_node = &nodes[parent];
            unwrap! { (): {
                // SAFETY: tree is balanced, so nodes on node level cannot be null
                let mut sibling = parent_node.children[1 - I]?;
                let sibling_node = &mut nodes[sibling];
                if sibling_node.is_red() {
                    // Case 3.1
                    sibling_node.color = Color::Black;
                    nodes[parent].color = Color::Red;
                    Tree::rotate::<I>(parent, root, nodes);
                    // SAFETY: tree is balanced, so nodes on parent level cannot be null
                    parent = nodes[ptr].parent?;
                    // SAFETY: tree is balanced, so nodes on node level cannot be null
                    sibling = nodes[parent].children[1 - I]?;
                }
                let nephews = nodes[sibling].children;
                let is_black = !nephews[1 - I].is_some_and( |nephew| nodes[nephew].is_red() );
                if !nephews[I].is_some_and( |nephew| nodes[nephew].is_red() ) && is_black {
                    // Case 3.2
                    nodes[sibling].color = Color::Red;
                    ptr = parent;
                } else {
                    if is_black {
                        // Case 3.3
                        discard! {
                            nodes[nephews[I]?].color = Color::Black
                        };
                        nodes[sibling].color = Color::Red;
                        Tree::rotate::<{1 - I}>(sibling, root, nodes);
                        // SAFETY: tree is balanced, so nodes on parent level cannot be null
                        parent = nodes[ptr].parent?;
                        // SAFETY: tree is balanced, so nodes on node level cannot be null
                        sibling = nodes[parent].children[1 - I]?;
                    }
                    // Case 3.4
                    // SAFETY: sibling is child of parent, both exist
                    let [sibling_node, parent_node] = nodes.get_many_mut([sibling, parent])?;
                    sibling_node.color = parent_node.color;
                    parent_node.color = Color::Black;
                    // SAFETY: tree is balanced, so nodes on node level cannot be null
                    let nephew = sibling_node.children[1 - I]?;
                    nodes[nephew].color = Color::Black;
                    Tree::rotate::<I>(parent, root, nodes);
                    ptr = *root;
                }
            } };
            ptr
        }

        loop {
            let node = &mut nodes[ptr];
            if let (Some(parent), Color::Black) = (node.parent, node.color) {
                // Case 3
                ptr = if nodes[parent].children[0].is_some_and( |left| left == ptr ) {
                    helper::<0, K, V>(ptr, parent, root, nodes)
                } else {
                    helper::<1, K, V>(ptr, parent, root, nodes)
                };
            } else {
                // Case 1
                node.color = Color::Black;
                return;
            }
        }
    }
    #[inline(always)]
    pub fn clear(&mut self) {
        self.port.clear()
    }
}