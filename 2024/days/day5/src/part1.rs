#![feature(array_windows)]

use advent_of_code::*;
use day5::*;

/* NOTE: this solution assumes that all rules are always active, which is not the case here
const FORWARD_OFFSET: u8 = 0;
const FORWARD_MASK: u8 = 1 << FORWARD_OFFSET;
const BACKWARDS_OFFSET: u8 = 1;
const BACKWARDS_MASK: u8 = 1 << BACKWARDS_OFFSET;

const CONNECTED_OFFSET: u8 = 0;
const CONNECTED_MASK: u8 = 1 << CONNECTED_OFFSET;
const MARK_OFFSET: u8 = 1;
const MARK_MASK: u8 = 1 << MARK_OFFSET;
const CLOSED_OFFSET: u8 = 2;
const CLOSED_MASK: u8 = 1 << CLOSED_OFFSET;

fn solve(input: Input) -> DResult<impl ToString> {
    fn visit(
        rules: &[Box<[u8]>],
        connections: &mut NArray<2, Box<[u8]>>,
        row: &mut [u8],
        page: usize,
    ) {
        if row[page] & MARK_MASK == 0 {
            row[page] |= MARK_MASK;
            for child in &rules[page] {
                let child = *child as usize;
                visit(rules, connections, row, child);
                row[child] |= CONNECTED_MASK;
                // SAFETY: unwrap: page is a valid index
                for ([i], x) in &mut view_mut!(connections, .., page).unwrap() {
                    let connected = row[i] & CONNECTED_MASK >> CONNECTED_OFFSET;
                    *x = connected << FORWARD_OFFSET | *x & !FORWARD_MASK;
                }
                // SAFETY: unwrap: page is a valid index
                for ([i], x) in &mut view_mut!(connections, page, ..).unwrap() {
                    let connected = row[i] & CONNECTED_MASK >> CONNECTED_OFFSET;
                    *x = connected << BACKWARDS_OFFSET | *x & !BACKWARDS_MASK;
                }
            }
        } else if row[page] & CLOSED_MASK == 0 {
            panic!("cycle detected!");
        }
        for (i, x) in row.iter_mut().enumerate() {
            let forward = connections[[i, page]] & FORWARD_MASK >> FORWARD_OFFSET;
            *x = forward << CONNECTED_OFFSET | *x & !CONNECTED_MASK;
        }
        row[page] |= CLOSED_MASK;
    }
    let mut connections = NArray::<2, Box<[u8]>>::new([PAGE_COUNT; 2]);
    let mut row = vec![0; PAGE_COUNT].into_boxed_slice();
    for page in 0..PAGE_COUNT {
        if row[page] & MARK_MASK == 0 {
            visit(&input.rules, &mut connections, &mut row, page);
        }
    }

    let mut result = 0;
    let mut open = Vec::new();
    'order: for order in input.orders {
        open.clear();
        // SAFETY: unwrap: order is not empty by construction
        open.push(*order.last().unwrap() as usize);
        for page in order.iter().rev().skip(1) {
            let page = *page as usize;
            let mut valid = true;
            let mut connected = false;
            open.retain(|&open| {
                match connections[[open, page]] {
                    FORWARD_MASK => {
                        connected = true;
                        false
                    }
                    BACKWARDS_MASK => {
                        valid = false;
                        true
                    }
                    0 => true,
                    // SAFETY: graph is acyclic
                    _ => unreachable!("cycle detected"),
                }
            });
            if !valid {
                continue 'order;
            }
            if !connected {
                open.push(page);
            }
        }
        result += FIRST_PAGE + order[order.len() / 2] as usize;
    }
    Ok(result)
} */

fn solve(input: Input) -> DResult<impl ToString> {
    let mut result = 0;
    'order: for order in input.orders {
        for &[from, to] in order.array_windows() {
            if input.rules[to as usize].binary_search(&from).is_ok() {
                continue 'order;
            }
        }
        result += FIRST_PAGE + order[order.len() / 2] as usize;
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

    const RESULT: &str = "143";

    #[test]
    fn test() -> DResult<()> {
        let input = include_str!("../data/example.dat");
        let input = input.parse::<Input>()?;
        let result = solve(input)?;
        assert_eq!(result.to_string(), RESULT);
        Ok(())
    }
}
