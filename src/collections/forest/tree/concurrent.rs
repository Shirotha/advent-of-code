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

impl<K: Ord, V> const Collection for Tree<K, V> {
    type Item = V;
}
impl<K: Ord, V> MapInsert<K> for Tree<K, V> {
    type Output = bool;
    #[inline(always)]
    fn insert(&mut self, key: K, value: Self::Item) -> Self::Output {
        self.insert(key, value)
    }
}
impl<K: Ord, V> Remove<K> for Tree<K, V> {
    #[inline(always)]
    fn remove(&mut self, key: K) -> Option<Self::Item> {
        self.remove(key)
    }
}

#[derive(Debug)]
pub struct TreeReadGuard<'a, K: Ord, V> {
    tree: &'a Tree<K, V>,
    nodes: PortReadGuard<'a, Node<K, V>>
}
impl<'a, K: Ord, V> TreeReadGuard<'a, K, V> {
    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        match Tree::search(self.tree.root, key, &self.nodes) {
            SearchResult::Here(ptr) => Some(&self.nodes[ptr].value),
            _ => None
        }
    }
    #[inline]
    pub fn contains(&self, key: &K) -> bool {
        matches!(Tree::search(self.tree.root, key, &self.nodes), SearchResult::Here(_))
    }
}
impl<'a, K: Ord, V> const Collection for TreeReadGuard<'a, K, V> {
    type Item = V;
}
impl<'a, K: Ord, V> const CollectionRef for TreeReadGuard<'a, K, V> {
    type ItemRef<'b> = &'b V where Self: 'b;
    covariant_item_ref!();
}
impl<'a, K: Ord, V> const Len for TreeReadGuard<'a, K, V> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.tree.len()
    }
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
}
impl<'a, 'b, K: Ord, V> Get<&'b K> for TreeReadGuard<'a, K, V> {
    #[inline(always)]
    fn get(&self, key: &'b K) -> Option<Self::ItemRef<'_>> {
        self.get(key)
    }
    #[inline(always)]
    fn contains(&self, key: &'b K) -> bool {
        self.contains(key)
    }
}
impl<'a, K: Ord, V> const Keyed for TreeReadGuard<'a, K, V> {
    type Key = K;
}
impl<'a, K: Ord, V> const KeyedRef for TreeReadGuard<'a, K, V> {
    type KeyRef<'b> = &'b K where Self: 'b;
    covariant_key_ref!();
}
impl<'a, 'b, K: Ord, V> GetKeyValue<&'b K> for TreeReadGuard<'a, K, V> {
    #[inline]
    fn get_key_value(&self, key: &K) -> Option<(Self::KeyRef<'_>, Self::ItemRef<'_>)> {
        match Tree::search(self.tree.root, key, &self.nodes) {
            SearchResult::Here(ptr) => {
                let node = &self.nodes[ptr];
                Some((&node.key, &node.value))
            },
            _ => None
        }
    }
}
impl<'a, 'b, K: Ord, V> Index<&'b K> for TreeReadGuard<'a, K, V> {
    type Output = V;
    #[inline]
    fn index(&self, key: &K) -> &Self::Output {
        match self.get(key) {
            Some(value) => value,
            None => panic!("invalid key")
        }
    }
}

#[derive(Debug)]
pub struct TreeWriteGuard<'a, K: Ord, V> {
    tree: &'a Tree<K, V>,
    nodes: PortWriteGuard<'a, Node<K, V>>
}
impl<'a, K: Ord, V> TreeWriteGuard<'a, K, V> {
    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        match Tree::search(self.tree.root, key, &self.nodes) {
            SearchResult::Here(ptr) => Some(&self.nodes[ptr].value),
            _ => None
        }
    }
    #[inline]
    pub fn contains(&self, key: &K) -> bool {
        matches!(Tree::search(self.tree.root, key, &self.nodes), SearchResult::Here(_))
    }
    #[inline]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match Tree::search(self.tree.root, key, &self.nodes) {
            SearchResult::Here(ptr) => Some(&mut self.nodes[ptr].value),
            _ => None
        }
    }
    #[inline]
    pub fn get_many_mut<const N: usize>(&mut self, keys: [&K; N]) -> Result<[&mut V; N], Error> {
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
impl<'a, K: Ord, V> const Collection for TreeWriteGuard<'a, K, V> {
    type Item = V;
}
impl<'a, K: Ord, V> const CollectionRef for TreeWriteGuard<'a, K, V> {
    type ItemRef<'b> = &'b V where Self: 'b;
    covariant_item_ref!();
}
impl<'a, K: Ord, V> const CollectionMut for TreeWriteGuard<'a, K, V> {
    type ItemMut<'b> = &'b mut V where Self: 'b;
    covariant_item_mut!();
}
impl<'a, K: Ord, V> const Len for TreeWriteGuard<'a, K, V> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.tree.len()
    }
    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
}
impl<'a, 'b, K: Ord, V> Get<&'b K> for TreeWriteGuard<'a, K, V> {
    #[inline(always)]
    fn get(&self, key: &'b K) -> Option<Self::ItemRef<'_>> {
        self.get(key)
    }
    #[inline(always)]
    fn contains(&self, key: &'b K) -> bool {
        self.contains(key)
    }
}
impl<'a, 'b, K: Ord, V> GetMut<&'b K> for TreeWriteGuard<'a, K, V> {
    #[inline(always)]
    fn get_mut(&mut self, key: &'b K) -> Option<Self::ItemMut<'_>> {
        self.get_mut(key)
    }
}
impl<'a, 'b, K: Ord, V> GetManyMut<&'b K> for TreeWriteGuard<'a, K, V> {
    type Error = tree::Error;
    #[inline(always)]
    fn get_many_mut<const N: usize>(&mut self, keys: [&'b K; N]) -> Result<[Self::ItemMut<'_>; N], Self::Error> {
        self.get_many_mut(keys)
    }
}
impl<'a, K: Ord, V> const Keyed for TreeWriteGuard<'a, K, V> {
    type Key = K;
}
impl<'a, K: Ord, V> const KeyedRef for TreeWriteGuard<'a, K, V> {
    type KeyRef<'b> = &'b K where Self: 'b;
    covariant_key_ref!();
}
impl<'a, 'b, K: Ord, V> GetKeyValue<&'b K> for TreeWriteGuard<'a, K, V> {
    #[inline]
    fn get_key_value(&self, key: &K) -> Option<(Self::KeyRef<'_>, Self::ItemRef<'_>)> {
        match Tree::search(self.tree.root, key, &self.nodes) {
            SearchResult::Here(ptr) => {
                let node = &self.nodes[ptr];
                Some((&node.key, &node.value))
            },
            _ => None
        }
    }
}
impl<'a, 'b, K: Ord, V> GetKeyValueMut<&'b K> for TreeWriteGuard<'a, K, V> {
    #[inline]
    fn get_key_value_mut(&mut self, key: &'b K) -> Option<(Self::KeyRef<'_>, Self::ItemMut<'_>)> {
        match Tree::search(self.tree.root, key, &self.nodes) {
            SearchResult::Here(ptr) => {
                let node = &mut self.nodes[ptr];
                Some((&node.key, &mut node.value))
            },
            _ => None
        }
    }
}
impl<'a, 'b, K: Ord, V> Index<&'b K> for TreeWriteGuard<'a, K, V> {
    type Output = V;
    #[inline]
    fn index(&self, key: &K) -> &Self::Output {
        match self.get(key) {
            Some(value) => value,
            None => panic!("invalid key")
        }
    }
}
impl<'a, 'b, K: Ord, V> IndexMut<&'b K> for TreeWriteGuard<'a, K, V> {
    #[inline]
    fn index_mut(&mut self, key: &'b K) -> &mut Self::Output {
        match self.get_mut(key) {
            Some(value) => value,
            None => panic!("invalid key")
        }
    }
}