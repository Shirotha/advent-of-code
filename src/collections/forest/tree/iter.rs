pub use sorted_iter::sorted_pair_iterator::SortedByKey;

use super::*;

// TODO: double ended, sorted iterator
struct Iter<'a, K: Ord, V> {
    port: PortReadGuard<'a, Node<K, V>>,
    front: NodeRef,
    back: NodeRef,
    remaining: usize
}
impl<'a, K: Ord, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 { return None; }
        // SAFETY: iterator is not empty
        let current = self.front.unwrap();
        let node = self.port.get(current).unwrap();
        self.front = node.order[1];
        Some((&node.key, &node.value))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<'a, K: Ord, V> ExactSizeIterator for Iter<'a, K, V> {}
impl<'a, K: Ord, V> DoubleEndedIterator for Iter<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 { return None; }
        // SAFETY: iterator is not empty
        let current = self.back.unwrap();
        let node = self.port.get(current).unwrap();
        self.back = node.order[1];
        Some((&node.key, &node.value))
    }
}
impl<'a, K: Ord, V> SortedByKey for Iter<'a, K, V> {}

struct IterMut<'a, K: Ord, V> {
    port: PortWriteGuard<'a, Node<K, V>>,
    front: NodeRef,
    back: NodeRef,
    remaining: usize
}
impl<'a, K: Ord, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 { return None; }
        // SAFETY: iterator is not empty
        let current = self.front.unwrap();
        let node = self.port.get_mut(current).unwrap();
        self.front = node.order[1];
        Some((&node.key, &mut node.value))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.port.len()))
    }
}
impl<'a, K: Ord, V> ExactSizeIterator for IterMut<'a, K, V> {}
impl<'a, K: Ord, V> DoubleEndedIterator for IterMut<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 { return None; }
        // SAFETY: iterator is not empty
        let current = self.back.unwrap();
        let node = self.port.get_mut(current).unwrap();
        self.back = node.order[1];
        Some((&node.key, &mut node.value))
    }
}
impl<'a, K: Ord, V> SortedByKey for IterMut<'a, K, V> {}
