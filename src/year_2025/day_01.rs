use std::{io::BufRead, str::FromStr};

use crate::common;

/// Parsed puzzle input which represents spinning the safe dial a certain number of ticks
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Spin {
    Left(i32),
    Right(i32),
}

impl Spin {
    /// Applies the [[Spin]] to the current position of the dial (`v`)
    fn apply(self, v: i32) -> i32 {
        match self {
            Spin::Left(n) => v - n,
            Spin::Right(n) => v + n,
        }
    }
}

impl FromStr for Spin {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (dir, amt) = s.split_at(1);

        if let Ok(amt) = amt.parse::<i32>() {
            match dir {
                "L" => Ok(Spin::Left(amt)),
                "R" => Ok(Spin::Right(amt)),
                _ => Err("First character must be 'L' or 'R'"),
            }
        } else {
            Err("String after first character is not a number")
        }
    }
}

/// Count the number of times applying the [[Spin]], parsed from each line in `lines`, lands on 0
fn count_stop_at_zero<I, S>(lines: I) -> i32
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let mut pos = 50;
    let mut zero_count = 0;

    for l in lines {
        pos = l
            .as_ref()
            .parse::<Spin>()
            .unwrap()
            .apply(pos)
            .rem_euclid(100);

        if pos == 0 {
            zero_count += 1;
        }
    }

    zero_count
}

/// Count the number of times applying the [[Spin]], parsed from each line in `lines`, passes over
/// or lands on 0
fn count_hit_zero<I, S>(lines: I) -> i32
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let mut pos = 50;
    let mut zero_count = 0;

    for l in lines {
        match l.as_ref().parse::<Spin>().unwrap() {
            Spin::Left(n) => {
                let next = pos - n;
                let corrections = -next.div_euclid(100);

                if corrections != 0 {
                    zero_count += corrections;

                    // Don't count passing zero if starting at zero
                    if pos == 0 {
                        zero_count -= 1;
                    }
                }

                pos = next.rem_euclid(100);

                // Count ending at zero
                if pos == 0 {
                    zero_count += 1;
                }
            }
            Spin::Right(n) => {
                let next = pos + n;

                // Only way to get to zero is to wrap
                zero_count += next.div_euclid(100);

                pos = next.rem_euclid(100);
            }
        }
    }

    zero_count
}

pub fn part_01() {
    let zero_count = count_stop_at_zero(
        common::puzzle_input("2025-01")
            .unwrap()
            .lines()
            .map_while(Result::ok),
    );

    println!("Times stopping at zero: {zero_count}");
}

pub fn part_02() {
    let zero_count = count_hit_zero(
        common::puzzle_input("2025-01")
            .unwrap()
            .lines()
            .map_while(Result::ok),
    );

    println!("Times hitting zero: {zero_count}");
}

#[cfg(test)]
mod test {
    use crate::year_2025::day_01::*;

    fn test_data() -> &'static str {
        "L68
L30
R48
L5
R60
L55
L1
L99
R14
L82"
    }

    #[test]
    fn at_zero() {
        let count = count_stop_at_zero(test_data().lines());

        assert_eq!(count, 3);
    }

    #[test]
    fn pass_zero() {
        let count = count_hit_zero(test_data().lines());

        assert_eq!(count, 6);
    }
}
