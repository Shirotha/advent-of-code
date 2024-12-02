use advent_of_code::*;
use day1::*;

fn solve(mut input: Input) -> DResult<String> {
    input.right.sort_unstable();
    let len = input.right.len();
    let mut sum = 0;
    for n in &input.left {
        if let Ok(i) = input.right.binary_search(n) {
            let left = input
                .right
                .iter()
                .rev()
                .skip(len - i)
                .take_while(|x| *x == n)
                .count() as u32;
            let right = input.right.iter().skip(i).take_while(|x| *x == n).count() as u32;
            sum += *n * (left + right);
        }
    }
    Ok(sum.to_string())
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
        assert_eq!(result, "31");
        Ok(())
    }
}
