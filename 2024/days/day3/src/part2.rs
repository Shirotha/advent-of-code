use advent_of_code::*;
use day3::*;

fn solve(input: Input) -> DResult<impl ToString> {
    let mut sum = 0;
    let mut enabled = true;
    for instruction in input.instructions {
        match instruction {
            Instruction::Mul(a, b) if enabled => sum += a * b,
            Instruction::Do => enabled = true,
            Instruction::Dont => enabled = false,
            _ => (),
        }
    }
    Ok(sum)
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

    const RESULT: &str = "48";

    #[test]
    fn test() -> DResult<()> {
        let input = include_str!("../data/example2.dat");
        let input = input.parse::<Input>()?;
        let result = solve(input)?;
        assert_eq!(result.to_string(), RESULT);
        Ok(())
    }
}
