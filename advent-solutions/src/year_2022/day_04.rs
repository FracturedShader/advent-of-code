use std::{fmt::Debug, io::BufRead, str::FromStr};

/// Simple type to capture the start and end of a 1D range
#[derive(Clone, Debug, PartialEq, Eq)]
struct SectionRange<Idx>(Idx, Idx);

impl<Idx> SectionRange<Idx>
where
    Idx: PartialEq + PartialOrd + Copy,
{
    /// Determines if this `SectionRange` inclusively contains the given value
    fn contains(&self, value: Idx) -> bool {
        value >= self.0 && value <= self.1
    }

    /// Determines if this `SectionRange` inclusively contains the other `SectionRange`
    fn contains_range(&self, other: &Self) -> bool {
        self.0 <= other.0 && self.1 >= other.1
    }

    /// Determines if this `SectionRange` inclusively overlaps the other `SectionRange`
    fn overlaps(&self, other: &Self) -> bool {
        self.contains(other.0)
            || self.contains(other.1)
            || other.contains(self.0)
            || other.contains(self.1)
    }
}

/// A `SectionRange` for this problem is represented by two numbers separated with a hyphen
impl<D, I> FromStr for SectionRange<I>
where
    D: Debug,
    I: FromStr<Err = D>,
{
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once('-')
            .map(|(start, end)| {
                SectionRange(
                    start.parse().expect("range start should be a number"),
                    end.parse().expect("range end should be a number"),
                )
            })
            .ok_or("Provided str does not represent a range")
    }
}

/// A convenient iterator adapter to create pairs of `SectionRange`s from lines containing a
/// comma-separated string equivalent
struct RangePairs<S, I>(I)
where
    S: AsRef<str>,
    I: Iterator<Item = S>;

impl<S, I> Iterator for RangePairs<S, I>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    type Item = (SectionRange<u32>, SectionRange<u32>);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|l| {
            l.as_ref()
                .split_once(',')
                .map(|(l, r)| {
                    (
                        l.parse()
                            .expect("first part of line should represent a range"),
                        r.parse()
                            .expect("second part of line should represent a range"),
                    )
                })
                .expect("line should be two ranges separated by a comma")
        })
    }
}

/// Helper trait to add the method `as_range_pairs` to any `Iterator` that it applies to. This
/// method converts the `Iterator` into one that outputs pairs of `SectionRange`s instead.
trait IntoRangePairs<S: AsRef<str>>: Iterator<Item = S> {
    fn range_pairs(self) -> RangePairs<S, Self>
    where
        Self: Sized,
    {
        RangePairs(self)
    }
}

impl<S: AsRef<str>, T: Sized> IntoRangePairs<S> for T where T: Iterator<Item = S> {}

pub fn part_01(reader: Option<impl BufRead>) {
    let contain_count: u32 = reader
        .expect("data should be available for this problem")
        .lines()
        .flatten()
        .range_pairs()
        .map(|(l, r)| u32::from(l.contains_range(&r) || r.contains_range(&l)))
        .sum();

    println!(
        "Number of assignment pairs where one fully contains the other: {contain_count}"
    );
}

pub fn part_02(reader: Option<impl BufRead>) {
    let overlap_count: u32 = reader
        .expect("data should be available for this problem")
        .lines()
        .flatten()
        .range_pairs()
        .map(|(l, r)| u32::from(l.overlaps(&r)))
        .sum();

    println!(
        "Number of assignment pairs where one overlaps the other: {overlap_count}"
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_ranges() {
        let input = r"2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8";

        let ranges = input.lines().range_pairs().collect::<Vec<_>>();

        assert_eq!(
            vec![
                (SectionRange(2, 4), SectionRange(6, 8)),
                (SectionRange(2, 3), SectionRange(4, 5)),
                (SectionRange(5, 7), SectionRange(7, 9)),
                (SectionRange(2, 8), SectionRange(3, 7)),
                (SectionRange(6, 6), SectionRange(4, 6)),
                (SectionRange(2, 6), SectionRange(4, 8))
            ],
            ranges
        );
    }

    #[test]
    fn range_contains() {
        let ranges = vec![
            (SectionRange(2, 4), SectionRange(6, 8)),
            (SectionRange(2, 3), SectionRange(4, 5)),
            (SectionRange(5, 7), SectionRange(7, 9)),
            (SectionRange(2, 8), SectionRange(3, 7)),
            (SectionRange(6, 6), SectionRange(4, 6)),
            (SectionRange(2, 6), SectionRange(4, 8)),
        ];

        let full_overlaps = ranges
            .iter()
            .map(|(l, r)| l.contains_range(r) || r.contains_range(l))
            .collect::<Vec<_>>();

        assert_eq!(vec![false, false, false, true, true, false], full_overlaps);
    }

    #[test]
    fn range_overlaps() {
        let ranges = vec![
            (SectionRange(2, 4), SectionRange(6, 8)),
            (SectionRange(2, 3), SectionRange(4, 5)),
            (SectionRange(5, 7), SectionRange(7, 9)),
            (SectionRange(2, 8), SectionRange(3, 7)),
            (SectionRange(6, 6), SectionRange(4, 6)),
            (SectionRange(2, 6), SectionRange(4, 8)),
        ];

        let partial_overlaps = ranges
            .iter()
            .map(|(l, r)| l.overlaps(r))
            .collect::<Vec<_>>();

        assert_eq!(vec![false, false, true, true, true, true], partial_overlaps);
    }
}
