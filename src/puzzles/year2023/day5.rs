use std::{
    borrow::Cow,
    ops::{Add, AddAssign}, 
    cmp::Ordering,
    collections::VecDeque,
};
use tap::{Tap, Pipe};

use crate::{*, parse::*};

#[derive(Debug, Clone, Copy)]
struct Interval<T> {
    min: T,
    max: T
}
impl<T> Interval<T> {
    #[inline]
    const fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}
impl<T: Ord> Interval<T> {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        &self.min <= value && value <= &self.max
    }
    #[inline]
    fn overlaps(&self, other: &Interval<T>) -> bool {
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
impl <T, S: AddAssign> Node<T, S> {
    #[inline]
    fn shift(&mut self, value: S) {
        match self {
            Self::Leaf { shift, .. } => *shift += value,
            _ => ()
        }
    }
}
impl<T: Ord, S> Node<T, S> {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        match self {
            Node::Branch { range , .. } => range.contains(value),
            Node::Leaf { range, .. } => range.contains(value)
        }
    }
}

#[derive(Debug)]
struct Map<T, S> {
    nodes: Vec<Node<T, S>>,
    queue: VecDeque<usize>
}
impl <T: Default, S: Default> Map<T, S> {
    fn from_iter<I: ExactSizeIterator<Item = (Interval<T>, S)>>(leaves: I) -> Self {
        let mut this = Self { nodes: vec![Node::Leaf { range: Interval::new(T::default(), T::default()), shift: S::default() }], queue: VecDeque::new() };
        this.build_from_leaves(0, leaves);
        this
    }
}
impl<T: Copy + Ord, S: Copy + AddAssign> Map<T, S> {
    #[inline]
    const fn new() -> Self {
        Self { nodes: Vec::new(), queue: VecDeque::new() }
    }
    fn build_from_leaves<I: ExactSizeIterator<Item = (Interval<T>, S)>>(&mut self, root: usize, leaves: I) {
        let (first, n) = (self.nodes.len(), leaves.len());
        self.nodes.reserve((n << 1) - 2);
        todo!()
    }
    fn insert(&mut self, location: Interval<T>, value: S) {
        if self.nodes.is_empty() {
            self.nodes.push(Node::Leaf { range: location, shift: value });
            return;
        } else {
            self.queue.push_back(0);
        }
        while let Some(current) = self.queue.pop_front() {
            match unsafe { self.nodes.get_unchecked(current) } {
                Node::Branch { range, left, right } => {
                    if location.overlaps(range) {
                        self.queue.push_back(*left);
                        self.queue.push_back(*right);
                    }
                },
                Node::Leaf { range, shift } => {
                    match location.partial_cmp(range) {
                        Some(Ordering::Less) =>
                            self.build_from_leaves(current, [(location, value), (*range, *shift)].into_iter()),
                        Some(Ordering::Equal) => unsafe { self.nodes.get_unchecked_mut(current).shift(value); },
                        Some(Ordering::Greater) =>
                            self.build_from_leaves(current, [(*range, *shift), (location, value)].into_iter()),
                        None => {
                            if location.min < range.min {
                                if location.max > range.max {
                                    // overlap center (outer)
                                    //   split into left (value), middle (value + shift), right (value)
                                } else {
                                    // overlap left
                                    //   split into left (value), middle (value + shift), right (shift)
                                }
                            } else if location.max > range.max {
                                // overlap right
                                //   split into left (shift), middle (shift + value), right (value)
                            } else {
                                // overlap center (inner)
                                //   split into left (shift), middle (shift + value), right (shift)
                            }
                            // connect new nodes and root into current
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