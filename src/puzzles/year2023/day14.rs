use std::borrow::Cow;
use ndarray::{Array2, ShapeBuilder, Axis};
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
    fn tilt_north(&mut self) {
        let (w, h) = self.0.dim();
        for y in 1..h {
            for x in 0..w {
                if self.0[[x, y]] != Tile::Round { continue; }
                let mut i = y;
                while i != 0 && self.0[[x, i - 1]] == Tile::Empty {
                    i -= 1;
                }
                if i == y { continue; }
                self.0[[x, i]] = Tile::Round;
                self.0[[x, y]] = Tile::Empty;
            }
        }
    }
    #[inline]
    fn load_north(&self) -> usize {
        let h = self.0.len_of(Axis(1));
        self.0.axis_iter(Axis(1)).enumerate()
            .map( |(y, row)|
                (h - y) * row.iter().filter( |&tile| *tile == Tile::Round ).count()
            )
            .sum::<usize>()
    }
}

pub fn part1(input: &str) -> Answer {
    parse(input, Platform::parse)?
        .pipe( |mut platform| {
            platform.tilt_north();
            platform.load_north()
        } )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    todo!()
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