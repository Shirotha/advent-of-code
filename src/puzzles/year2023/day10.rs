use std::{
    borrow::Cow,
    cmp::{Ordering, minmax},
    collections::{LinkedList, linked_list::CursorMut},
    mem::transmute,
};
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

type Pos = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Orient {
    Right    = 0x04,
    Straight = 0x10,
    Left     = 0x40
}
impl Orient {
    #[inline]
    const fn sign(&self) -> i16 {
        match self {
            Self::Right => -1,
            Self::Straight => 0,
            Self::Left => 1,
        }
    }
    #[inline]
    fn from_corners(corners: &[Pos]) -> Self {
        let (a, b, c) = (&corners[0], &corners[1], &corners[2]);
        let cmp = if a.0 == b.0 {
            match a.1.cmp(&b.1) {
                Ordering::Less => c.0.cmp(&b.0),
                Ordering::Equal => panic!(),
                Ordering::Greater => b.0.cmp(&c.0)
            }
        } else if a.1 == b.1 {
            match a.0.cmp(&b.0) {
                Ordering::Less => c.1.cmp(&b.1),
                Ordering::Equal => panic!(),
                Ordering::Greater => b.1.cmp(&c.1)
            }
        } else {
            panic!()
        };
        match cmp {
            Ordering::Less => Self::Left,
            Ordering::Equal => panic!(),
            Ordering::Greater => Self::Right
        }
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
    #[inline]
    const fn orient(&self, dir: Dir) -> Orient {
        let from = dir.reverse() as u8;
        let dirs = (*self as u8) & 0x55;
        if dirs & from == 0 { panic!(); }
        let to = dirs.rotate_right(from.trailing_zeros()) & 0x15;
        unsafe { transmute(to) }
    }
}

#[inline]
const fn move_towards(mut pos: Pos, dir: Dir) -> Pos {
    match dir {
        Dir::E => pos.1 += 1,
        Dir::N => pos.0 -= 1,
        Dir::W => pos.1 -= 1,
        Dir::S => pos.0 += 1
    }
    pos
}

type Move = (Pos, Dir);

#[derive(Debug)]
struct Walker<'a> {
    data: &'a Array2<Pipe>,
    pos: Pos,
    last: Dir
}
impl<'a> Walker<'a> {
    #[inline]
    fn new(data: &'a Array2<Pipe>, pos: Pos, towards: Dir) -> Self {
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

type Corner = (Pos, Orient);

#[derive(Debug)]
struct Corners<'a> {
    walker: Walker<'a>,
    first: Option<Pos>
}
impl<'a> Corners<'a> {
    #[inline]
    fn new(data: &'a Array2<Pipe>, pos: Pos, towards: Dir) -> Self {
        Corners { walker: Walker::new(data, pos, towards), first: None }
    }
}
impl Iterator for Corners<'_> {
    type Item = Corner;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((pos, dir)) = self.walker.next() {
            let orient = self.walker.data[pos].orient(dir);
            match orient {
                Orient::Right | Orient:: Left => {
                    if let Some(first) = self.first {
                        return if pos == first { None }
                            else { Some((pos, orient)) };
                    } else {
                        self.first = Some(pos);
                        return Some((pos, orient));
                    }
                },
                Orient::Straight => continue
            }
        }
        None
    }
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
fn can_move(data: &Array2<Pipe>, from: Pos, dir: Dir) -> bool {
    let (h, w) = data.dim();
    match dir {
        Dir::E => from.1 != w - 1,
        Dir::N => from.0 != 0,
        Dir::W => from.1 != 0,
        Dir::S => from.0 != h - 1
    }
}

