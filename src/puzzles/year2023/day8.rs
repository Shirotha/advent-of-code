use std::{
    borrow::Cow,
    collections::HashSet
};
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::{line_ending, one_of, alphanumeric1},
    sequence::{separated_pair, delimited, preceded},
    combinator::iterator, multi::many1
};
use tap::{Tap, Pipe};

use crate::{*, parse::*};

#[derive(Debug)]
struct BinaryGraph<'a> {
    nodes: HashMap<&'a str, usize>,
    edges: Vec<(usize, usize)>,
    pos: Vec<usize>,
    goal: HashSet<usize>
}
impl<'a> BinaryGraph<'a> {
    fn parse_sequential(input: &'a str) -> IResult<&str, Self> {
        let mut iter = iterator(input, preceded(line_ending,
            separated_pair(
                alphanumeric1,
                tag(" = "),
                delimited(
                    tag("("),
                    separated_pair(alphanumeric1, tag(", "), alphanumeric1),
                    tag(")")
                )
            )
        ));
        let mut result = Self { nodes: HashMap::new(), edges: Vec::new(), pos: Vec::new(), goal: HashSet::new() };
        iter.for_each( |(node, (left, right))| _ = result.insert(node, left, right) );
        result.index("AAA").pipe( |i| result.pos.push(i) );
        result.index("ZZZ").pipe( |i| result.goal.insert(i) );
        Ok((iter.finish()?.0, result))
    }
    fn parse_parallel(input: &'a str) -> IResult<&str, Self> {
        let mut iter = iterator(input, preceded(line_ending,
            separated_pair(
                alphanumeric1,
                tag(" = "),
                delimited(
                    tag("("),
                    separated_pair(alphanumeric1, tag(", "), alphanumeric1),
                    tag(")")
                )
            )
        ));
        let mut result = Self { nodes: HashMap::new(), edges: Vec::new(), pos: Vec::new(), goal: HashSet::new() };
        for (node, (left, right)) in &mut iter {
            let index = result.insert(node, left, right);
            if node.ends_with('A') {
                result.pos.push(index);
            } else if node.ends_with('Z') {
                result.goal.insert(index);
            }
        }
        Ok((iter.finish()?.0, result))
    }
    #[inline]
    fn index(&mut self, node: &'a str) -> usize {
        *self.nodes.entry(node)
            .or_insert_with( || self.edges.len().tap( |&i| self.edges.push((i, i)) ) )
    }
    #[inline]
    fn insert(&mut self, node: &'a str, left: &'a str, right: &'a str) -> usize {
        let node = self.index(node);
        let left = self.index(left);
        let right = self.index(right);
        self.edges[node] = (left, right);
        node
    }

    #[inline]
    fn moveto(&mut self, instruction: &char) -> bool {
        match instruction {
            'L' => self.pos.iter_mut().for_each( |p| *p = self.edges[*p].0 ),
            'R' => self.pos.iter_mut().for_each( |p| *p = self.edges[*p].1 ),
            _ => panic!()
        }
        self.pos.iter().all( |p| self.goal.contains(p) )
    }
}

fn instructions(input: &str) -> IResult<&str, impl Iterator<Item = char>> {
    let (input, instructions) = many1(one_of("LR"))(input)?;
    Ok((input, instructions.into_iter().cycle()))
}

pub fn part1(input: &str) -> Answer {
    parse(input, separated_pair(instructions, line_ending, BinaryGraph::parse_sequential))?
        .pipe( |(inst, mut graph)| 
            inst.take_while( |inst| !graph.moveto(inst) ).count() + 1
        )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    parse(input, separated_pair(instructions, line_ending, BinaryGraph::parse_parallel))?
        .pipe( |(inst, mut graph)| 
            inst.take_while( |inst| !graph.moveto(inst) ).take(1_000_000).count() + 1
        )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

inventory::submit! { Puzzle::new(2023, 8, 1, part1) }
inventory::submit! { Puzzle::new(2023, 8, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        LLR

        AAA = (BBB, BBB)
        BBB = (AAA, ZZZ)
        ZZZ = (ZZZ, ZZZ)
    "};
    const OUTPUT1: &str = "6";

    const INPUT2: &str = indoc! {"
        LR

        11A = (11B, XXX)
        11B = (XXX, 11Z)
        11Z = (11B, XXX)
        22A = (22B, XXX)
        22B = (22C, 22C)
        22C = (22Z, 22Z)
        22Z = (22B, 22B)
        XXX = (XXX, XXX)
    "};
    const OUTPUT2: &str = "6";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}