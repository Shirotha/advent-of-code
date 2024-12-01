use std::borrow::Cow;
use nom::{
    IResult, 
    bytes::complete::tag, 
    character::complete::{char, digit1},
    sequence::{delimited, separated_pair, preceded, pair},
    multi::{separated_list1, many1},
    combinator::map_res, 
};
use sorted_iter::{assume::AssumeSortedByItemExt, SortedIterator};
use tap::{Pipe, Tap};

use crate::{*, parse::*};

#[derive(Debug)]
struct Card {
    winning: Vec<u8>,
    owned: Vec<u8>
}
impl Card {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, (mut winning, mut owned)) = preceded(
            delimited(
                pair(tag("Card"), many1(char(' '))),
                digit1,
                pair(char(':'), many1(char(' ')))
            ),
            separated_pair(
                separated_list1(many1(char(' ')), map_res(digit1, |number: &str| number.parse::<u8>() )),
                pair(tag(" |"), many1(char(' '))),
                separated_list1(many1(char(' ')), map_res(digit1, |number: &str| number.parse::<u8>() )),
            )
        )(input)?;
        winning.sort_unstable();
        owned.sort_unstable();
        Ok((input, Card { winning, owned }))
    }
    fn into_intersection(self) -> impl Iterator<Item = u8> {
        self.winning.into_iter().assume_sorted_by_item()
            .intersection(self.owned.into_iter().assume_sorted_by_item())
    }
}

pub fn part1(input: &str) -> Answer {
    parse(input, lines(Card::parse))?.into_iter()
        .map( |card|
            card.into_intersection()
                .count()
                .pipe( |n| if n == 0 { 0 } else { 1 << (n - 1) } )
        )
        .sum::<usize>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    parse(input, lines(Card::parse))?.pipe( |cards| {
        let mut counts = vec![1u32; cards.len()];
        cards.into_iter().enumerate().map( |(i, card)| {
            counts[i].tap( |count|
                card.into_intersection()
                    .count()
                    .pipe( |n| 
                        counts.iter_mut()
                            .skip(i + 1).take(n)
                            .for_each( |next| *next += count )
                    )
            )
            } ).sum::<u32>()
            .pipe( |result| Ok(Cow::Owned(result.to_string())) )
    } )

}

inventory::submit! { Puzzle::new(2023, 4, 1, part1) }
inventory::submit! { Puzzle::new(2023, 4, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
        Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
        Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
        Card 4: 41 92 73 84 69 |  9 84 76 51 58  5 54 83
        Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
        Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11
    "};
    const OUTPUT1: &str = "13";

    const INPUT2: &str = indoc! {"
        Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
        Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
        Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
        Card 4: 41 92 73 84 69 |  9 84 76 51 58  5 54 83
        Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
        Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11
    "};
    const OUTPUT2: &str = "30";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}