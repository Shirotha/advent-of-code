use std::{
    borrow::Cow,
    collections::hash_map::Entry
};
use bit_vec::BitVec;
use ndarray::prelude::*;
use nom::IResult;
use tap::Pipe;

use crate::{*, parse::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Empty,
    Round,
    Cube
}
impl Tile {
    #[inline]
    fn from_char(chr: char) -> Self {
        match chr {
            '.' => Self::Empty,
            'O' => Self::Round,
            '#' => Self::Cube,
            _ => panic!()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    N,
    W,
    S,
    E
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Platform(Array2<Tile>);
impl Platform {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, data) = grid(input, Tile::from_char)?;
        Ok((input, Self(data)))
    }
    fn tilt(&mut self, dir: Dir) {
        #[inline(always)]
        fn increment(x: &mut usize, reverse: bool) {
            if reverse {
                *x -= 1
            } else {
                *x += 1
            }
        }
        #[inline]
        fn tilt_view(mut view: ArrayViewMut2<Tile>, reverse: bool) {
            let (inner_len, outer_len) = view.dim();
            /*/ DEBUG
            for i in 0..outer_len {
                for j in 0..inner_len {
                    match view[[j, i]] {
                        Tile::Empty => print!("."),
                        Tile::Round => print!("O"),
                        Tile::Cube => print!("#")
                    }
                }
                println!();
            }
            println!();
            // DEBUG end */
            let (min, max) = if reverse {
                (outer_len - 1, 0)
            } else {
                (0, outer_len - 1)
            };
            let mut outer = min;
            while outer != max {
                increment(&mut outer, reverse);
                for inner in 0..inner_len {
                    if view[[inner, outer]] != Tile::Round { continue; }
                    let mut i = outer;
                    while i != min {
                        increment(&mut i, !reverse);
                        if view[[inner, i]] != Tile::Empty {
                            increment(&mut i, reverse);
                            break;
                        }
                    }
                    if i == outer { continue; }
                    view[[inner, i]] = Tile::Round;
                    view[[inner, outer]] = Tile::Empty;
                }
            }
        }

        let (mirror, reverse) = match dir {
            Dir::N => (false, false),
            Dir::W => (true, false),
            Dir::S => (false, true),
            Dir::E => (true, true)
        };
        if mirror {
            let (w, h) = self.0.dim();
            let buffer = self.0.as_slice_memory_order_mut().unwrap();
            let mut view = ArrayViewMut2::from_shape((h, w), buffer).unwrap();
            tilt_view(view.view_mut(), reverse);
        } else {
            tilt_view(self.0.view_mut(), reverse);
        }
    }
    #[inline]
    fn cycle(&mut self) {
        self.tilt(Dir::N);
        self.tilt(Dir::W);
        self.tilt(Dir::S);
        self.tilt(Dir::E);
    }
    #[inline]
    fn load(&self) -> usize {
        let h = self.0.len_of(Axis(1));
        self.0.axis_iter(Axis(1)).enumerate()
            .map( |(y, row)|
                (h - y) * row.iter().filter( |&tile| *tile == Tile::Round ).count()
            )
            .sum::<usize>()
    }
    #[inline]
    fn is_round(&self) -> BitVec {
        let (w, h) = self.0.dim();
        let mut round = BitVec::from_elem(w * h, false);
        for y in 0..h {
            for x in 0..w {
                if self.0[[x, y]] == Tile::Round {
                    round.set(y * w + x, true);
                }
            }
        }
        round
    }
    #[inline]
    fn stabilized_load(&mut self, count: usize) -> usize {
        let mut visited = HashMap::new();
        visited.insert(self.is_round(), 0);
        for i in 1..=count {
            self.cycle();
            let round = self.is_round();
            match visited.entry(round) {
                Entry::Occupied(e) => {
                    let period = i - e.get();
                    let remaining = (count - i) % period;
                    for _ in 0..remaining {
                        self.cycle();
                    }
                    return self.load();
                },
                Entry::Vacant(e) => _ = e.insert(i)
            }
        }
        self.load()
    }
}

pub fn part1(input: &str) -> Answer {
    parse(input, Platform::parse)?
        .pipe( |mut platform| {
            platform.tilt(Dir::N);
            platform.load()
        } )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    parse(input, Platform::parse)?
        .pipe( |mut platform| {
            platform.stabilized_load(1_000_000_000)
        } )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

inventory::submit! { Puzzle::new(2023, 14, 1, part1) }
inventory::submit! { Puzzle::new(2023, 14, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        O....#....
        O.OO#....#
        .....##...
        OO.#O....O
        .O.....O#.
        O.#..O.#.#
        ..O..#O..O
        .......O..
        #....###..
        #OO..#....
    "};
    const OUTPUT1: &str = "136";

    const INPUT2: &str = indoc! {"
        O....#....
        O.OO#....#
        .....##...
        OO.#O....O
        .O.....O#.
        O.#..O.#.#
        ..O..#O..O
        .......O..
        #....###..
        #OO..#....
    "};
    const OUTPUT2: &str = "64";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}