use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Color {
    Black,
    Red
}

#[derive(Debug)]
pub(super) struct Node<K, V> {
    pub(super) key: K,
    pub(super) value: V,
    pub(super) color: Color,
    pub(super) parent: NodeRef,
    pub(super) left: NodeRef,
    pub(super) right: NodeRef,
    pub(super) prev: NodeRef,
    pub(super) next: NodeRef
}

impl<K, V> Node<K, V> {
    pub(super) const fn is_root(&self) -> bool {
        self.parent.is_none()
    }
    #[inline(always)]
    pub(super) const fn is_black(&self) -> bool {
        match self.color {
            Color::Black => true,
            Color::Red => false
        }
    }
    #[inline(always)]
    pub(super) const fn is_red(&self) -> bool {
        match self.color {
            Color::Black => false,
            Color::Red => true
        }
    }
}