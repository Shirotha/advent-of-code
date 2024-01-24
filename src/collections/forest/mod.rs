mod arena;
mod node;
mod tree;

use std::{
    fmt::Debug,
    ops::{Index as IndexRO, IndexMut}
};

use arena::*;
use node::*;
pub use tree::*;

trait Reader<T> {
    type Item;
    fn get(&self, index: T) -> Option<&Self::Item>;
    fn contains(&self, index: T) -> bool;
}

pub(super) trait Writer<T, E>: Reader<T> {
    fn get_mut(&mut self, index: T) -> Option<&mut Self::Item>;
    fn get_many_mut<const N: usize>(&mut self, indices: [T; N]) -> Result<[&mut Self::Item; N], E>;
}

type NodeIndex = Index;
type NodeRef = Option<NodeIndex>;
type TreeIndex = Index;
type TreeRef = Option<TreeIndex>;
trait NodeReader<K, V> = Reader<Index, Item = Node<K, V>> + IndexRO<NodeIndex, Output = Node<K, V>>;
trait NodeWriter<K, V> = Writer<Index, arena::Error, Item = Node<K, V>> + IndexMut<NodeIndex, Output = Node<K, V>>;

#[derive(Debug)]
pub struct Forest<K, V> {
    node_arena: Arena<Node<K, V>>,
    tree_arena: Arena<Tree<K, V>>,
    front: TreeRef,
    back: TreeRef
}