use std::{sync::{Arc, Mutex, PoisonError}, cmp::Ordering};

use crate::*;

use super::*;

#[derive(Debug, Error)]
enum Error<K> {
    #[error("failed to aquire lock")]
    LockError,
    #[error("key already exists: {0}")]
    DuplicateKey(K)
}
impl<K, T> From<PoisonError<T>> for Error<K> {
    #[inline(always)]
    fn from(value: PoisonError<T>) -> Self {
        Self::LockError
    }
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
    pub(super) forest: Arc<Mutex<Forest<K, V>>>,
    pub(super) root: NodeRef,
    pub(super) first: NodeRef,
    pub(super) last: NodeRef
}

impl<K, V> Tree<K, V> {
    #[inline]
    fn rotate_left(ptr: NodeIndex, root: &mut NodeIndex, nodes: &mut Arena<Node<K, V>>) {
        // ASSERT: node has a non-null right child
        let node = &nodes[ptr];
        let parent = node.parent;
        unwrap! { (): {
            let right = node.right?;
            let left_right = nodes[right].left;
            discard! {
                nodes[left_right?].parent = Some(ptr)
            };
            if let Some(parent) = parent {
                let parent_node = &mut nodes[parent];
                if parent_node.left.is_some_and( |left| left == ptr ) {
                    parent_node.left = Some(right);
                } else {
                    parent_node.right = Some(right);
                }
            } else {
                *root = right;
            }
        } }
    }
    #[inline]
    fn rotate_right(ptr: NodeIndex, root: &mut NodeIndex, nodes: &mut Arena<Node<K, V>>) {
        // ASSERT: node has a non-null left child
        let node = &nodes[ptr];
        let parent = node.parent;
        unwrap! { (): {
            let left = node.left?;
            let right_left = nodes[left].right;
            discard! {
                nodes[right_left?].parent = Some(ptr)
            };
            if let Some(parent) = parent {
                let parent_node = &mut nodes[parent];
                if parent_node.right.is_some_and( |right| right == ptr ) {
                    parent_node.right = Some(left);
                } else {
                    parent_node.left = Some(left);
                }
            } else {
                *root = left;
            }
        } }
    }
}

