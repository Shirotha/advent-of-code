use std::{
    borrow::Cow,
    ops::Add, 
    cmp::Ordering,
    ptr::read,
    collections::VecDeque,
};
use tap::{Tap, Pipe};

use crate::{*, parse::*};

#[derive(Debug, Clone, Copy, Default)]
struct Interval<T> {
    min: T,
    max: T
}
impl<T> Interval<T> {
    #[inline]
    const fn new(min: T, max: T) -> Self {
        // ASSERT: min <= max
        Self { min, max }
    }
}
impl<T: Clone> Interval<T> {
    #[inline]
    fn combine(&self, right: &Self) -> Self{
        // ASSERT self.min <= right.max
        Self::new(self.min.clone(), right.max.clone())
    }
}
impl<T: Ord> Interval<T> {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        &self.min <= value && value <= &self.max
    }
    #[inline]
    fn overlaps(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(Ordering::Equal) | None)
    }
}
impl<T: PartialEq> PartialEq for Interval<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.min == other.min && self.max == other.max
    }
}
impl<T: Eq> Eq for Interval<T> {}
impl<T: PartialOrd> PartialOrd for Interval<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.max < other.min { Some(Ordering::Less) }
        else if self.min > other.max { Some(Ordering::Greater) }
        else if self == other { Some(Ordering::Equal) }
        else { None }
    }
}

#[derive(Debug)]
enum Node<T, S> {
    Branch {
        range: Interval<T>,
        left: usize,
        right: usize
    },
    Leaf {
        range: Interval<T>,
        shift: S
    }
}
impl<T, S> Node<T, S> {
    #[inline]
    fn range(&self) -> &Interval<T> {
        match self {
            Self::Branch { range, .. } => range,
            Self::Leaf { range, .. } => range,
        }
    }
}
impl<T: Clone, S> Node<T, S> {
    #[inline]
    fn combine(&self, right: &Self) -> Interval<T> {
        // ASSERT self.range().min <= right.range().max
        self.range().combine(right.range())
    }
}
impl<T: Default, S: Default> Default for Node<T, S> {
    #[inline]
    fn default() -> Self {
        Self::Leaf { range: Interval::default(), shift: S::default() }
    }
}
impl<T: Ord, S> Node<T, S> {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        self.range().contains(value)
    }
}

