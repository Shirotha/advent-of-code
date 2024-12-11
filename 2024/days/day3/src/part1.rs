use advent_of_code::*;
use day3::*;

fn solve(input: Input) -> DResult<impl ToString> {
    Ok(input
        .instructions
        .into_iter()
        .filter_map(Instruction::into_mul)
        .map(|(a, b)| a * b)
        .sum::<u32>())
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

    const RESULT: &str = "161";

    #[test]
    fn test() -> DResult<()> {
        let input = include_str!("../data/example1.dat");
        let input = input.parse::<Input>()?;
        let result = solve(input)?;
        assert_eq!(result.to_string(), RESULT);
        Ok(())
    }
}
