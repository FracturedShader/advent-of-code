use std::{collections::BTreeSet, io::BufRead};

use crate::common;

/// Finds sequences of repeated numbers within the range `r`. Can be limited to only finding only
/// repetitions of length two by setting `halves_only` to `true`.
fn repeated_digits_in_range(r: &(&str, &str), halves_only: bool) -> Vec<u64> {
    let (lower_str, upper_str) = r;
    let lower: u64 = lower_str.parse().unwrap();
    let upper: u64 = upper_str.parse().unwrap();
    let (min_len, max_len) = (lower_str.len(), upper_str.len());

    // When only two repetitions are allowed, minimum length must be at least half the lower bound
    let min_part_len = if halves_only { min_len / 2 } else { 1 };
    let max_part_len = max_len / 2;

    let mut found = BTreeSet::new();

    for part_len in min_part_len..=max_part_len {
        let (min_reps, max_reps) = if halves_only {
            (2, 2)
        } else {
            // Number of digits must be at least as long as the lower bound
            let min_reps = if min_len.rem_euclid(part_len) == 0 {
                min_len / part_len
            } else {
                min_len / part_len + 1
            }
            .max(2); // Must also occur at least twice

            (min_reps, max_len / part_len)
        };

        // If no amount of repetitions is the right length, skip
        if part_len * min_reps > max_len {
            continue;
        }

        let mult = 10u64.pow(part_len.try_into().unwrap());

        // If an order of magnitude is bridged, all numbers are an option
        let mut base = if max_len > min_len {
            mult / 10
        } else {
            lower_str[..part_len].parse().unwrap()
        };

        loop {
            let mut got_too_large = true;

            for reps in min_reps..=max_reps {
                let mut candidate = base;

                for _ in 1..reps {
                    candidate = candidate * mult + base;
                }

                if candidate <= upper {
                    got_too_large = false;

                    if candidate >= lower {
                        found.insert(candidate);
                    }
                }
            }

            if got_too_large {
                break;
            }

            base += 1;

            // If the next number isn't the same number of digits, move on
            if base / mult != 0 {
                break;
            }
        }
    }

    found.into_iter().collect()
}

/// Splits a comma-separated list of hyphen-separated numbers apart. Returned values are references
/// into `l`.
fn to_range_strings(l: &str) -> Vec<(&str, &str)> {
    static ERR_STR: &str = "Comma-separated entries should have one hyphen-separated pair";

    l.split(',')
        .map(|r| {
            let mut parts = r.split('-');

            (parts.next().expect(ERR_STR), parts.next().expect(ERR_STR))
        })
        .collect()
}

pub fn part_01() {
    let first_line = &common::puzzle_input("2025-02")
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .unwrap();

    let total: u64 = to_range_strings(&first_line)
        .iter()
        .map(|r| repeated_digits_in_range(r, true))
        .flatten()
        .sum();

    println!("Sum of found doubles: {total}");
}

pub fn part_02() {
    let first_line = &common::puzzle_input("2025-02")
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .unwrap();

    let total: u64 = to_range_strings(&first_line)
        .iter()
        .map(|r| repeated_digits_in_range(r, false))
        .flatten()
        .sum();

    println!("Sum of found repeated: {total}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn data() -> &'static str {
        "11-22,95-115,998-1012,1188511880-1188511890,222220-222224,1698522-1698528,446443-446449,\
            38593856-38593862,565653-565659,824824821-824824827,2121212118-2121212124"
    }

    #[test]
    fn matching_halves() {
        let doubled: Vec<_> = to_range_strings(data())
            .iter()
            .map(|r| repeated_digits_in_range(r, true))
            .collect();

        let expected = vec![
            vec![11, 22],
            vec![99],
            vec![1010],
            vec![1188511885],
            vec![222222],
            vec![],
            vec![446446],
            vec![38593859],
            vec![],
            vec![],
            vec![],
        ];

        assert_eq!(doubled, expected);
    }

    #[test]
    fn repeated() {
        let found: Vec<_> = to_range_strings(data())
            .iter()
            .map(|r| repeated_digits_in_range(r, false))
            .collect();

        let expected = vec![
            vec![11, 22],
            vec![99, 111],
            vec![999, 1010],
            vec![1188511885],
            vec![222222],
            Default::default(),
            vec![446446],
            vec![38593859],
            vec![565656],
            vec![824824824],
            vec![2121212121],
        ];

        assert_eq!(found, expected);
    }
}
