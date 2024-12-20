use advent_of_code::*;
use day4::*;

fn solve(input: Input) -> DResult<impl ToString> {
    fn invert(char: u8) -> Option<u8> {
        match char {
            b'M' => Some(b'S'),
            b'S' => Some(b'M'),
            _ => None,
        }
    }
    let size = *input.data.size();
    let check = |pos, dir| -> bool {
        // SAFETY: assume: pos is not on the border
        let corner = shift(pos, size, dir, 1).unwrap();
        let Some(expected) = invert(input.data[corner]) else {
            return false;
        };
        let other_corner = shift(pos, size, dir.invert(), 1).unwrap();
        input.data[other_corner] == expected
    };
    let mut sum = 0;
    for (pos, char) in &input.data {
        if *char != b'A'
            || pos[0] == 0
            || pos[1] == 0
            || pos[0] + 1 == size[0]
            || pos[1] + 1 == size[1]
        {
            continue;
        }
        if check(pos, Dir::TopLeft) && check(pos, Dir::TopRight) {
            sum += 1;
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

    const RESULT: &str = "9";

    #[test]
    fn test() -> DResult<()> {
        let input = include_str!("../data/example.dat");
        let input = input.parse::<Input>()?;
        let result = solve(input)?;
        assert_eq!(result.to_string(), RESULT);
        Ok(())
    }
}
