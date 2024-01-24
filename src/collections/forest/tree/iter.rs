pub use sorted_iter::sorted_pair_iterator::SortedByKey;

use super::*;
/* FIXME: lifetime issue
struct Iter<'a, K: Ord, V> {
    port: PortReadGuard<'a, Node<K, V>>,
    front: NodeRef,
    back: NodeRef
}
impl<'a, K: Ord, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.front?;
        let node = self.port.get(current).unwrap();
        if self.front == self.back {
            self.front = None;
            self.back = None;
        } else {
            self.front = node.order[1];
        }
        Some((&node.key, &node.value))
    }
}
impl<'a, K: Ord, V> DoubleEndedIterator for Iter<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let current = self.back?;
        let node = self.port.get(current).unwrap();
        if self.front == self.back {
            self.front = None;
            self.back = None;
        } else {
            self.back = node.order[0];
        }
        Some((&node.key, &node.value))
    }
}
impl<'a, K: Ord, V> SortedByKey for Iter<'a, K, V> {}

struct IterMut<'a, K: Ord, V> {
    port: PortWriteGuard<'a, Node<K, V>>,
    front: NodeRef,
    back: NodeRef
}
impl<'a, K: Ord, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.front?;
        let node = self.port.get_mut(current).unwrap();
        if self.front == self.back {
            self.front = None;
            self.back = None;
        } else {
            self.front = node.order[1];
        }
        Some((&node.key, &mut node.value))
    }
}
impl<'a, K: Ord, V> DoubleEndedIterator for IterMut<'a, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let current = self.back?;
        let node = self.port.get_mut(current).unwrap();
        if self.front == self.back {
            self.front = None;
            self.back = None;
        } else {
            self.back = node.order[0];
        }
        Some((&node.key, &mut node.value))
    }
}
impl<'a, K: Ord, V> SortedByKey for IterMut<'a, K, V> {}
 */