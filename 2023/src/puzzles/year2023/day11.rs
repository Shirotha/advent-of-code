use std::{
    borrow::Cow,
    collections::BTreeSet,
    mem::MaybeUninit,
};
use itertools::{Itertools, izip};
use nom::IResult;
use tap::Pipe;

use crate::{*, parse::*};

#[derive(Debug)]
struct SparsePointSet {
    points: Vec<[u8; 2]>,
    xs: Vec<u8>,
    ys: Vec<u8>,
    max: [u8; 2]
}
impl SparsePointSet {
    fn parse(input: &str) -> IResult<&str, Self> {
        let mut points = Vec::new();
        let mut xs = BTreeSet::new();
        let mut ys = BTreeSet::new();
        let mut width = None;
        let mut x = 0;
        let mut y = 0;
        for chr in input.chars() {
            match chr {
                '\n' => {
                    if let Some(width) = width {
                        if width != x {
                            assert_eq!(width, x);
                        }
                    } else {
                        width = Some(x);
                    }
                    x = 0;
                    y += 1;
                    continue;
                }
                '#' => {
                    xs.insert(x);
                    ys.insert(y);
                    points.push([x, y]);
                }
                '.' => (),
                _ => continue,
            }
            x += 1;
        }
        let xs = xs.into_iter().collect_vec();
        let ys = ys.into_iter().collect_vec();
        let max = [*xs.last().unwrap(), *ys.last().unwrap()];
        Ok(("", Self { points, xs, ys, max}))
    }
}

#[derive(Debug)]
struct Metric1 {
    cost: Vec<u32>
}
impl Metric1 {
    fn new(set: &[u8], max: u8, expand: u32) -> Self {
        // ASSERT: max >= set.last
        let mut cost = vec![0; max as usize + 1];
        let mut i = 0usize;
        for next in set {
            while i as u8 != *next {
                cost[i] = cost[i.saturating_sub(1)] + expand;
                i += 1;
            }
            cost[i] = cost[i.saturating_sub(1)] + 1;
            i += 1;
        }
        Metric1 { cost }
    }
    fn distance(&self, a: u8, b: u8) -> u64 {
        self.cost[a as usize].abs_diff(self.cost[b as usize]) as u64
    }
}
#[derive(Debug)]
struct Metric<const N: usize>([Metric1; N]);
impl<const N: usize> Metric<N> {
    fn new(sets: [&[u8]; N], max: [u8; N], expand: u32) -> Self {
        unsafe {
            let mut dims = MaybeUninit::<Metric1>::uninit_array::<N>();
            for (metric, set, max) in izip!(&mut dims, sets, max) {
                metric.write(Metric1::new(set, max, expand));
            }
            Self(MaybeUninit::array_assume_init(dims))
        }
    }
    fn distance(&self, a: [u8; N], b: [u8; N]) -> u64 {
        izip!(&self.0, a, b).map( |(metric, a, b)| metric.distance(a, b) ).sum::<u64>()
    }
}

pub fn part1(input: &str) -> Answer {
    parse(input, SparsePointSet::parse)?.pipe( |sps| {
        let metric = Metric::new([&sps.xs, &sps.ys], sps.max, 2);
        sps.points.into_iter().tuple_combinations()
            .map( |(a, b)| metric.distance(a, b))
            .sum::<u64>()
    } )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    parse(input, SparsePointSet::parse)?.pipe( |sps| {
        let metric = Metric::new([&sps.xs, &sps.ys], sps.max, 1_000_000);
        sps.points.into_iter().tuple_combinations()
            .map( |(a, b)| metric.distance(a, b))
            .sum::<u64>()
    } )
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

inventory::submit! { Puzzle::new(2023, 11, 1, part1) }
inventory::submit! { Puzzle::new(2023, 11, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        ....#........
        .........#...
        #............
        .............
        .............
        ........#....
        .#...........
        ............#
        .............
        .............
        .........#...
        #....#.......
    "};
    const OUTPUT1: &str = "538";

    const INPUT2: &str = indoc! {"
        ....#........
        .........#...
        #............
        .............
        .............
        ........#....
        .#...........
        ............#
        .............
        .............
        .........#...
        #....#.......    
    "};
    const OUTPUT2: &str = "164000210";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}