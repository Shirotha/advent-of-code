use std::{
    borrow::Cow,
    cmp::Ordering
};
use nom::{
    IResult,
    character::complete::{char, alphanumeric1, digit1},
    combinator::{verify, map_res},
    sequence::separated_pair};
use tap::{Pipe, Tap};

use crate::{*, parse::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Card {
    JJ,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    T,
    J,
    Q,
    K,
    A
}
impl Card {
    #[inline]
    const fn from_u8<const J: bool>(char: u8) -> Self {
        match char {
            b'2' => Card::D2,
            b'3' => Card::D3,
            b'4' => Card::D4,
            b'5' => Card::D5,
            b'6' => Card::D6,
            b'7' => Card::D7,
            b'8' => Card::D8,
            b'9' => Card::D9,
            b'T' => Card::T,
            b'J' => if J { Card::JJ } else { Card::J },
            b'Q' => Card::Q,
            b'K' => Card::K,
            b'A' => Card::A,
            _ => panic!("invalid character")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Hand<const N: usize> {
    cards: [Card; N],
    value: u8
}
impl<const N: usize> Hand<N> {
    fn parse(input: &str) -> IResult<&str, Self> {
        let (input, bytes) = verify(alphanumeric1, |input: &str| input.len() == N )(input)?;
        let bytes = bytes.as_bytes();
        let mut cards = [Card::A; N];
        let mut found = [0; N];
        let mut points = [0; N];
        let mut n = 0;
        'outer: for (i, &c) in bytes.iter().enumerate() {
            cards[i] = Card::from_u8::<false>(c);
            for i in 0..n {
                if c == found[i] {
                    points[i] = (points[i] << 1) | 1;
                    continue 'outer;
                }
            }
            found[n] = c;
            points[n] = 1;
            n += 1;
        }
        Ok((input, Self { cards, value: points.into_iter().sum() }))
    }
    fn parse_joker(input: &str) -> IResult<&str, Self> {
        let (input, bytes) = verify(alphanumeric1, |input: &str| input.len() == N )(input)?;
        let bytes = bytes.as_bytes();
        let mut cards = [Card::A; N];
        let mut found = [0; N];
        let mut points = [0; N];
        let mut jokers = 0;
        let (mut best_p, mut best_i) = (0, 0);
        let mut n = 0;
        'outer: for (i, &c) in bytes.iter().enumerate() {
            let x = Card::from_u8::<true>(c);
            cards[i] = x;
            if x == Card::JJ {
                jokers += 1;
                continue;
            }
            for i in 0..n {
                if c == found[i] {
                    let p = (points[i] << 1) | 1;
                    points[i] = p;
                    if p > best_p {
                        (best_p, best_i) = (p, i);
                    }
                    continue 'outer;
                }
            }
            found[n] = c;
            points[n] = 1;
            n += 1;
        }
        if jokers != N && best_p == 0 { best_p = 1; }
        points[best_i] = (1 << jokers) * (best_p + 1) - 1;
        Ok((input, Self { cards, value: points.into_iter().sum() }))
    }
}
impl<const N: usize> Ord for Hand<N> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.value.cmp(&other.value) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.cards.cmp(&other.cards),
            Ordering::Greater => Ordering::Greater
        }
    }
}
impl<const N: usize> PartialOrd for Hand<N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn part1(input: &str) -> Answer {
    const N: usize = 5;
    parse(input, lines( |input|
            separated_pair(
                Hand::<N>::parse,
                char(' '),
                map_res(digit1, |number: &str| number.parse::<u16>())
            )(input)
        ))?
        .tap_mut( |players| players.sort_unstable_by_key( |p| p.0 ) )
        .into_iter()
        .enumerate()
        .map( |(place, player)| player.1 as u32 * (place as u32 + 1) )
        .sum::<u32>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

pub fn part2(input: &str) -> Answer {
    const N: usize = 5;
    parse(input, lines( |input|
            separated_pair(
                Hand::<N>::parse_joker,
                char(' '),
                map_res(digit1, |number: &str| number.parse::<u16>())
            )(input)
        ))?
        .tap_mut( |players| players.sort_unstable_by_key( |p| p.0 ) )
        .into_iter()
        .enumerate()
        .map( |(place, player)| player.1 as u32 * (place as u32 + 1) )
        .sum::<u32>()
        .pipe( |result| Ok(Cow::Owned(result.to_string())) )
}

inventory::submit! { Puzzle::new(2023, 7, 1, part1) }
inventory::submit! { Puzzle::new(2023, 7, 2, part2) }

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    const INPUT1: &str = indoc! {"
        32T3K 765
        T55J5 684
        KK677 28
        KTJJT 220
        QQQJA 483
    "};
    const OUTPUT1: &str = "6440";

    const INPUT2: &str = indoc! {"
        32T3K 765
        T55J5 684
        KK677 28
        KTJJT 220
        QQQJA 483
    "};
    const OUTPUT2: &str = "5905";

    #[test]
    fn test1() {
        assert_eq!(OUTPUT1, &part1(INPUT1).unwrap());
    }

    #[test]
    fn test2() {
        assert_eq!(OUTPUT2, &part2(INPUT2).unwrap());
    }
}