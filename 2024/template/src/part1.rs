use advent_of_code::*;
use {{crate_name}}::*;

fn solve(input: Input) -> DResult<String> {
    todo!()
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
        assert_eq!(result, "");
        Ok(())
    }
}
