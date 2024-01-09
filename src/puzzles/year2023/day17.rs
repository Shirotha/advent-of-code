use std::{
    borrow::Cow,
    ops::Add,
    hash::Hash,
    collections::HashSet,
    cmp::Reverse, mem::transmute
};
use keyed_priority_queue::{KeyedPriorityQueue, Entry};
use num_traits::Zero;

use crate::{*, parse::*};

fn astar<T, X, S, N, NR, H, G>(start: S, mut neighbours: N, mut heuristic: H, mut goal: G)
    -> Option<(X, Vec<T>)>
where
    T: Copy + Eq + Hash,
    X: Copy + Ord + Add<X, Output = X> + Zero,
    S: IntoIterator<Item = T>,
    N: FnMut(T) -> NR,
    NR: IntoIterator<Item = (T, X)>,
    H: FnMut(T) -> X,
    G: FnMut(T) -> bool
{
    let mut front = KeyedPriorityQueue::new();
    let mut closed = HashSet::new();
    let mut parent = HashMap::new();
    for t in start {
        front.push(t, Reverse((heuristic(t), X::zero())));
    }
    while let Some((mut t, Reverse((_, g)))) = front.pop() {
        if goal(t) {
            let mut path = Vec::new();
            path.push(t);
            while let Some(p) = parent.get(&t) {
                t = *p;
                path.push(t);
            }
            path.reverse();
            return Some((g, path))
        }
        closed.insert(t);
        for (n, c) in neighbours(t).into_iter()
            .filter( |(n, _)| !closed.contains(n) )
        {
            let g = g + c;
            let p = Reverse((g + heuristic(n), g));
            match front.entry(n) {
                Entry::Vacant(e) => {
                    e.set_priority(p);
                    parent.insert(n, t);
                }
                Entry::Occupied(e) if *e.get_priority() < p => {
                    _ = e.set_priority(p);
                    parent.insert(n, t);
                }
                _ => ()
            }
        }
    }
    None
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
enum Dir {
    #[default]
    Unknown = 0x55,
    E = 0x0f,
    N = 0x00,
    W = 0xf0,
    S = 0xff
}
impl Dir {
    #[inline(always)]
    fn invert(self) -> Self {
        unsafe {
            transmute(!(self as u8))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidResult {
    Invalid,
    First,
    Straight,
    Turn
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct History<const N: u8>(Dir, u8);
impl<const N: u8> History<N> {
    #[inline]
    fn valid(&self, dir: Dir) -> ValidResult {
        if self.0 == Dir::Unknown {
            return ValidResult::First;
        }
        if self.0 == dir {
            return ValidResult::Straight;
        }
        if self.0 != dir.invert() {
            return ValidResult::Turn;
        }
        ValidResult::Invalid
    }
    #[inline]
    fn concat(self, rhs: Dir) -> Option<Self> {
        match self.valid(rhs) {
            ValidResult::Straight =>
                if self.1 != N {
                    Some(Self(self.0, self.1 + 1))
                } else { None },
                ValidResult::First | ValidResult::Turn => Some(Self(rhs, 1)),
                ValidResult::Invalid => None
        }
    }
}

type Node = ([usize; 2], History<3>);

pub fn part1(input: &str) -> Answer {
    let grid = parse(input, grid(&mut |c| c as u8 - b'0' ))?;
    let (w, h) = grid.dim();
    let node = #[inline]
        |(i, h): Node, dir: Dir|
            h.concat(dir).and_then( |h| 
                grid.get(i).map( |x| ((i, h), *x as u16) )
            );
    let neighbours = #[inline]
        |([x, y], h): Node|
            node(([x.wrapping_sub(1), y], h), Dir::W).into_iter()
                .chain(node(([x, y.wrapping_sub(1)], h), Dir::N))
                .chain(node(([x + 1, y], h), Dir::E))
                .chain(node(([x, y + 1], h), Dir::S));
    let heuristic = #[inline]
        |([x, y], _): Node|
            ((w - x - 1) + (h - y - 1)) as u16;
    let goal = #[inline]
        |(i, _): Node|
            i == [w - 1, h - 1];
    let (cost, path) = astar([([0, 0], History::default())], neighbours, heuristic, goal)
        .expect("valid path");
    for (_, h) in path {
        print!("{}", match h.0 {
            Dir::E => "E",
            Dir::N => "N",
            Dir::W => "W",
            Dir::S => "S",
            _ => "?",
        });
    }
    println!();
    Ok(Cow::Owned(cost.to_string()))
}

pub fn part2(input: &str) -> Answer {
    todo!()
}

inventory::submit! { Puzzle::new(2023, 17, 1, part1) }
inventory::submit! { Puzzle::new(2023, 17, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        2413432311323
        3215453535623
        3255245654254
        3446585845452
        4546657867536
        1438598798454
        4457876987766
        3637877979653
        4654967986887
        4564679986453
        1224686865563
        2546548887735
        4322674655533
    "};
    const OUTPUT1: &str = "102";

    const INPUT2: &str = indoc! {"
    
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