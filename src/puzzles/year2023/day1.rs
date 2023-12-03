use std::borrow::Cow;
use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::anychar,
    branch::alt,
    combinator::{map, value, verify, eof},
};
use crate::{*, parse::*};

fn digit(input: &str) -> IResult<&str, u8> {
    map(
        verify(anychar, |char| char.is_ascii_digit() ),
        |digit| digit.to_digit(10).expect("a digit") as u8
    )(input)
}

fn word(input: &str) -> IResult<&str, u8> {
    alt((
        value(0, tag("zero")),
        value(1, tag("one")),
        value(2, tag("two")),
        value(3, tag("three")),
        value(4, tag("four")),
        value(5, tag("five")),
        value(6, tag("six")),
        value(7, tag("seven")),
        value(8, tag("eight")),
        value(9, tag("nine"))
    ))(input)
}

fn sum<F>(input: &str, parser: F) -> Result<usize, PuzzleError>
    where F: Send + Sync + Clone + FnMut(&str) -> IResult<&str, u8>
{
    let digits = parse(input, lines(
        move |input| many_overlapping_till(parser.clone(), eof)(input)
    ))?;
    let sum = digits.into_iter()
        .map( |line| Some(line.first()? * 10 + line.last()?) )
        .fold(0, |s, n| s + n.unwrap_or(0) as usize );
    Ok(sum)
}

pub fn part1(input: &str) -> Answer {
    Ok(Cow::Owned(sum(input,
        digit
    )?.to_string()))
}

pub fn part2(input: &str) -> Answer {
    Ok(Cow::Owned(sum(input,
        |input| alt((digit, word))(input)
    )?.to_string()))
}

inventory::submit! { Puzzle::new(2023, 1, 1, part1) }
inventory::submit! { Puzzle::new(2023, 1, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        1abc2
        pqr3stu8vwx
        a1b2c3d4e5f
        treb7uchet
    "};
    const OUTPUT1: &str = "142";

    const INPUT2: &str = indoc!{"
        two1nine
        eightwothree
        abcone2threexyz
        xtwone3four
        4nineeightseven2
        zoneight234
        7pqrstsixteen
    "};
    const OUTPUT2: &str = "281";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}