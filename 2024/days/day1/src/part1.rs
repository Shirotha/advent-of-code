use advent_of_code::*;
use day1::*;

fn solve(mut input: Input) -> DResult<String> {
    input.left.sort_unstable();
    input.right.sort_unstable();
    let result = input
        .left
        .into_iter()
        .zip(input.right)
        .fold(0u32, |sum, (left, right)| sum + left.abs_diff(right));
    Ok(result.to_string())
}

pub fn main() -> DResult<()> {
    let input = get_input()?;
    let input = input.parse::<Input>()?;
    let solution = solve(input)?;
    println!("Solution: {}", solution);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() -> DResult<()> {
        let input = include_str!("../data/example.dat");
        let input = input.parse::<Input>()?;
        let result = solve(input)?;
        assert_eq!(result, "11");
        Ok(())
    }
}
