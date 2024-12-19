#![feature(array_windows)]

use advent_of_code::*;
use day5::*;

fn solve(input: Input) -> DResult<impl ToString> {
    let sort = |order: &mut [_]| {
        let mut was_sorted = true;
        for front in 0..order.len() - 1 {
            let target = order[front + 1] as usize;
            if input.rules[target].binary_search(&order[front]).is_ok() {
                was_sorted = false;
                order.swap(front, front + 1);
                for backtrack in (0..front).rev() {
                    if input.rules[target].binary_search(&order[backtrack]).is_ok() {
                        order.swap(backtrack, backtrack + 1);
                    } else {
                        break;
                    }
                }
            }
        }
        was_sorted
    };
    let mut result = 0;
    for mut order in input.orders {
        if !sort(&mut order) {
            result += FIRST_PAGE + order[order.len() / 2] as usize;
        }
    }
    Ok(result)
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

    const RESULT: &str = "123";

    #[test]
    fn test() -> DResult<()> {
        let input = include_str!("../data/example.dat");
        let input = input.parse::<Input>()?;
        let result = solve(input)?;
        assert_eq!(result.to_string(), RESULT);
        Ok(())
    }
}
