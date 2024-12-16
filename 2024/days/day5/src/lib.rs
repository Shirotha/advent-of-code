use core::ops::RangeInclusive;
use std::str::FromStr;

use nom::{
    IResult,
    bytes::complete::take_while_m_n,
    character::{
        complete::{char, line_ending, multispace0},
        is_digit,
    },
    combinator::{map, map_res},
    multi::separated_list1,
    sequence::{preceded, separated_pair},
};

pub const FIRST_PAGE: u8 = 11;
pub const LAST_PAGE: u8 = 99;
pub const PAGES: RangeInclusive<u8> = FIRST_PAGE..=LAST_PAGE;
pub const PAGE_COUNT: u8 = LAST_PAGE - FIRST_PAGE + 1;

#[derive(Debug)]
pub struct Input {
    pub rules: [Box<[Box<[u8]>]>; 2],
    pub orders: Box<[Box<[u8]>]>,
}
impl FromStr for Input {
    type Err = nom::Err<nom::error::Error<String>>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn page(input: &str) -> IResult<&str, u8> {
            map_res(take_while_m_n(2, 2, |c| is_digit(c as u8)), |n: &str| {
                n.parse::<u8>()
            })(input)
        }
        fn rule(input: &str) -> IResult<&str, (u8, u8)> {
            separated_pair(page, char('|'), page)(input)
        }
        fn order(input: &str) -> IResult<&str, Box<[u8]>> {
            map(separated_list1(char(','), page), |v| v.into_boxed_slice())(input)
        }
        let (s, rules) = separated_list1(line_ending, rule)(s).map_err(|e| e.to_owned())?;
        let (_, orders) = preceded(multispace0, separated_list1(line_ending, order))(s)
            .map_err(|e| e.to_owned())?;
        let mut forward_rules = (0..PAGE_COUNT).map(|_| Vec::new()).collect::<Box<_>>();
        let mut backwards_rules = (0..PAGE_COUNT).map(|_| Vec::new()).collect::<Box<_>>();
        for (from, to) in rules {
            assert!(PAGES.contains(&from));
            assert!(PAGES.contains(&to));
            forward_rules[from as usize].push(to);
            backwards_rules[to as usize].push(from);
        }
        Ok(Input {
            rules: [
                forward_rules
                    .into_iter()
                    .map(Vec::into_boxed_slice)
                    .collect(),
                backwards_rules
                    .into_iter()
                    .map(Vec::into_boxed_slice)
                    .collect(),
            ],
            orders: orders.into_boxed_slice(),
        })
    }
}
