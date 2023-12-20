use std::borrow::Cow;
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

struct Platform(Array2<Tile>);
impl Platform {
    fn parse(input: &str) -> IResult<&str, Self> {
        let mut buffer = Vec::new();
        let (mut current, mut width, mut height) = (0, None, 0);
        for chr in input.chars() {
            match chr {
                '\n' => {
                    if let Some(width) = width {
                        if width != current {
                            break;
                        }
                    } else {
                        width = Some(current);
                    }
                    current = 0;
                    height += 1;
                }
                '\r' => (),
                _ => {
                    buffer.push(Tile::from_char(chr));
                    current += 1;
                }
            }
        }
        let width = width.unwrap();
        if current == width {
            height += 1;
        }
        let len = width * height;
        buffer.resize(len, Tile::Empty);
        let data = Array2::from_shape_vec((width, height).f(), buffer).unwrap();
        Ok(("", Self(data)))
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
    // FIXME: cycle will produce a cyclic load, not a steady state
    #[inline]
    fn steady_state_load(&mut self, infinity: usize) -> usize {
        let mut last = self.load();
        for _ in 0..infinity {
            self.cycle();
            let load = self.load();
            if load == last { return load; }
            last = load;
        }
        last
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
            platform.steady_state_load(1_00/*0_000_000*/)
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