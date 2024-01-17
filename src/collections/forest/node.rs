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
    pub(super) children: [NodeRef; 2],
    pub(super) order: [NodeRef; 2]
}

impl<K, V> Node<K, V> {
    #[inline]
    pub(super) const fn new(key: K, value: V, parent: NodeIndex, order: [NodeRef; 2]) -> Self {
        Self {
            key, value,
            color: Color::Red,
            parent: Some(parent),
            children: [None, None],
            order
        }
    }
    #[inline]
    pub(super) const fn root(key: K, value: V) -> Self {
        Self {
            key, value,
            color: Color::Black,
            parent: None,
            children: [None, None],
            order: [None, None]
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