#![feature(array_windows)]

use std::cmp::Ordering;

use advent_of_code::*;
use day2::*;

fn solve(input: Input) -> DResult<impl ToString> {
    let mut count = 0;
    for line in input.lines {
        assert!(line.len() >= 2);
        let valid = match line[1].cmp(&line[0]) {
            Ordering::Equal => continue,
            Ordering::Less => -3..=-1,
            Ordering::Greater => 1..=3,
        };
        if line
            .array_windows::<2>()
            .all(|pair| valid.contains(&(pair[1] - pair[0])))
        {
            count += 1;
        }
    }
    Ok(count)
}

pub fn main() -> DResult<()> {
    let input = get_input()?;
    let input = input.parse::<Input>()?;
    let solution = solve(input)?;
    println!("Solution: {}", solution.to_string());
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    const RESULT: &str = "2";

    #[test]
    fn test() -> DResult<()> {
        let input = include_str!("../data/example.dat");
        let input = input.parse::<Input>()?;
        let result = solve(input)?;
        assert_eq!(result.to_string(), RESULT);
        Ok(())
    }
}
