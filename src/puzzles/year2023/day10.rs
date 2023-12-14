use std::{
    borrow::Cow,
    cmp::{Ordering, minmax},
    collections::{LinkedList, linked_list::CursorMut, hash_map::Entry},
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
    Right    = 0x01,
    Straight = 0x04,
    Left     = 0x10
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
                Ordering::Less => b.0.cmp(&c.0),
                Ordering::Equal => panic!(),
                Ordering::Greater => c.0.cmp(&b.0)
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
            Ordering::Less => Self::Right,
            Ordering::Equal => panic!(),
            Ordering::Greater => Self::Left
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
        let to = dirs.rotate_right(from.trailing_zeros() + 2) & 0x15;
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
    // topleft: Pos,
    size: Pos
}
impl Quad {
    #[inline]
    fn from_corners(a: Pos, b: Pos) -> Quad {
        let ([ymin, ymax], [xmin, xmax]) = (minmax(a.0, b.0), minmax(a.1, b.1));
        Quad { /* topleft: (xmin, ymin), */ size: (ymax - ymin, xmax - xmin) }
    }
    #[inline]
    const fn area(&self) -> usize {
        self.size.0 * self.size.1
    }
}

fn quadrangulate(mut corners: Vec<(usize, usize)>) -> Vec<Quad> {
    // ASSERT: corners are positively oriented
    // NOTE: positions will be treated as top-left of the tile
    // NOTE: returns inner shape (without borders)
    const PATTERN: [Orient; 4] = [Orient::Right, Orient::Left, Orient::Left, Orient::Right];

    #[inline]
    fn next<'a>(cursor: &'a mut CursorMut<Pos>) -> &'a mut Pos {
        cursor.move_next();
        if cursor.current().is_none() {
            cursor.move_next();
        } 
        cursor.current().unwrap()
    }
    #[inline]
    fn prev<'a>(cursor: &'a mut CursorMut<Pos>) -> &'a mut Pos {
        cursor.move_prev();
        if cursor.current().is_none() {
            cursor.move_prev();
        }
        cursor.current().unwrap()
    }
    #[inline]
    fn remove_before(cursor: &mut CursorMut<Pos>, n: u8) {
        for _ in 0..n {
            prev(cursor);
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
    #[inline]
    fn update_next(
        buffer: &mut [Pos; 5],
        orients: &mut [Orient; 4],
        cursor: &mut CursorMut<Pos>
    ) {
        let next = *next(cursor);
        *buffer = [buffer[1], buffer[2], buffer[3], buffer[4], next];
        *orients = [orients[1], orients[2], orients[3], Orient::from_corners(&buffer[2..])];
    } 
    #[inline]
    fn init_here(
        buffer: &mut [Pos; 5],
        orients: &mut [Orient; 4],
        cursor: &mut CursorMut<Pos>
    ) {
        for corner in &mut buffer[3..] {
            *corner = *next(cursor);
        }
        for _ in 0..4 {
            update_next(buffer, orients, cursor);
        }
    }
    #[inline]
    fn split_before(cursor: &mut CursorMut<Pos>, len: usize) -> LinkedList<Pos> {
        // ASSERT: cursor doesn't cross None
        let mut middle = cursor.split_before();
        let mut local = middle.cursor_back_mut();
        for _ in 1..len {
            local.move_prev();
        }
        let left = local.split_before();
        cursor.splice_before(left);
        middle
    }
    #[inline]
    fn split_after(cursor: &mut CursorMut<Pos>, len: usize) -> LinkedList<Pos> {
        // ASSERT: cursor doesn't cross None
        let mut middle = cursor.split_after();
        let mut local = middle.cursor_front_mut();
        for _ in 1..len {
            local.move_next();
        }
        let right = local.split_after();
        cursor.splice_after(right);
        middle
    }
    fn partition(quads: &mut Vec<Quad>, cursor: &mut CursorMut<Pos>, mut len: usize) {
        { // DEBUG: print corners
            for _ in 0..len {
                let corner = *next(cursor);
                println!("(x: {}, y: {})", corner.1, corner.0);
            }
        }
        let mut buffer = [(0, 0); 5];
        let mut orients = [Orient::Straight; 4];
        init_here(&mut buffer, &mut orients, cursor);
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
        while len != 4 {
            // FIXME: lerp points wrong (replace b with a - b ?)
            update_next(&mut buffer, &mut orients, cursor);
            // ... -> buffer[0] -> ... -> buffer[3] -> *buffer[4]* -> ...
            let a = distance(buffer[0], buffer[1]);
            let b = distance(buffer[2], buffer[3]);
            match a.cmp(&b) {
                Ordering::Equal if orients == PATTERN => {
                    // Case A
                    quads.push(Quad::from_corners(buffer[0], buffer[2]));
                    remove_before(cursor, 4);
                    len -= 4;
                    init_here(&mut buffer, &mut orients, cursor);
                },
                Ordering::Less if orients[..3] == PATTERN[..3] => {
                    // Case B1
                    quads.push(Quad::from_corners(buffer[0], buffer[2]));
                    prev(cursor);
                    remove_before(cursor, 2);
                    *prev(cursor) = lerp(buffer[2], buffer[3], a);
                    len -= 2;
                    init_here(&mut buffer, &mut orients, cursor);
                },
                Ordering::Greater if orients[1..] == PATTERN[1..] => {
                    // Case B2
                    quads.push(Quad::from_corners(buffer[1], buffer[3]));
                    prev(cursor);
                    remove_before(cursor, 2);
                    *cursor.current().unwrap() = lerp(buffer[0], buffer[1], b);
                    len -= 2;
                    init_here(&mut buffer, &mut orients, cursor);
                }
                _ => ()
            }
        }
        quads.push(Quad::from_corners(buffer[0], buffer[2]));
    }

    let mut quads = Vec::new();
    let mut len = corners.len();
    for (i, j) in (0..len).circular_tuple_windows() {
        let [a, b] = corners.get_many_mut([i, j]).unwrap();
        if a.0 == b.0 && a.1 > b.1 {
            a.0 += 1;
            b.0 += 1;
        } else if a.1 == b.1 && a.0 > b.0 {
            a.1 += 1;
            b.1 += 1;
        }
    }
    let mut corners = LinkedList::from_iter(corners);
    let mut cursor = corners.cursor_front_mut();
    {
        /*
         * remove zero size quads (after shift)
         * 
         * Case A: remove pairs twice
         * 
         *   F---J     ->  |
         *   L---7         |
         * 
         * Case B1: remove pair, last corner flips orientation
         * 
         *   F---J         |
         *   L-------  ->  L---
         * 
         * Case B2: remove pair, last corner flips orientation
         * 
         *   F-------
         *   L---7     ->  F---
         * 
         */
        let mut last = *cursor.current().unwrap();
        #[allow(clippy::mut_range_bound)]
        for _ in 0..len {
            let current = *next(&mut cursor);
            // Case *
            if last == current {
                cursor.remove_current();
                remove_before(&mut cursor, 1);
                // Case A
                last = *prev(&mut cursor);
                len -= 2;
            } else {
                last = current;
            }
        }
        if len == 0 {
            return quads;
        }
    }
    /*
     * split polygon at overlapping points
     * 
     *     I | O
     *    ---%---
     *     O | I
     * 
     */
    let mut visited = HashMap::new();
    let mut i: usize = 0;
    let mut parity = false;
    loop {
        cursor.move_next();
        if cursor.current().is_none() {
            cursor.move_next();
            parity = !parity;
        }
        let current = *cursor.current().unwrap();
        match visited.entry(current) {
            Entry::Occupied(previous) => {
                let (j, p) = *previous.get();
                /*
                 * sub-list between j and i is free off overlapp and can be handled as a simple loop
                 * 
                 * Case A: split off before i, split off before j, join left and right parts, handle middle part
                 * 
                 *   None -> ... -> (j -> ... -> i - 1) -> i -> ... -> None
                 * 
                 * Case B: split off before i, split off before j, join left nd right parts, handle merged part
                 * 
                 *         ... -> (j -> ... -> None -> ... -> i - 1) -> i -> ...
                 *                                <=>
                 *   None -> ... -> i - 1) -> [i -> ... -> j - 1] -> (j -> ... -> None
                 * 
                 * Case C: there are no overlapps, handle whole list (i - j == len)
                 * 
                 *   .. -> j = i -> ...
                 * 
                 */
                let sub = i - j;
                if sub == len {
                    partition(&mut quads, &mut cursor, len);
                    return quads;
                }
                visited.clear();
                len -= sub;
                if p != parity {
                    // TODO: think about if None can be reached here (and if it even matters)
                    cursor.move_prev();
                    let outer = split_after(&mut cursor, len);
                    partition(&mut quads, &mut cursor, len);
                    corners = outer;
                    cursor = corners.cursor_front_mut();
                } else {
                    let mut inner = split_before(&mut cursor, sub);
                    partition(&mut quads, &mut inner.cursor_front_mut(), sub);
                }
            },
            Entry::Vacant(empty) => {
                empty.insert((i, parity));
                i += 1;
            }
        }
    }
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
            quadrangulate(corners).into_iter()
                .map( |q| q.area() )
                .sum::<usize>()
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