#[derive(Debug)]
struct Map<T, S> {
    nodes: Vec<Node<T, S>>,
    queue: VecDeque<usize>
}
impl<T, S> Map<T, S> {
    #[inline]
    const fn new() -> Self {
        Self { nodes: Vec::new(), queue: VecDeque::new() }
    }
}
impl <T: Clone + Default, S: Default> Map<T, S> {
    fn from_iter<I: ExactSizeIterator<Item = (Interval<T>, S)>>(leaves: I) -> Self {
        Self { nodes: vec![Node::default()], queue: VecDeque::new() }
            .tap_mut( |this| this.replace_with_subtree(0, leaves) )
    }
}
impl<T: Clone, S> Map<T, S> {
    #[inline(always)]
    fn push_branch(&mut self, left: usize, right: usize) {
        let range = unsafe {
            self.nodes.get_unchecked(left)
            .combine(self.nodes.get_unchecked(right))
        };
        self.nodes.push(Node::Branch { range, left, right })
    }
    fn replace_with_subtree<I: ExactSizeIterator<Item = (Interval<T>, S)>>(&mut self, root: usize, leaves: I) {
        // ASSERT: leaves have to be sorted and contained in nodes[root].range()
        // NOTE: children of root will not be freed
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
        let (mut current, mut n) = (self.nodes.len(), leaves.len());
        self.nodes.reserve((n << 1) - 2);
        leaves.for_each( |(range, shift)| self.nodes.push(Node::Leaf { range, shift }) );
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
impl<T: Clone + Ord, S: Clone + Add<Output = S>> Map<T, S> {
    fn insert(&mut self, location: Interval<T>, value: S) {
        if self.nodes.is_empty() {
            self.nodes.push(Node::Leaf { range: location, shift: value });
            return;
        } else {
            self.queue.push_back(0);
        }
        while let Some(current) = self.queue.pop_front() {
            match unsafe { read(self.nodes.get_unchecked(current)) } {
                Node::Branch { range, left, right } => {
                    if location.overlaps(&range) {
                        self.queue.push_back(left);
                        self.queue.push_back(right);
                    }
                },
                Node::Leaf { range, shift } =>
                    match location.partial_cmp(&range) {
                        Some(Ordering::Less) =>
                            self.replace_with_subtree(current, [
                                (location.clone(), value.clone()),
                                (range, shift)
                            ].into_iter()),
                        Some(Ordering::Equal) => 
                            self.nodes[current] = Node::Leaf { range, shift: shift + value.clone() },
                        Some(Ordering::Greater) =>
                            self.replace_with_subtree(current, [
                                (range, shift), 
                                (location.clone(), value.clone())
                            ].into_iter()),
                        None => {
                            /* new (A)  both {B}  leaf [C]
                             * ( A [     B     }
                             * ( A [  B  )  C  ]
                             *     {  B  )  C  ]
                             *      < mirror >
                             *     [  C  (  B  }
                             *     [  C  (  B  ] A )
                             *     {     B     ] A )
                             * 
                             *     [ C ( B ) C ]
                             * ( A [     B     ] A )
                             */
                            let (left, right) = (
                                location.min.cmp(&range.min),
                                location.max.cmp(&range.max)
                            );
                            if left == right.reverse() {
                                // symmetrical (Equal case is impossible in overlapping case)
                                if matches!(left, Ordering::Less) {
                                    // overlap center (inner)
                                    //   split into left (shift), middle (shift + value), right (shift)
                                } else {
                                    // overlap center (outer)
                                    //   split into left (value), middle (value + shift), right (value)
                                }
                            } else {
                                // anti-symmetrical
                                // TODO: mirror here
                                if matches!(left, Ordering::Equal) {
                                    // inner fit
                                    //   split into left (shift + value), right (shift)
                                } else if matches!(right, Ordering::Equal) {
                                    // outer fit
                                    //   split into left (value), right (value + shift)
                                } else {
                                    // overlap left
                                    //   split into left (value), middle (value + shift), right (shift)
                                }
                            }
                        }
                    }
            }
        }
    }
}
impl<T: Copy + Ord + Add<S, Output = T>, S: Copy> Map<T, S> {
    fn apply(&self, value: T) -> T {
        if self.nodes.is_empty() { return value; }
        let mut current = 0;
        loop {
            match unsafe { self.nodes.get_unchecked(current) } {
                Node::Branch { range, left, right } => {
                    if range.contains(&value) {
                        let node = unsafe { self.nodes.get_unchecked(*left) };
                        if node.contains(&value) { current = *left; continue; }
                        let node = unsafe { self.nodes.get_unchecked(*right) };
                        if node.contains(&value) { current = *right; continue; }                    
                        return value;
                    } else { break; }
                },
                Node::Leaf { shift, .. } => return value.add(*shift)
            }
        }
        value
    }
}

pub fn part1(input: &str) -> Answer {
    todo!()
}

pub fn part2(input: &str) -> Answer {
    todo!()
}

inventory::submit! { Puzzle::new(2023, 5, 1, part1) }
inventory::submit! { Puzzle::new(2023, 5, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        seeds: 79 14 55 13

        seed-to-soil map:
        50 98 2
        52 50 48

        soil-to-fertilizer map:
        0 15 37
        37 52 2
        39 0 15

        fertilizer-to-water map:
        49 53 8
        0 11 42
        42 0 7
        57 7 4

        water-to-light map:
        88 18 7
        18 25 70

        light-to-temperature map:
        45 77 23
        81 45 19
        68 64 13

        temperature-to-humidity map:
        0 69 1
        1 0 69

        humidity-to-location map:
        60 56 37
        56 93 4
    "};
    const OUTPUT1: &str = "";

    const INPUT2: &str = indoc!{"
    
    "};
    const OUTPUT2: &str = "";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}