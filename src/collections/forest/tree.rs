use std::sync::{Arc, Mutex};

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
    fn rotate_left(&mut self, ptr: NodeIndex, nodes: &mut Arena<Node<K, V>>) {
        let node = &nodes[ptr];
        let right = node.right;
        let parent = node.parent;
        if let Some(right) = right {
            let right_left =
                if let (Some(node), Some(right_node)) =
                    nodes.get2_mut(ptr, right)
                {
                    node.right = right_node.left;
                    right_node.parent = node.parent;
                    right_node.left
                } else {
                    panic!("this shouldn't happen");
                };
            if let Some(right_left) = right_left {
                nodes[right_left].parent = Some(ptr);
            }
        }
        if let Some(parent) = parent {
            let parent = &mut nodes[parent];
            if parent.left.is_some_and( |left| left == ptr ) {
                parent.left = right;
            } else {
                parent.right = right;
            }
        } else {
            self.root = right;
        }
    }
    #[inline]
    fn rotate_right(&mut self, ptr: NodeIndex, nodes: &mut Arena<Node<K, V>>) {
        let node = &nodes[ptr];
        let left = node.left;
        let parent = node.parent;
        if let Some(left) = left {
            let left_right =
                if let (Some(node), Some(left_node)) =
                    nodes.get2_mut(ptr, left)
                {
                    node.left = left_node.right;
                    left_node.parent = node.parent;
                    left_node.right
                } else {
                    panic!("this shouldn't happen");
                };
            if let Some(left_right) = left_right {
                nodes[left_right].parent = Some(ptr);
            }
        }
        if let Some(parent) = parent {
            let parent = &mut nodes[parent];
            if parent.right.is_some_and( |right| right == ptr ) {
                parent.right = left;
            } else {
                parent.left = left;
            }
        } else {
            self.root = left;
        }
    }
}

impl<K, V> Tree<K, V>
    where K: Ord
{
    fn fix_delete(&mut self, mut ptr: NodeIndex) {
        /*
        let mut lock = self.forest.lock()
            .expect("aquired lock");
        let nodes = &mut lock.node_arena;
        loop {
            let node = &mut nodes[ptr];
            if let (Some(parent), Color::Red) = (node.parent, node.color) {
                let parent_node = &nodes[parent];
                if parent_node.left.is_some_and( |parent_left| parent_left == ptr) {
                    if let Some(sibling) = parent_node.right {
                        let sibling =
                            if let (Some(parent_node), Some(sibling_node)) =
                                nodes.get2_mut(parent, sibling)
                            {
                                if sibling_node.is_red() {
                                    sibling_node.color = Color::Black;
                                    parent_node.color = Color::Red;
                                    //self.rotate_left(parent, nodes);
                                    todo!("return node.parent.right")
                                } else {
                                    todo!("return sibling")
                                }
                            } else {
                                panic!("this shouldn't happen");
                            };
                        
                    } else {
                        todo!("can this even happen?");
                    }
                } else {
                    todo!("node is right child")
                }
            } else {
                node.color = Color::Black;
                return;
            }
        }
         */
    }
}