use std::{io::BufRead, ops::RangeInclusive};

use crate::common;

/// Pulls lines from `lines` until the first empty line is reached. The pulled lines are parsed
/// into the lower and upper bounds of the range. Overlapping ranges are then merged together to
/// produce a pairwise disjoint set of ranges.
fn pull_merged_ranges<I, S>(lines: I) -> Vec<RangeInclusive<u64>>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    static FORMAT_ERR: &str = "Lines should be two hyphen-spearated numbers";

    let mut ranges = Vec::new();

    for l in lines {
        let l = l.as_ref();

        if l.is_empty() {
            break;
        }

        let mut parts = l.split('-');
        let lower: u64 = parts.next().expect(FORMAT_ERR).parse().expect(FORMAT_ERR);
        let upper: u64 = parts.next().expect(FORMAT_ERR).parse().expect(FORMAT_ERR);

        ranges.push((lower, upper));
    }

    ranges.sort_unstable();

    let mut merged = Vec::new();
    let mut active: Option<(u64, u64)> = None;

    for (l, u) in ranges {
        if let Some((al, au)) = active {
            if l <= au + 1 {
                active = Some((al, au.max(u)));
            } else {
                merged.push(al..=au);
                active = Some((l, u));
            }
        } else {
            active = Some((l, u));
        }
    }

    merged.extend(active.map(|(l, u)| l..=u));

    merged
}

/// Parses each line in `lines` as a number and counts how many are in any range in `ranges`.
fn ids_in_ranges<I, S>(lines: I, ranges: &[RangeInclusive<u64>]) -> usize
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    lines
        .filter(|l| {
            ranges
                .iter()
                .any(|r| r.contains(&l.as_ref().parse::<u64>().expect("Lines should be numbers")))
        })
        .count()
}

/// Counts the total number of IDs covered by the pairwise disjoint set of `ranges`.
fn ids_covered(ranges: &[RangeInclusive<u64>]) -> u64 {
    ranges
        .iter()
        .fold(0, |acc, r| acc + (r.end() + 1 - r.start()))
}

pub fn part_01() {
    let mut lines = common::puzzle_input("2025-05")
        .unwrap()
        .lines()
        .map_while(Result::ok);
    let ranges = pull_merged_ranges(&mut lines);
    let count = ids_in_ranges(lines, &ranges);

    println!("Number of fresh ingredients: {count}");
}

pub fn part_02() {
    let mut lines = common::puzzle_input("2025-05")
        .unwrap()
        .lines()
        .map_while(Result::ok);
    let ranges = pull_merged_ranges(&mut lines);
    let total_covered = ids_covered(&ranges);

    println!("Total IDs covered: {total_covered}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn data() -> &'static str {
        r#"3-5
10-14
16-20
12-18

1
5
8
11
17
32"#
    }

    #[test]
    fn freshness() {
        let mut lines = data().lines();
        let ranges = pull_merged_ranges(&mut lines);
        let count = ids_in_ranges(lines, &ranges);

        assert_eq!(count, 3);
    }

    #[test]
    fn all_fresh() {
        let lines = data().lines();
        let ranges = pull_merged_ranges(lines);

        assert_eq!(ranges, vec![3..=5, 10..=20]);

        let total_covered = ids_covered(&ranges);

        assert_eq!(total_covered, 14);
    }
}