impl<K, V> Tree<K, V>
    where K: Ord
{
    #[inline]
    fn search(mut ptr: NodeRef, key: &K, nodes: &Arena<Node<K, V>>) -> SearchResult<NodeIndex> {
        let (mut parent, mut left) = (None, false);
        while let Some(valid) = ptr {
            parent = ptr;
            let node = &nodes[valid];
            match node.key.cmp(key) {
                Ordering::Greater => {
                    left = true;
                    ptr = node.left;
                },
                Ordering::Equal => return SearchResult::Here(valid),
                Ordering::Less => {
                    left = false;
                    ptr = node.right;
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

    fn insert(&mut self, key: K, value: V) -> Result<(), Error<K>> {
        let mut lock = self.forest.lock()?;
        let nodes = &mut lock.node_arena;
        match Self::search(self.root, &key, nodes) {
            SearchResult::Here(_) => return Err(Error::DuplicateKey(key)),
            SearchResult::Empty => {
                // Case 1
                let ptr = nodes.insert(Node::root(key, value));
                self.root = Some(ptr);
                return Ok(())
            },
            SearchResult::LeftOf(parent) => {
                let parent_node = &nodes[parent];
                let prev = parent_node.prev;
                let node = Node::new(key, value, parent, prev, Some(parent));
                let ptr = nodes.insert(node);
                discard! {
                    nodes[prev?].next = Some(ptr)
                };
                let parent_node = &mut nodes[parent];
                parent_node.left = Some(ptr);
                parent_node.prev = Some(ptr);
                if parent_node.parent.is_some() {
                    Self::fix_insert(ptr, self.root.as_mut().unwrap(), nodes)
                }
            },
            SearchResult::RightOf(parent) => {
                let parent_node = &nodes[parent];
                let next = parent_node.next;
                let node = Node::new(key, value, parent, Some(parent), next);
                let ptr = nodes.insert(node);
                discard! {
                    nodes[next?].prev = Some(ptr)
                };
                let parent_node = &mut nodes[parent];
                parent_node.right = Some(ptr);
                parent_node.next = Some(ptr);
                if parent_node.parent.is_some() {
                    Self::fix_insert(ptr, self.root.as_mut().unwrap(), nodes)
                }
            }
        }
        Ok(())
    }
    #[inline]
    fn fix_insert(mut ptr: NodeIndex, root: &mut NodeIndex, nodes: &mut Arena<Node<K, V>>) {
        // ASSERT: node has a non-null grand-parent
        unwrap! { (): loop {
            let node = &nodes[ptr];
            let parent = node.parent?;
            let parent_node = &nodes[parent];
            if parent_node.is_black() {
                // Case 2
                break;
            }
            let is_left = parent_node.left.is_some_and( |left| left == ptr );
            let grandparent = parent_node.parent?;
            let grandparent_node = &nodes[grandparent];
            if grandparent_node.right? == parent{
                let uncle = grandparent_node.left?;
                let uncle_node = &mut nodes[uncle];
                if uncle_node.is_red() {
                    // Case 3.1
                    uncle_node.color = Color::Black;
                    nodes[parent].color = Color::Black;
                    nodes[grandparent].color = Color::Red;
                    ptr = grandparent;
                } else {
                    if is_left {
                        // Case 3.2.2
                        ptr = parent;
                        Self::rotate_right(ptr, root, nodes);
                    }
                    // Case 3.2.1
                    let parent = nodes[ptr].parent?;
                    let parent_node = &mut nodes[parent];
                    parent_node.color = Color::Black;
                    let grandparent = parent_node.parent?;
                    nodes[grandparent].color = Color::Red;
                    Self::rotate_left(grandparent, root, nodes);
                }
            } else {
                let uncle = grandparent_node.right?;
                let uncle_node = &mut nodes[uncle];
                if uncle_node.is_red() {
                    // Case 3.1 (mirror)
                    uncle_node.color = Color::Black;
                    nodes[parent].color = Color::Black;
                    nodes[grandparent].color = Color::Red;
                    ptr = grandparent;
                } else {
                    if !is_left {
                        // Case 3.2.2 (mirror)
                        ptr = parent;
                        Self::rotate_left(ptr, root, nodes);
                    }
                    // Case 3.2.1 (mirror)
                    let parent = nodes[ptr].parent?;
                    let parent_node = &mut nodes[parent];
                    parent_node.color = Color::Black;
                    let grandparent = parent_node.parent?;
                    nodes[grandparent].color = Color::Red;
                    Self::rotate_right(grandparent, root, nodes);
                }
            }
            if ptr == *root { break }
        } };
        nodes[*root].color = Color::Black
    }

    fn delete(&mut self, key: &K) -> Result<Option<V>, Error<K>> {
        let mut lock = self.forest.lock()?;
        let nodes = &mut lock.node_arena;
        unwrap! { (): if let SearchResult::Here(ptr) = Self::search(self.root, key, nodes) {
            let node = &nodes[ptr];
            let mut color = node.color;
            let fix = if node.left.is_none() {
                let fix = node.right?;
                Self::transplant(ptr, node.right, &mut self.root, nodes);
                fix
            } else if node.right.is_none() {
                let fix = node.left?;
                Self::transplant(ptr, node.left, &mut self.root, nodes);
                fix
            } else {
                let min = Self::min(ptr, nodes);
                let min_node = &nodes[min];
                color = min_node.color;
                let fix = min_node.right?;
                if min_node.parent.is_some_and( |parent| parent == ptr ) {
                    nodes[fix].parent = Some(min);
                } else {
                    Self::transplant(min, nodes[min].right, &mut self.root, nodes);
                    let right = nodes[ptr].right;
                    nodes[min].right = right;
                    nodes[right?].parent = Some(min);
                }
                Self::transplant(ptr, Some(min), &mut self.root, nodes);
                let node = &nodes[ptr];
                let left = node.left;
                let color = node.color;
                let min_node = &mut nodes[min];
                min_node.left = left;
                min_node.color = color;
                nodes[left?].parent = Some(min);
                fix
            };
            let node = nodes.remove(ptr)?;
            if color == Color::Black {
                Self::fix_delete(fix, self.root.as_mut().unwrap(), nodes)
            }
            return Ok(Some(node.value));
        } };
        Ok(None)
    }
    #[inline]
    fn transplant(ptr: NodeIndex, child: NodeRef, root: &mut NodeRef, nodes: &mut Arena<Node<K, V>>) {
        if let Some(parent) = nodes[ptr].parent {
            let parent_node = &mut nodes[parent];
            if parent_node.left.is_some_and( |left| left == ptr ) {
                parent_node.left = child;
            } else {
                parent_node.right = child;
            }
        } else {
            *root = child;
        }
    }
    #[inline]
    fn fix_delete(mut ptr: NodeIndex, root: &mut NodeIndex, nodes: &mut Arena<Node<K, V>>) {
        // ASSERT: node has at most 1 child
        /*
        loop {
            let node = &mut nodes[ptr];
            if let (Some(mut parent), Color::Black) = (node.parent, node.color) {
                let parent_node = &nodes[parent];
                if parent_node.left.is_some_and( |left| left == ptr ) {
                    let mut sibling = parent_node.right;
                    if let Some(sib) = sibling {
                        let sibling_node = &mut nodes[sib];
                        if sibling_node.is_red() {
                            sibling_node.color = Color::Black;
                            nodes[parent].color = Color::Red;
                            Self::rotate_left(parent, root, nodes);
                            parent = nodes[ptr].parent.unwrap();
                            sibling = nodes[parent].right;
                        }
                    }
                    // ...
                } else {
                    let sibling = parent_node.left;
                    todo!("node is right child");
                }
            } else {
                // Case 1
                node.color = Color::Black;
                return;
            }
        }
         */
    }

    #[inline]
    fn min(mut ptr: NodeIndex, nodes: &Arena<Node<K, V>>) -> NodeIndex {
        while let Some(left) = nodes[ptr].left {
            ptr = left;
        }
        ptr
    }
}