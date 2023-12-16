use std::{
    borrow::Cow,
    mem::transmute
};
use bit_vec::BitVec;
use itertools::Itertools;
use ndarray::prelude::*;
use nom::IResult;
use tap::Pipe as TapPipe;

use crate::{*, parse::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Dir {
    E = 0x01,
    N = 0x04,
    W = 0x10,
    S = 0x40
}
impl Dir {
    #[inline]
    const fn rotate_left(&self, n: u32) -> Self {
        unsafe { transmute((*self as u8).rotate_left(n << 1)) }
    }
    #[inline]
    const fn reverse(&self) -> Self {
        self.rotate_left(2)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Pipe {
    NS     = 0x44,
    EW     = 0x11,
    NE     = 0x05,
    NW     = 0x14,
    SW     = 0x50,
    SE     = 0x41,
    Ground = 0x00,
    Start  = 0xaa,
}
impl Pipe {
    #[inline]
    const fn from_u8(chr: u8) -> Self {
        match chr {
            b'|' => Self::NS,
            b'-' => Self::EW,
            b'L' => Self::NE,
            b'J' => Self::NW,
            b'7' => Self::SW,
            b'F' => Self::SE,
            b'.' => Self::Ground,
            b'S' => Self::Start,
            _ => panic!()
        }
    }
    #[inline]
    const fn dirs(&self) -> Option<(Dir, Dir)> {
        let dirs = (*self as u8) & 0x55;
        if dirs == 0 { return None; }
        unsafe { Some((
            transmute(1u8 << dirs.trailing_zeros()),
            transmute(0x80u8 >> dirs.leading_zeros())
        )) }
    }
    #[inline]
    const fn other_dir(&self, dir: Dir) -> Dir {
        let dir = dir as u8;
        let dirs = (*self as u8) & 0x55;
        if dirs & dir == 0 { panic!(); }
        unsafe { transmute(dirs ^ dir) }
    }
    #[inline]
    const fn has_dir(&self, dir: Dir) -> bool {
        (*self as u8) & (dir as u8) != 0
    }
}

#[inline]
const fn move_towards(mut pos: (usize, usize), dir: Dir) -> (usize, usize) {
    match dir {
        Dir::E => pos.1 += 1,
        Dir::N => pos.0 -= 1,
        Dir::W => pos.1 -= 1,
        Dir::S => pos.0 += 1
    }
    pos
}

type Move = ((usize, usize), Dir);

#[derive(Debug)]
struct Walker<'a> {
    data: &'a Array2<Pipe>,
    pos: (usize, usize),
    last: Dir
}
impl<'a> Walker<'a> {
    #[inline]
    fn new(data: &'a Array2<Pipe>, pos: (usize, usize), towards: Dir) -> Self {
        Walker { data, pos, last: data[pos].other_dir(towards).reverse() }
    }
}
impl Iterator for Walker<'_> {
    type Item = Move;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.data[self.pos].other_dir(self.last.reverse());
        self.last = next;
        let pos = move_towards(self.pos, next);
        self.pos = pos;
        
        Some((pos, next))
    }
}
#[inline]
fn has_crossed(a: &Move, b: &Move) -> bool {
    a.0 == b.0 || a.0 == move_towards(b.0, a.1)
}

fn grid(input: &str) -> IResult<&str, (Array2<Pipe>, (usize, usize))> {
    let mut i = 0;
    let mut width = None;
    let mut current = 0;
    let mut height = 0;
    let mut start = None;
    let bytes = input.as_bytes();
    let mut buffer = Vec::new();
    while let Some(&chr) = bytes.get(i) {
        if chr == b'\n' {
            if let Some(w) = width {
                if w != current { break; }
            } else {
                width = Some(current);
            }
            current = 0;
            height += 1;
        } else if chr != b'\r' {
            let pipe = Pipe::from_u8(chr);
            buffer.push(pipe);
            if pipe == Pipe::Start {
                start = Some((height, current));
            }
            current += 1;
        }
        i += 1;
    }
    let width = width.unwrap_or(current);
    if width == current {
        height += 1;
    }
    let len = width * height;
    buffer.resize(len, Pipe::Ground);
    let grid = Array2::from_shape_vec((height, width), buffer).unwrap();
    Ok((&input[len..], (grid, start.unwrap())))
}

#[inline]
fn can_move(data: &Array2<Pipe>, from: (usize, usize), dir: Dir) -> bool {
    let (h, w) = data.dim();
    match dir {
        Dir::E => from.1 != w - 1,
        Dir::N => from.0 != 0,
        Dir::W => from.1 != 0,
        Dir::S => from.0 != h - 1
    }
}

#[inline]
fn patch(data: &mut Array2<Pipe>, at: (usize, usize)) -> Pipe {
    let side = |dir|
        if can_move(data, at, dir)
            && data[move_towards(at, dir)].has_dir(dir.reverse()) 
        { dir as u8 } else { 0 };
    let dirs = side(Dir::E) | side(Dir::N) | side(Dir::W) | side(Dir::S);
    if dirs.count_ones() != 2 { panic!(); }
    let pipe = unsafe { transmute(dirs) };
    data[at] = pipe;
    pipe
}

pub fn part1(input: &str) -> Answer {
    parse(input, grid)?
        .pipe( |(mut grid, start)| {
            let (a, b) = patch(&mut grid, start).dirs().unwrap();
            Walker::new(&grid, start, a)
                .zip(Walker::new(&grid, start, b))
                .take_while( |(a, b)| !has_crossed(a, b) )
                .count() + 1
        } )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    parse(input, grid)?
        .pipe( |(mut grid, start)| {
            let (height, width) = grid.dim();
            let (dir, _) = patch(&mut grid, start).dirs().unwrap();
            let mut path = BitVec::from_elem(width * height, false);
            for ((row, column), _) in Walker::new(&grid, start, dir)
                .take_while_inclusive( |(pos, _)| *pos != start )
            {
                path.set(row * width + column, true);
            }
            let mut sum = 0;
            let mut is_inside = false;
            for (i, pipe) in grid.iter().enumerate() {
                if path[i] {
                    if matches!(pipe, Pipe::NS | Pipe::NW | Pipe::NE) {
                        is_inside = !is_inside;
                    }
                } else if is_inside {
                    sum += 1;
                }  
            }
            sum
        } )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

inventory::submit! { Puzzle::new(2023, 10, 1, part1) }
inventory::submit! { Puzzle::new(2023, 10, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        ..F7.
        .FJ|.
        SJ.L7
        |F--J
        LJ...
    "};
    const OUTPUT1: &str = "8";

    const INPUT2: &str = indoc! {"
        FF7FSF7F7F7F7F7F---7
        L|LJ||||||||||||F--J
        FL-7LJLJ||||||LJL-77
        F--JF--7||LJLJ7F7FJ-
        L---JF-JLJ.||-FJLJJ7
        |F|F-JF---7F7-L7L|7|
        |FFJF7L7F-JF7|JL---7
        7-L-JL7||F7|L7F-7F7|
        L.L7LFJ|||||FJL7||LJ
        L7JLJL-JLJLJL--JLJ.L
    "};
    const OUTPUT2: &str = "10";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}