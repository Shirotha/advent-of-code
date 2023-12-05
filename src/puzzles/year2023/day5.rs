use std::{
    borrow::Cow,
    ops::Add, 
    cmp::Ordering,
    collections::VecDeque,
    ptr::NonNull, 
    alloc::{alloc, Layout}
};
use tap::Tap;

use crate::{*, parse::*};

#[derive(Debug)]
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

struct Shift<T, S> {
    range: Interval<T>,
    shift: S
}
impl<T, S> Shift<T, S> {
    #[inline]
    const fn new(range: Interval<T>, shift: S) -> Self {
        Shift { range, shift }
    }
}
impl<T: Copy + Ord + Add<S, Output = T>, S: Copy> Shift<T, S> {
    #[inline]
    fn apply(&self, value: T) -> T {
        if self.range.contains(&value) { value.add(self.shift) } else { value }
    }
}
type Ref<T, S> = NonNull<Node<T, S>>;
type MaybeRef<T, S> = Option<Ref<T, S>>;
enum Node<T, S> {
    Branch {
        range: Interval<T>,
        left: MaybeRef<T, S>,
        right: MaybeRef<T, S>
    },
    Leaf(Shift<T, S>)
}
impl<T, S> Node<T, S> {
    const LAYOUT: Layout = Layout::new::<Node<T, S>>();
    unsafe fn alloc(value: Node<T, S>) -> Ref<T, S> {
        let raw = alloc(Self::LAYOUT);
        NonNull::new(raw as *mut Node<T, S>)
            .expect("valid pointer")
            .tap_mut( |ptr| *ptr.as_mut() = value)
    }
}
impl<T: Ord, S> Node<T, S> {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        match self {
            Node::Branch { range , ..} => range.contains(value),
            Node::Leaf(shift) => shift.range.contains(value)
        }
    }
}
struct Map<T, S> {
    root: MaybeRef<T, S>,
    queue: VecDeque<Ref<T, S>>
}
impl<T: Ord, S> Map<T, S> {
    #[inline]
    const fn new() -> Self {
        Self { root: None, queue: VecDeque::new() }
    }
    fn insert(&mut self, value: Shift<T, S>) {
        self.queue.clear(); // TODO: is this nessesary?
        if let Some(root) = self.root {
            self.queue.push_back(root);
        } else {
            self.root = unsafe { Some(Node::alloc(Node::Leaf(value))) };
            return;
        }
        // traverse tree to find all overlapping leafs
        while let Some(current) = self.queue.pop_front() {
            match unsafe { current.as_ref() } {
                Node::Branch { range, left, right } => {
                    if value.range.overlaps(range) {
                        if let Some(left) = left {
                            self.queue.push_back(*left);
                        }
                        if let Some(right) = right {
                            self.queue.push_back(*right);
                        }
                    }
                },
                Node::Leaf(shift) => {
                    match value.range.partial_cmp(&shift.range) {
                        Some(Ordering::Less) => {
                            // replace current with Branch
                            // put old current on the right and value on the left
                        },
                        Some(Ordering::Equal) => {
                            // modify shift value of current
                        },
                        Some(Ordering::Greater) => {
                            // replace current with Branch
                            // put old current on the left and value on the right
                        },
                        None => {
                            // split current
                        }
                    }
                }
            }
        }
        // for each leaf, split and/or update shifts
        todo!("insert shift operation into map")
    }
}
impl<T: Copy + Ord + Add<S, Output = T>, S: Copy> Map<T, S> {
    fn apply(&self, value: T) -> T {
        let mut current = 
            if let Some(root) = self.root { root }
            else { return value; };
        loop {
            match unsafe { current.as_ref() } {
                Node::Branch { range, left, right } => {
                    if range.contains(&value) {
                        if let Some(left) = left {
                            let node = unsafe { left.as_ref() };
                            if node.contains(&value) { current = *left; continue; }
                        }
                        if let Some(right) = right {
                            let node = unsafe { right.as_ref() };
                            if node.contains(&value) { current = *right; continue; }
                        }
                        return value;
                    } else { break; }
                },
                Node::Leaf(shift) => return shift.apply(value)
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