use super::*;

// TODO: allow for upgradable locks

impl<K: Ord, V> Tree<K, V> {
    #[inline]
    pub fn read(&self) -> Result<TreeReadGuard<K, V>, Error> {
        let nodes = self.port.read();
        Ok(TreeReadGuard { tree: self, nodes })
    }
    #[inline]
    pub fn write(&mut self) -> Result<TreeWriteGuard<K, V>, Error> {
        let nodes = self.port.write();
        Ok(TreeWriteGuard { tree: self, nodes })
    }
}

#[derive(Debug)]
pub struct TreeReadGuard<'a, K: Ord, V> {
    tree: &'a Tree<K, V>,
    nodes: PortReadGuard<'a, Node<K, V>>
}
impl<'a, K: Ord, V> Reader<&K> for TreeReadGuard<'a, K, V> {
    type Item = V;
    #[inline]
    fn get(&self, key: &K) -> Option<&V> {
        match Tree::search(self.tree.root, key, &self.nodes) {
            SearchResult::Here(ptr) => Some(&self.nodes[ptr].value),
            _ => None
        }
    }
    #[inline]
    fn contains(&self, key: &K) -> bool {
        matches!(Tree::search(self.tree.root, key, &self.nodes), SearchResult::Here(_))
    }
}

#[derive(Debug)]
pub struct TreeWriteGuard<'a, K: Ord, V> {
    tree: &'a Tree<K, V>,
    nodes: PortWriteGuard<'a, Node<K, V>>
}
impl<'a, K: Ord, V> Reader<&K> for TreeWriteGuard<'a, K, V> {
    type Item = V;
    #[inline]
    fn get(&self, key: &K) -> Option<&V> {
        match Tree::search(self.tree.root, key, &self.nodes) {
            SearchResult::Here(ptr) => Some(&self.nodes[ptr].value),
            _ => None
        }
    }
    #[inline]
    fn contains(&self, key: &K) -> bool {
        matches!(Tree::search(self.tree.root, key, &self.nodes), SearchResult::Here(_))
    }
}
impl<'a, K: Ord, V> Writer<&K, Error> for TreeWriteGuard<'a, K, V> {
    #[inline]
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match Tree::search(self.tree.root, key, &self.nodes) {
            SearchResult::Here(ptr) => Some(&mut self.nodes[ptr].value),
            _ => None
        }
    }
    #[inline]
    fn get_many_mut<const N: usize>(&mut self, keys: [&K; N]) -> Result<[&mut V; N], Error> {
        let mut ptrs = MaybeUninit::uninit_array::<N>();
        for (ptr, key) in ptrs.iter_mut().zip(keys) {
            match Tree::search(self.tree.root, key, &self.nodes) {
                SearchResult::Here(found) => _ = ptr.write(found),
                _ => Err(Error::GetManyMut)?
            }
        }
        let ptrs = unsafe { MaybeUninit::array_assume_init(ptrs) };
        let nodes = self.nodes.get_many_mut(ptrs)?;
        let mut result = MaybeUninit::uninit_array::<N>();
        for (result, node) in result.iter_mut().zip(nodes) {
            result.write(&mut node.value);
        }
        Ok(unsafe { MaybeUninit::array_assume_init(result) })
    }
}
