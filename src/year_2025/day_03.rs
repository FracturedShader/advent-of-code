use std::{collections::BTreeSet, io::BufRead, str::FromStr, usize};

use crate::common;

/// A battery bank, as described in the puzzle input, but internally stores the bank in a more
/// efficient format for determining largest joltage.
struct BatteryBank(Vec<(u64, usize)>);

impl BatteryBank {
    /// Determines the largest joltage possible when selecting `max_batteries` to turn on. If the
    /// bank is not large enough to support `max_batteries`, the whole bank is turned on.
    pub fn largest_joltage(&self, max_batteries: usize) -> u64 {
        if max_batteries >= self.0.len() {
            let mut entries = self.0.clone();

            entries.sort_by(|lhs, rhs| lhs.1.cmp(&rhs.1));

            return entries
                .into_iter()
                .map(|(v, _)| v)
                .reduce(|acc, v| acc * 10 + v)
                .unwrap_or(0);
        }

        let mut selected = Vec::<(u64, usize)>::with_capacity(max_batteries);

        // Every selection must at least leave room for the selections that remain
        for max_index in (self.0.len() - max_batteries)..self.0.len() {
            // Every selection must start after the most recent selection (or at 0)
            let min_index = selected.last().map(|(_, i)| *i + 1).unwrap_or(0);

            // The highest number at the lowest index within the range is always the best choice
            selected.push(
                self.0
                    .iter()
                    .filter(|(_, i)| *i >= min_index && *i <= max_index)
                    .copied()
                    .next()
                    .unwrap(), // Safety: always at least one option by construction
            );
        }

        // Once all the digits have been selected, build up the total joltage
        selected
            .into_iter()
            .map(|(v, _)| v)
            .reduce(|acc, v| acc * 10 + v)
            .unwrap_or(0) // Unreachable
    }
}

impl FromStr for BatteryBank {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Extract all the numbers and sort in descending order of digit followed by ascending
        // order of index
        let values: BTreeSet<(std::cmp::Reverse<u64>, usize)> = s
            .chars()
            .enumerate()
            .map(|(i, c)| match c {
                '0'..='9' => Ok((std::cmp::Reverse(u64::from(c) - u64::from('0')), i)),
                _ => Err("All characters must be in the inclusive range [0, 9]"),
            })
            .collect::<Result<_, _>>()?;

        Ok(Self(values.into_iter().map(|(r, i)| (r.0, i)).collect()))
    }
}

pub fn part_01() {
    let total: u64 = common::puzzle_input("2025-03")
        .unwrap()
        .lines()
        .map_while(Result::ok)
        .map(|s| s.parse::<BatteryBank>().unwrap().largest_joltage(2))
        .sum();

    println!("Total output joltage: {total}");
}

pub fn part_02() {
    let total: u64 = common::puzzle_input("2025-03")
        .unwrap()
        .lines()
        .map_while(Result::ok)
        .map(|s| s.parse::<BatteryBank>().unwrap().largest_joltage(12))
        .sum();

    println!("Total output joltage: {total}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn data() -> &'static str {
        r#"987654321111111
811111111111119
234234234234278
818181911112111"#
    }

    #[test]
    fn two_batteries() {
        let joltages: Vec<_> = data()
            .lines()
            .map(|s| s.parse::<BatteryBank>().unwrap().largest_joltage(2))
            .collect();

        let expected = vec![98, 89, 78, 92];

        assert_eq!(joltages, expected);
    }

    #[test]
    fn twelve_batteries() {
        let joltages: Vec<_> = data()
            .lines()
            .map(|s| s.parse::<BatteryBank>().unwrap().largest_joltage(12))
            .collect();

        let expected = vec![987654321111, 811111111119, 434234234278, 888911112111];

        assert_eq!(joltages, expected);
    }
}
