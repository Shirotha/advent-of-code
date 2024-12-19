use advent_of_code::*;
use day4::*;

fn solve(input: Input) -> DResult<impl ToString> {
    let size = *input.data.size();
    let mut matches = NArray::<2, Box<[u32]>>::new(size);
    let mut update = |pos: [usize; 2], dir: Dir, index: usize| {
        // SAFETY: assumes that other_pos is in-bounds
        let other_pos = shift(pos, size, dir, 1).unwrap();
        let Some(other) = linear_search(&WORD, input.data[other_pos]) else {
            return;
        };
        if index + 1 == other {
            if let Some(root) = shift(pos, size, dir.invert(), index) {
                matches[root] |= 1 << (dir.offset() + index as u32);
            }
        } else if index == other + 1 {
            if let Some(root) = shift(pos, size, dir, index) {
                matches[root] |= 1 << (dir.invert().offset() + other as u32);
            }
        }
    };
    println!(
        "size: {:?}, stride: {:?}",
        input.data.size(),
        input.data.stride()
    );
    for (pos, &char) in &input.data {
        let Some(index) = linear_search(&WORD, char) else {
            continue;
        };
        if pos[0] != 0 {
            update(pos, Dir::Left, index);
        }
        if pos[1] != 0 {
            if pos[0] != 0 {
                update(pos, Dir::TopLeft, index);
            }
            update(pos, Dir::Top, index);
            if pos[0] + 1 != size[0] {
                update(pos, Dir::TopRight, index);
            }
        }
    }
    let mut sum = 0;
    for (_, r#match) in &matches {
        sum += MASK.iter().filter(|&m| r#match & m == *m).count()
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

    const RESULT: &str = "18";

    #[test]
    fn test() -> DResult<()> {
        let input = include_str!("../data/example.dat");
        let input = input.parse::<Input>()?;
        let result = solve(input)?;
        assert_eq!(result.to_string(), RESULT);
        Ok(())
    }
}
