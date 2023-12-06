mod interval;

use std::{
    cmp::Ordering,
    ptr::read,
    mem::take,
    collections::VecDeque
};
use interval::*;
use tap::{Tap, Pipe};

#[derive(Debug)]
enum Node<D, T> {
    Branch {
        range: Interval<D>,
        left: usize,
        right: usize
    },
    Leaf {
        range: Interval<D>,
        value: T,
        prev: Option<usize>,
        next: Option<usize>
    }
}
impl<D, T> Node<D, T> {
    #[inline]
    const fn range(&self) -> &Interval<D> {
        match self {
            Self::Branch { range, .. } => range,
            Self::Leaf { range, .. } => range,
        }
    }
    #[inline]
    const fn prev(&self) -> Option<&Option<usize>> {
        match self {
            Self::Leaf { prev, .. } => Some(prev),
            _ => None
        }
    }
    #[inline]
    fn prev_mut(&mut self) -> Option<&mut Option<usize>> {
        match self {
            Self::Leaf { prev, .. } => Some(prev),
            _ => None
        }
    }
    #[inline]
    const fn next(&self) -> Option<&Option<usize>> {
        match self {
            Self::Leaf { next, .. } => Some(next),
            _ => None
        }
    }
    #[inline]
    fn next_mut(&mut self) -> Option<&mut Option<usize>> {
        match self {
            Self::Leaf { next, .. } => Some(next),
            _ => None
        }
    }
}
impl<D: Ord, T> Node<D, T> {
    #[inline]
    fn contains(&self, value: &D) -> bool {
        self.range().contains(value)
    }
}
impl<D: Clone, T> Node<D, T> {
    #[inline]
    fn combine(&self, right: &Self) -> Interval<D> {
        // ASSERT self.range().min <= right.range().max
        Interval::new(self.range().min().clone(), right.range().max().clone())
    }
}
impl<D: Default, T: Default> Default for Node<D, T> {
    #[inline]
    fn default() -> Self {
        Self::Leaf { range: Interval::default(), value: T::default(), prev: None, next: None }
    }
}
/*
 * interface
 * 
 *   get value at position (if exists)
 *   => traverse using binary search
 * 
 *   iterate all segments in order
 *   => have to store all leaves in order
 * 
 *   range entry
 *     modify with f(existing) -> new callback
 *     iterate over all segments in range
 * 
 */
#[derive(Debug)]
pub struct RangeMap<D, T> {
    nodes: Vec<Node<D, T>>,
    first: usize,
    last: usize,
    queue: VecDeque<usize>
}
impl<D, T> RangeMap<D, T> {
    #[inline]
    fn iter(&self) -> Iter<D, T> {
        Iter { nodes: &self.nodes, next: Some(self.first) }
    }
}
impl<D: Default, T: Default> RangeMap<D, T> {
    #[inline]
    fn new() -> Self {
        Self { nodes: vec![Node::default()], queue: VecDeque::new(), first: 0, last: 0 }
    }
}
impl<D: Clone + Default, T: Default> RangeMap<D, T> {
    fn from_iter<I: ExactSizeIterator<Item = (Interval<D>, T)>>(leaves: I) -> Self {
        Self::new()
            .tap_mut( |this| this.replace_with_subtree(0, leaves) )
    }
}
impl<D: Clone, T> RangeMap<D, T> {
    #[inline(always)]
    fn push_branch(&mut self, left: usize, right: usize) {
        let range = unsafe {
            self.nodes.get_unchecked(left)
            .combine(self.nodes.get_unchecked(right))
        };
        self.nodes.push(Node::Branch { range, left, right })
    }
    fn replace_with_subtree<I: ExactSizeIterator<Item = (Interval<D>, T)>>(&mut self, root: usize, leaves: I) {
        // ASSERT: leaves have to be sorted and not overlap root.prev or root.next
        // NOTE: ranges of ancestors have to be updated manually
        // ASSERT: root has to be a Leaf
        #[derive(Debug)]
        enum Carry {
            Left(usize),
            Right(usize),
            None
        }
        impl Carry {
            #[inline]
            fn is_some(&self) -> bool {
                !matches!(self, Self::None)
            }
        }
        let (prev, next) = if let Node::Leaf { prev, next, .. } = self.nodes[root] {
            (prev, next)
        } else { panic!("root has to be a leaf"); };
        let (mut current, mut n) = (self.nodes.len(), leaves.len());
        self.nodes.reserve((n << 1) - 2);
        leaves.enumerate().for_each( |(i, (range, shift))| 
            self.nodes.push(Node::Leaf { range, value: shift, prev: Some(current + i - 1), next: Some(current + i + 1) })
        );
        unsafe {
            if let Some(prev) = prev {
                *self.nodes[prev].next_mut().unwrap_unchecked() = Some(current);
            }
            if let Some(next) = next {
                *self.nodes[next].prev_mut().unwrap_unchecked() = Some(self.nodes.len() - 1);
            }
            *self.nodes[current].prev_mut().unwrap_unchecked() = prev;
            *self.nodes.last_mut().unwrap_unchecked().prev_mut().unwrap_unchecked() = next;
        }
        let mut carry = Carry::None;
        while n > 2 || (n == 2 && carry.is_some()) {
            let end = current + n;
            let right = match carry {
                Carry::Left(left) => {
                    self.push_branch(left, current);
                    current += 1;
                    carry = Carry::None;
                    None
                }
                Carry::Right(right) if n & 1 == 0 => {
                    carry = Carry::Left(current);
                    current += 1;
                    Some(right)
                },
                _ => None
            };
            while current < end - 1 {
                self.push_branch(current, current + 1);
                current += 2;
            }
            if let Some(right) = right {
                self.push_branch(current, right);
            } else if current != end {
                carry = Carry::Right(current);
            }
            current = end; n >>= 1;
        }
        let (left, right) = match carry {
            Carry::Left(left) => (left, current),
            Carry::Right(right) => (current, right),
            Carry::None => (current, current + 1)
        };
        let range = unsafe {
            self.nodes.get_unchecked(left)
                .combine(self.nodes.get_unchecked(right))
            };
        self.nodes[root] = Node::Branch { range, left, right };
    }
}

pub struct Iter<'a, D, T> {
    nodes: &'a [Node<D, T>],
    next: Option<usize>,
}
impl<'a, D, T> Iterator for Iter<'a, D, T> {
    type Item = (&'a Interval<D>, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map( |current| match &self.nodes[current] {
            Node::Leaf { range, value, next, .. } => {
                self.next = *next;
                (range, value)
            },
            _ => panic!("bad pointer!")
        } )
    }
}
impl<'a, D, T> IntoIterator for &'a RangeMap<D, T> {
    type IntoIter = Iter<'a, D, T>;
    type Item = <Self::IntoIter as Iterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        Iter { nodes: &self.nodes, next: Some(self.first) }
    }
}

pub struct IntoIter<D, T> {
    nodes: Vec<Node<D, T>>,
    next: Option<usize>
}
impl<D: Default, T: Default> Iterator for IntoIter<D, T> {
    type Item = (Interval<D>, T);
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map( |current| match take(&mut self.nodes[current]) {
            Node::Leaf { range, value, next, .. } => {
                self.next = next;
                (range, value)
            },
            _ => panic!("bad pointer!")
        } )
    }
}
impl<D: Default, T: Default> IntoIterator for RangeMap<D, T> {
    type IntoIter = IntoIter<D, T>;
    type Item = <Self::IntoIter as Iterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { nodes: self.nodes, next: Some(self.first) }
    }
}