#[inline]
fn patch(data: &mut Array2<Pipe>, at:Pos) -> Pipe {
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

struct Quad {
    topleft: Pos,
    size: Pos
}
impl Quad {
    #[inline]
    fn from_corners(corners: &[Pos; 4]) -> Quad {
        let ([xmin, xmax], [ymin, ymax]) = if corners[0].0 == corners[1].0 {
            (
                minmax(corners[0].0, corners[2].0),
                minmax(corners[0].1, corners[1].1)
            )
        } else if corners[0].1 == corners[1].1 {
            (
                minmax(corners[0].0, corners[1].0),
                minmax(corners[0].1, corners[2].1)
            )
        } else {
            panic!()
        };
        Quad { topleft: (xmin, ymin), size: (xmax - xmin, ymax - ymin) }
    }
}

fn quadrangulate(corners: Vec<(usize, usize)>) -> Vec<Quad> {
    /*
     * replace bends with shortcuts, add removed corners as quads to result
     * 
     * Case A: four corners building a Quad, orientation RLLR  
     * 
     *   F---J        |
     *   |        ->  |
     *   L---7        |
     * 
     * Case B1: need new corner at the end, orientation RLL?
     * 
     *   F---J        |
     *   |        ->  |
     *   L-------     L---
     * 
     * Case B2: need new corner at the eginning, orientation ?LLR
     * 
     *   F-------     F---
     *   |        ->  |
     *   L---7        |
     * 
     * resulting shape should represent inner shape (top/left border coordinates + 1)
     */
    const PATTERN: [Orient; 4] = [Orient::Right, Orient::Left, Orient::Left, Orient::Right];

    #[inline]
    fn move_next(
        buffer: &mut [Pos; 4],
        orients: &mut [Orient; 4],
        cursor: &mut CursorMut<Pos>
    ) {
        let next = if let Some(next) = cursor.current() {
            *next
        } else {
            cursor.move_next();
            *cursor.current().unwrap()
        };
        *buffer = [buffer[1], buffer[2], buffer[3], next];
        *orients = [orients[1], orients[2], orients[3], Orient::from_corners(&buffer[1..])];
    } 
    #[inline]
    fn remove_before(cursor: &mut CursorMut<Pos>, n: u8) {
        for _ in 0..n {
            cursor.move_prev();
            cursor.remove_current();
        }
    }
    #[inline]
    const fn distance(a: Pos, b: Pos) -> usize {
        a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
    }
    #[inline]
    fn lerp(a: Pos, b: Pos, d: usize) -> Pos {
        if a.0 == b.0 {
            match a.1.cmp(&b.1) {
                Ordering::Less => (a.0, a.1 + d),
                Ordering::Equal => panic!(),
                Ordering::Greater => (b.0, b.1 + d)
            }
        } else if a.1 == b.1 {
            match a.0.cmp(&b.0) {
                Ordering::Less => (a.0 + d, a.1),
                Ordering::Equal => panic!(),
                Ordering::Greater => (b.0 + d, b.1)
            }
        } else {
            panic!();
        }
    }

    let mut len = corners.len();
    let mut quads = Vec::new();
    let mut corners = LinkedList::from_iter(corners);
    let mut cursor = corners.cursor_front_mut();
    let mut buffer = [(0, 0); 4];
    for corner in &mut buffer[1..] {
        *corner = *cursor.current().unwrap();
        cursor.move_next();
    }
    let mut orients = [Orient::Straight; 4];
    orients[3] = Orient::from_corners(&buffer[1..]);
    for _ in 0..3 {
        move_next(&mut buffer, &mut orients, &mut cursor);
    }
    while len != 4 {
        move_next(&mut buffer, &mut orients, &mut cursor);
        // ... -> buffer[0] -> ... -> *buffer[3]* -> ...
        let a = distance(buffer[0], buffer[1]);
        let b = distance(buffer[2], buffer[3]);
        match a.cmp(&b) {
            Ordering::Equal if orients == PATTERN => {
                // Case A
                quads.push(Quad::from_corners(&buffer));
                cursor.remove_current();
                remove_before(&mut cursor, 3);
                len -= 4;
            },
            Ordering::Less if orients[..3] == PATTERN[..3] => {
                // Case B1
                buffer[3] = lerp(buffer[2], buffer[3], a);
                quads.push(Quad::from_corners(&buffer));
                cursor.insert_before(buffer[3]);
                cursor.move_prev();
                remove_before(&mut cursor, 3);
                len -= 2;
            },
            Ordering::Greater if orients[1..] == PATTERN[1..] => {
                // Case B2
                buffer[0] = lerp(buffer[0], buffer[1], b);
                quads.push(Quad::from_corners(&buffer));
                cursor.remove_current();
                remove_before(&mut cursor, 2);
                cursor.insert_before(buffer[0]);
                len -= 2;
            }
            _ => ()
        }
    }
    quads
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
            let (dir, _) = patch(&mut grid, start).dirs().unwrap();
            let mut sum = 0;
            let mut corners = Corners::new(&grid, start, dir)
                .map( |(pos, orient)| {
                    sum += orient.sign();
                    pos
                } )
                .collect_vec();
            if sum.is_negative() {
                corners.reverse();
            }
            1
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