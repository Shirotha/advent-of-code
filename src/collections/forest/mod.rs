mod node;
mod tree;

use generational_arena::{Arena, Index};

pub use node::*;
pub use tree::*;

type NodeIndex = Index;
type NodeRef = Option<NodeIndex>;
type TreeIndex = Index;
type TreeRef = Option<TreeIndex>;

#[derive(Debug)]
pub struct Forest<K, V> {
    node_arena: Arena<Node<K, V>>,
    tree_arena: Arena<Tree<K, V>>,
    front: TreeRef,
    back: TreeRef
}