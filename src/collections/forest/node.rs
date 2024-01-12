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
    #[inline]
    pub(super) const fn new(key: K, value: V, parent: NodeIndex, prev: NodeRef, next: NodeRef) -> Self {
        Self {
            key, value,
            color: Color::Red,
            parent: Some(parent),
            left: None, right: None,
            prev, next,
        }
    }
    #[inline]
    pub(super) const fn root(key: K, value: V) -> Self {
        Self {
            key, value,
            color: Color::Black,
            parent: None,
            left: None, right: None,
            prev: None, next: None
        }
    }
    #[inline(always)]
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