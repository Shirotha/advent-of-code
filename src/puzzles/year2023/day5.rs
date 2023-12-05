use std::{
    borrow::Cow,
    ops::{Add, AddAssign}, 
    cmp::Ordering,
    collections::VecDeque,
    ptr::{read, write, NonNull}, 
    alloc::{alloc, Layout, Global, Allocator}
};
use tap::{Tap, Pipe};

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

type Ref<T, S> = NonNull<Node<T, S>>;
type MaybeRef<T, S> = Option<Ref<T, S>>;
enum Node<T, S> {
    Branch {
        range: Interval<T>,
        left: Ref<T, S>,
        right: Ref<T, S>
    },
    Leaf {
        range: Interval<T>,
        shift: S
    }
}
impl<T, S> Node<T, S> {
    const LAYOUT: Layout = Layout::new::<Node<T, S>>();
    #[inline]
    unsafe fn alloc(value: Node<T, S>) -> Ref<T, S> {
        Global.allocate(Self::LAYOUT)
            .expect("valid pointer")
            .pipe(NonNull::cast)
            .tap_mut( |ptr| ptr.write(value) )
    }
    #[inline]
    unsafe fn alloc2() -> (Ref<T, S>, Ref<T, S>) {
        let (layout, offset) = Self::LAYOUT.repeat(2)
            .expect("valid layout");
        Global.allocate(layout)
            .expect("valid pointer")
            .pipe( |raw| (raw.cast(), raw.byte_add(offset).cast()) )
    }
    #[inline]
    unsafe fn alloc4() -> (Ref<T, S>, Ref<T, S>, Ref<T, S>, Ref<T, S>) {
        let (layout, offset) = Self::LAYOUT.repeat(4)
            .expect("valid layout");
        Global.allocate(layout)
            .expect("valid pointer")
            .pipe( |raw| (
                raw.cast(),
                raw.byte_add(offset).cast(),
                raw.byte_add(offset << 1).cast(),
                raw.byte_add(offset + (offset << 1)).cast()
            ) )
    }
}
impl<T: Copy, S> Node<T, S> {
    unsafe fn insert_left(left_range: Interval<T>, left_shift: S, mut right_node: Ref<T, S>) {
        if let Node::Leaf { range, shift } = read(right_node.as_ptr()) {
            let total = Interval::new(left_range.min, range.max);
            let (mut left, mut right) = Node::alloc2();
            write(left.as_mut(), Node::Leaf { range: left_range, shift: left_shift });
            write(right.as_mut(), Node::Leaf { range, shift });
            write(right_node.as_mut(), Node::Branch { range: total, left, right });
        } else { panic!("bad pointer!"); }
    }
    unsafe fn insert_right(mut left_node: Ref<T, S>, right_range: Interval<T>, right_shift: S) {
        if let Node::Leaf { range, shift } = read(left_node.as_ptr()) {
            let total = Interval::new(range.min, right_range.max);
            let (mut left, mut right) = Node::alloc2();
            write(left.as_mut(), Node::Leaf { range, shift });
            write(right.as_mut(), Node::Leaf { range: right_range, shift: right_shift });
            write(left_node.as_mut(), Node::Branch { range: total, left, right });
        } else { panic!("bad pointer!"); }
    }
    unsafe fn build(
        mut root: Ref<T, S>,
        left_range: Interval<T>, left_shift: S,
        middle_range: Interval<T>, middle_shift: S,
        right_range: Interval<T>, right_shift: S
    ) {
        if let Node::Leaf { range, shift } = read(root.as_ptr()) {
            // TODO: skip when left/right is empty (also skip help when one was skipped)
            let (mut help, mut left, mut middle, mut right) = Self::alloc4();
            let total = Interval::new(left_range.min, right_range.max);
            let inter = Interval::new(left_range.min, middle_range.max);
            write(left.as_mut(), Node::Leaf { range: left_range, shift: left_shift });
            write(middle.as_mut(), Node::Leaf { range: middle_range, shift: middle_shift });
            write(right.as_mut(), Node::Leaf { range: right_range, shift: right_shift });
            write(help.as_mut(), Node::Branch { range: inter, left, right: middle });
            write(root.as_mut(), Node::Branch { range: total, left: help, right });
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
struct Map<T, S> {
    root: MaybeRef<T, S>,
    queue: VecDeque<Ref<T, S>>
}
impl<T: Copy + Ord, S: Copy + AddAssign> Map<T, S> {
    #[inline]
    const fn new() -> Self {
        Self { root: None, queue: VecDeque::new() }
    }
    fn insert(&mut self, location: Interval<T>, value: S) {
        if let Some(root) = self.root {
            self.queue.push_back(root);
        } else {
            self.root = unsafe { Some(Node::alloc(Node::Leaf { range: location, shift: value })) };
            return;
        }
        while let Some(mut current) = self.queue.pop_front() {
            match unsafe { current.as_mut() } {
                Node::Branch { range, left, right } => {
                    if location.overlaps(range) {
                        self.queue.push_back(*left);
                        self.queue.push_back(*right);
                    }
                },
                Node::Leaf { range, shift } => unsafe {
                    match location.partial_cmp(range) {
                        Some(Ordering::Less) => Node::insert_left(location, value, current),
                        Some(Ordering::Equal) => *shift += value,
                        Some(Ordering::Greater) => Node::insert_right(current, location, value),
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
                    return;
                }
            }
        }
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
                        let node = unsafe { left.as_ref() };
                        if node.contains(&value) { current = *left; continue; }
                        let node = unsafe { right.as_ref() };
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
// TODO: clean up all nodes when Map drops

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