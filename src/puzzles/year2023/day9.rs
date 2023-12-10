use std::borrow::Cow;
use itertools::Itertools;
use nom::{
    IResult,
    character::complete::{char, digit1},
    multi::separated_list1, combinator::{map_res, opt}, sequence::pair
};
use rayon::iter::{ParallelIterator, IntoParallelIterator};
use tap::Pipe;

use crate::{*, parse::*};

struct History {
    data: Vec<i32>
}
impl History {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, data) = separated_list1(char(' '),
            map_res(pair(opt(char::<&str, nom::error::Error<&str>>('-')), digit1),
                |(sign, number)|
                    number.parse::<i32>()
                        .map( |n| if sign.is_some() { -n } else { n } )
            )
        )(input)?;
        Ok((input, Self { data }))
    }
    fn extrapolate_right(&self) -> i32 {
        let mut buffer = Vec::new();
        let len = self.data.len();
        let mut index = 0;
        loop {
            let (mut left, mut right) = (self.data[len - index - 2], self.data[len - index - 1]);
            let len = buffer.len();
            for i in 0..index {
                left = right - left;
                right = buffer[len + i - index];
                buffer.push(left);
            }
            buffer.push(right - left);
            index += 1;
            if left == right { break; }
        }
        let (mut top, mut result) = (buffer.len() - 1, 0);
        while index != 1 {
            top -= index;
            result += buffer[top];
            index -= 1;
        }
        self.data[len - 1] + result
    }
    fn extrapolate_left(&self) -> i32 {
        let len = self.data.len();
        let mut buffer = Vec::with_capacity((len << 1) - 3);
        self.data.iter().tuple_windows()
            .for_each( |(left, right)| buffer.push(right - left) );
        let mut index = 1;
        loop {
            let mut done = true;
            let (mut left, mut right) = (0, 0);
            for i in ((buffer.len() - (len - index))..).take(len - index - 1) {
                (left, right) = (buffer[i], buffer[i + 1]);
                buffer.push(right - left);
                if left != right { done = false; }
            }
            index += 1;
            if done && left == right { break; }
        }
        let (mut top, mut result) = (buffer.len() - (len - index), 0);
        while index != 1 {
            index -= 1;
            top -= len - index;
            result = buffer[top] - result;
        }
        self.data[0] - result
    }
}

pub fn part1(input: &str) -> Answer {
    parse(input, lines(History::parse))?.into_par_iter()
        .map( |history| history.extrapolate_right() )
        .sum::<i32>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    parse(input, lines(History::parse))?.into_par_iter()
        .map( |history| history.extrapolate_left() )
        .sum::<i32>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

inventory::submit! { Puzzle::new(2023, 9, 1, part1) }
inventory::submit! { Puzzle::new(2023, 9, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        0 3 6 9 12 15
        1 3 6 10 15 21
        10 13 16 21 30 45
    "};
    const OUTPUT1: &str = "114";

    const INPUT2: &str = indoc! {"
        0 3 6 9 12 15
        1 3 6 10 15 21
        10 13 16 21 30 45
    "};
    const OUTPUT2: &str = "2";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}