mod arena;
mod node;
mod tree;

use std::fmt::Debug;
use cc_traits::CollectionMut;

pub use arena::*;
pub use node::*;
pub use tree::*;

trait GetManyMut<T>: CollectionMut {
    type Error: Debug;
    fn get_many_mut<const N: usize>(&mut self, indices: [T; N]) -> Result<[Self::ItemMut<'_>; N], Self::Error>;
}

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