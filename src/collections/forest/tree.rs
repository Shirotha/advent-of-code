use std::sync::{Arc, Mutex};

use crate::*;

use super::*;

#[derive(Debug)]
pub struct Tree<K, V> {
    pub(super) forest: Arc<Mutex<Forest<K, V>>>,
    pub(super) root: NodeRef,
    pub(super) first: NodeRef,
    pub(super) last: NodeRef
}

impl<K, V> Tree<K, V> {
    #[inline]
    fn rotate_left(ptr: NodeIndex, root: &mut NodeRef, nodes: &mut Arena<Node<K, V>>) {
        let node = &nodes[ptr];
        let parent = node.parent;
        let right = node.right;
        unwrap! { (): {
            let left_right = nodes[right?].left;
            discard! {
                nodes[left_right?].parent = Some(ptr)
            };
            if let Some(parent) = parent {
                let parent_node = &mut nodes[parent];
                if parent_node.left.is_some_and( |left| left == ptr ) {
                    parent_node.left = right;
                } else {
                    parent_node.right = right;
                }
            } else {
                *root = right;
            }
        } }
    }
    #[inline]
    fn rotate_right(ptr: NodeIndex, root: &mut NodeRef, nodes: &mut Arena<Node<K, V>>) {
        let node = &nodes[ptr];
        let parent = node.parent;
        let left = node.left;
        unwrap! { (): {
            let right_left = nodes[left?].right;
            discard! {
                nodes[right_left?].parent = Some(ptr)
            };
            if let Some(parent) = parent {
                let parent_node = &mut nodes[parent];
                if parent_node.right.is_some_and( |right| right == ptr ) {
                    parent_node.right = left;
                } else {
                    parent_node.left = left;
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
    fn fix_delete(mut ptr: NodeIndex, root: &mut NodeRef, nodes: &mut Arena<Node<K, V>>) {
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
                node.color = Color::Black;
                return;
            }
        }
    }
}