use std::{
    borrow::Cow,
    collections::{VecDeque, HashSet}
};
use bit_vec::BitVec;
use nom::IResult;
use petgraph::{
    graph::DiGraph,
    visit::{GraphBase, depth_first_search, DfsEvent}
};
use tap::Pipe;

use crate::{*, parse::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dir {
    E,
    N,
    W,
    S
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Splitter {
    Horizontal,
    Vertical
}
impl Splitter {
    #[inline]
    fn split(&self, dir: Dir) -> Option<[Dir; 2]> {
        match self {
            Self::Horizontal => if matches!(dir, Dir::N | Dir::S) { Some([Dir::E, Dir::W]) } else { None }
            Self::Vertical => if matches!(dir, Dir::E | Dir::W) { Some([Dir::N, Dir::S]) } else { None }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mirror {
    A,
    B
}
impl Mirror {
    #[inline]
    fn reflect(&self, dir: Dir) -> Dir {
        match dir {
            Dir::E => if *self == Mirror::A { Dir::N } else { Dir::S },
            Dir::N => if *self == Mirror::A { Dir::E } else { Dir::W },
            Dir::W => if *self == Mirror::A { Dir::S } else { Dir::N },
            Dir::S => if *self == Mirror::A { Dir::W } else { Dir::E }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RayCast(Pos, Dir, u8);
impl RayCast {
    #[inline]
    fn step(&mut self, max: Pos) -> bool {
        match self.1 {
            Dir::E if self.0[0] != max[0] => {
                self.0[0] += 1;
                self.2 += 1;
                true
            },
            Dir::N if self.0[1] != 0 => {
                self.0[1] -= 1;
                self.2 += 1;
                true
            },
            Dir::W if self.0[0] != 0 => {
                self.0[0] -= 1;
                self.2 += 1;
                true
            }
            Dir::S if self.0[1] != max[1] => {
                self.0[1] += 1;
                self.2 += 1;
                true
            },
            _ => false
        }
    }
    #[inline]
    fn try_from(pos: Pos, dir: Dir, max: Pos) -> Option<Self> {
        let mut out = RayCast(pos, dir, 0);
        if out.step(max) {
            Some(out)
        } else {
            None
        }
    }
}

type Pos = [usize; 2];
type Graph = DiGraph<(Pos, Tile), (Dir, u8), u16>;

fn parse_graph(input: &str) -> IResult<&str, (Graph, <Graph as GraphBase>::NodeId, Pos)> {
    let (input, grid) = grid(input, Tile::from_char)?;
    let mut graph = Graph::default();
    let root = graph.add_node(([0, 0], grid[[0, 0]]));
    let mut open = VecDeque::new();
    let mut closed = HashSet::new();
    open.push_back((root, RayCast([0, 0], Dir::E, 0)));
    let max = {
        let dim = grid.dim();
        [dim.0 - 1, dim.1 - 1]
    };
    while let Some((parent, mut raycast)) = open.pop_back() {
        while grid[raycast.0] == Tile::Empty && raycast.step(max) {}
        let (pos, dir, tile) = (raycast.0, raycast.1, grid[raycast.0]);
        if closed.contains(&(pos, dir)) { continue; }
        closed.insert((pos, dir));
        let node = graph.add_node((pos, tile));
        graph.add_edge(parent, node, (dir, raycast.2));
        match tile {
            Tile::Empty => (),
            Tile::Mirror(mirror) => {
                let dir = mirror.reflect(dir);
                if let Some(out) = RayCast::try_from(pos, dir, max) {
                    open.push_back((node, out));
                }
            },
            Tile::Splitter(splitter) => {
                if let Some([a, b]) = splitter.split(dir) {
                    if let Some(out) = RayCast::try_from(pos, a, max) {
                        open.push_back((node, out));
                    }
                    if let Some(out) = RayCast::try_from(pos, b, max) {
                        open.push_back((node, out));
                    }
                } else if let Some(out) = RayCast::try_from(pos, dir, max) {
                    open.push_back((node, out));
                }
            }
        }
    }
    Ok((input, (graph, root, [max[0] + 1, max[1] + 1])))
}
 
pub fn part1(input: &str) -> Answer {
    parse(input, parse_graph)?
        .pipe( |(graph, root, [w, h])| {
            let mut grid = BitVec::from_elem(w * h, false);
            depth_first_search(&graph, [root], |event| {
                match event {
                    DfsEvent::Discover(node, _) => {
                        let ([x, y], _) = graph[node];
                        grid.set(y * w + x, true);
                    },
                    DfsEvent::TreeEdge(a, b) => {
                        let (pos, _) = graph[a];
                        let edge = graph.find_edge(a, b).unwrap();
                        let (dir, len) = graph[edge];
                        let mut raycast = RayCast::try_from(pos, dir, [w, h]).unwrap();
                        for _ in 0..len {
                            let [x, y] = raycast.0;
                            grid.set(y * w + x, true);
                            raycast.step([w, h]);
                        }
                    },
                    _ => ()
                }
            } );
            grid.into_iter().filter( |x| *x ).count()
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
    const OUTPUT1: &str = "46";

    const INPUT2: &str = indoc! {r#"
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
    const OUTPUT2: &str = "51";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}