use std::{borrow::Cow, collections::VecDeque, ptr::NonNull};
use nom::IResult;
use smallvec::SmallVec;
use tap::Pipe;

use crate::{*, parse::*, collections::nodes::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    E,
    N,
    W,
    S
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Splitter {
    Horizontal,
    Vertical
}
impl Splitter {
    #[inline]
    fn split(&self, dir: Dir) -> Option<[Dir; 2]> {
        match self {
            Self::Horizontal => if matches!(dir, Dir::N | Dir::S) { Some([Dir::N, Dir::S]) } else { None }
            Self::Vertical => if matches!(dir, Dir::E | Dir::W) { Some([Dir::E, Dir::W]) } else { None }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mirror {
    A,
    B
}
impl Mirror {
    #[inline]
    fn reflect(&self, dir: Dir) -> Dir {
        match dir {
            Dir::E => if *self == Mirror::A { Dir::S } else { Dir::N },
            Dir::N => if *self == Mirror::A { Dir::W } else { Dir::E },
            Dir::W => if *self == Mirror::A { Dir::N } else { Dir::S },
            Dir::S => if *self == Mirror::A { Dir::E } else { Dir::W }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Empty,
    Splitter(Splitter),
    Mirror(Mirror)
}
impl Tile {
    #[inline]
    fn from_char(chr: char) -> Self {
        match chr {
            '.' => Self::Empty,
            '-' => Self::Splitter(Splitter::Horizontal),
            '|' => Self::Splitter(Splitter::Vertical),
            '/' => Self::Mirror(Mirror::A),
            '\\' => Self::Mirror(Mirror::B),
            _ => panic!()
        }
    }
}

type Pos = [usize; 2];

node! {
    Optic(
        (Tile, Pos),
        LinkWrapper<Optic, SmallVec<[Ref<Optic>; 2]>>
    ) {
        (Tile::Empty, [0, 0])
    }
}
impl Optic {
    fn parse_graph(pool: &mut Pool<Self>) -> impl FnMut(&str) -> IResult<&str, NonNull<Self>> + '_ {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct RayCast(Pos, Dir);
        impl RayCast {
            #[inline]
            fn step(&mut self, max: Pos) -> bool {
                match self.1 {
                    Dir::E if self.0[0] != max[0] => {
                        self.0[0] += 1;
                        true
                    },
                    Dir::N if self.0[1] != 0 => {
                        self.0[1] -= 1;
                        true
                    },
                    Dir::W if self.0[0] != 0 => {
                        self.0[0] -= 1;
                        true
                    }
                    Dir::S if self.0[1] != max[1] => {
                        self.0[1] += 1;
                        true
                    },
                    _ => false
                }
            }
            #[inline]
            fn try_from(pos: Pos, dir: Dir, max: Pos) -> Option<Self> {
                let mut out = RayCast(pos, dir);
                if out.step(max) {
                    Some(out)
                } else {
                    None
                }
            }
        }

        |input| {
            let mut node = |pos: Pos, tile: Tile| {
                let mut ptr = pool.get().expect("valid new node");
                let node = unsafe { ptr.as_mut() };
                node.data = (tile, pos);
                match tile {
                    Tile::Empty if pos == [0, 0] => {
                        node.link.push(None);
                    },
                    Tile::Mirror(_) => {
                        node.link.push(None);
                    },
                    Tile::Splitter(_) => {
                        node.link.push(None);
                        node.link.push(None);
                    },
                    _ => ()
                }
                ptr
            };
            let (input, grid) = grid(input, Tile::from_char)?;
            let mut open = VecDeque::new();
            let mut root = Some(node([0, 0], Tile::Empty));
            open.push_back((&mut root, RayCast([0, 0], Dir::E)));
            let max = {
                let dim = grid.dim();
                [dim.0 - 1, dim.1 - 1]
            };
            while let Some((ptr, mut raycast)) = open.pop_back() {
                while grid[raycast.0] == Tile::Empty && raycast.step(max) {}
                let (pos, tile) = (raycast.0, grid[raycast.0]);
                let mut next = node(pos, tile);
                let node = unsafe { next.as_mut() };
                match tile {
                    Tile::Empty => (),
                    Tile::Mirror(mirror) => if let Some(out) = RayCast::try_from(pos, mirror.reflect(raycast.1), max) {
                        open.push_back((&mut node.link[0], out));
                    },
                    Tile::Splitter(splitter) => {
                        if let Some([a, b]) = splitter.split(raycast.1) {
                            let [l1, l2] = node.link.get_many_mut([0, 1]).unwrap();
                            if let Some(out) = RayCast::try_from(pos, a, max) {
                                open.push_back((l1, out));
                            }
                            if let Some(out) = RayCast::try_from(pos, b, max) {
                                open.push_back((l2, out));
                            }
                        } else if let Some(out) = RayCast::try_from(pos, raycast.1, max) {
                            open.push_back((&mut node.link[0], out));
                        }
                    }
                }
                *ptr = Some(next);
            }
            Ok((input, root.unwrap()))
        }
    }
}

pub fn part1(input: &str) -> Answer {
    let mut pool = Pool::new().expect("valid pool");
    parse(input, Optic::parse_graph(&mut pool))?
        .pipe( |root| {
            0
        } )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    todo!()
}

inventory::submit! { Puzzle::new(2023, 16, 1, part1) }
inventory::submit! { Puzzle::new(2023, 16, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {r#"
        .|...\....
        |.-.\.....
        .....|-...
        ........|.
        ..........
        .........\
        ..../.\\..
        .-.-/..|..
        .|....-|.\
        ..//.|....
    "#};
    const OUTPUT1: &str = "";

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