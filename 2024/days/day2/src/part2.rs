#![feature(array_windows)]

use std::cmp::Ordering;

use advent_of_code::*;
use day2::*;

fn solve(input: Input) -> DResult<impl ToString> {
    let mut count = 0;
    for line in input.lines {
        assert!(line.len() >= 2);
        // SAFETY: unwrap: assert guaranties that line is not empty
        let valid = match unsafe {
            line.last()
                .unwrap_unchecked()
                .cmp(line.first().unwrap_unchecked())
        } {
            Ordering::Equal => continue,
            Ordering::Less => -3..=-1,
            Ordering::Greater => 1..=3,
        };
        let is_valid = |pair: &[i32; 2]| valid.contains(&(pair[1] - pair[0]));
        if let Some((i, &[first, second])) = line
            .array_windows()
            .enumerate()
            .find(|(_, pair)| !is_valid(pair))
        {
            if !((i == 0 || is_valid(&[line[i - 1], second]))
                && line[i + 1..].array_windows().all(is_valid)
                || (i == line.len() - 2 || is_valid(&[first, line[i + 2]]))
                    && line[i + 2..].array_windows().all(is_valid))
            {
                continue;
            }
        }
        count += 1;
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

    const RESULT: &str = "4";

    #[test]
    fn test() -> DResult<()> {
        let input = include_str!("../data/example.dat");
        let input = input.parse::<Input>()?;
        let result = solve(input)?;
        assert_eq!(result.to_string(), RESULT);
        Ok(())
    }
}
