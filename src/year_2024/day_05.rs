use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
};

use nom::{character, sequence, IResult};

use crate::common;

/// An `Ordering` is a pair of numbers indicating a `(before, after)` relationship.
type Ordering = (i32, i32);

/// Helper struct to keep multiple updates close together in memory (compared to `Vec<Vec<_>>`).
/// Lookup table is expected to be small enough to avoid cache eviction.
#[derive(Debug, Default)]
struct Updates {
    ranges: Vec<(usize, usize)>,
    data: Vec<i32>,
}

/// Helper struct to iterate over each separate update (list of pages) within `Updates`.
struct UpdateIterator<'updates> {
    idx: usize,
    updates: &'updates Updates,
}

impl<'updates> Iterator for UpdateIterator<'updates> {
    type Item = &'updates [i32];

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.updates.ranges.len() {
            let idx = self.idx;

            self.idx += 1;

            Some(self.updates.get(idx))
        } else {
            None
        }
    }
}

impl Updates {
    /// Returns a reference to a particular update (list of pages) by index.
    pub fn get(&self, update: usize) -> &[i32] {
        let (begin, end) = self.ranges[update];

        &self.data[begin..end]
    }

    /// Extend this collection of updates with another one.
    pub fn add_update<T: IntoIterator<Item = i32>>(&mut self, iter: T) {
        let range_begin = self.data.len();

        self.data.extend(iter);

        let range_end = self.data.len();

        self.ranges.push((range_begin, range_end));
    }

    pub fn iter(&self) -> UpdateIterator<'_> {
        UpdateIterator {
            idx: 0,
            updates: self,
        }
    }
}

/// Converts an `Ordering` slice into a `HashMap<i32, must_not_appear_after>`.
fn orderings_to_deny_lists(orderings: &[Ordering]) -> HashMap<i32, Vec<i32>> {
    let mut deny_lists: HashMap<_, Vec<_>> = HashMap::new();

    for &(before, after) in orderings {
        deny_lists.entry(after).or_default().push(before);
    }

    deny_lists
}

/// Determines if an `update` is ordered so as not to violate any `deny_lists`.
fn is_ordered(update: &[i32], deny_lists: &HashMap<i32, Vec<i32>>) -> bool {
    let mut deny: HashSet<i32> = HashSet::new();

    for page in update {
        if deny.contains(page) {
            return false;
        }

        if let Some(deny_list) = deny_lists.get(page) {
            deny.extend(deny_list);
        }
    }

    true
}

/// Produces a new `update` that honors all of the provided ordering rules. Requires that applying
/// `orderings` (directed edges) to `update` produces a Directed Acyclic Graph (DAG). If more than
/// one topological ordering is possible, ties are resolved by `update` entry order.
fn order_update(update: &[i32], orderings: &[Ordering]) -> Vec<i32> {
    let update: HashSet<_> = update.iter().copied().collect();
    let orderings = orderings
        .iter()
        .filter(|(b, a)| update.contains(b) && update.contains(a))
        .copied()
        .collect::<Vec<_>>();

    let mut out_edges: HashMap<_, Vec<_>> = HashMap::new();
    let mut in_edges = update.iter().map(|&p| (p, 0)).collect::<HashMap<_, _>>();

    for &(before, after) in &orderings {
        out_edges.entry(before).or_default().push(after);
        *in_edges.entry(after).or_default() += 1;
    }

    let mut ordered = Vec::new();
    let mut queue = update.into_iter().collect::<Vec<_>>();

    while !queue.is_empty() {
        queue.sort_by(|l, r| (in_edges.get(l).unwrap(), l).cmp(&(in_edges.get(r).unwrap(), r)));

        let p = queue.swap_remove(0);

        assert_eq!(*in_edges.get(&p).unwrap(), 0);

        if let Some(edges) = out_edges.get(&p) {
            for a in edges {
                *in_edges.get_mut(a).unwrap() -= 1;
            }
        }

        ordered.push(p);
    }

    ordered
}

/// Parses an "`i32`|`i32`" line into an `Ordering`
fn parse_ordering(input: &[u8]) -> IResult<&[u8], Ordering> {
    sequence::separated_pair(
        character::complete::i32::<&[u8], nom::error::Error<_>>,
        character::complete::char('|'),
        character::complete::i32,
    )(input)
}

/// Tries to parse file input into ordering rules and updates.
fn parse_input<R>(reader: R) -> Result<(Vec<Ordering>, Updates), &'static str>
where
    R: BufRead,
{
    let mut lines = reader.lines().map_while(Result::ok);

    let mut orderings = Vec::with_capacity(256);

    for line in lines.by_ref().take_while(|l| !l.is_empty()) {
        if let Ok((_, o)) = parse_ordering(line.as_bytes()) {
            orderings.push(o);
        } else {
            return Err("Lines at the beginning of the input should be of the form \"i32|i32\"");
        }
    }

    let mut updates = Updates::default();

    for line in lines.by_ref().take_while(|l| !l.is_empty()) {
        updates.add_update(line.split(',').map(str::parse::<i32>).map_while(Result::ok));
    }

    Ok((orderings, updates))
}

pub fn part_01() {
    let (orderings, updates) = parse_input(common::puzzle_input("2024-05").unwrap()).unwrap();

    let deny_lists = orderings_to_deny_lists(&orderings);

    let midpoint_sum = updates
        .iter()
        .filter(|u| is_ordered(u, &deny_lists))
        .map(|u| u[u.len() / 2])
        .sum::<i32>();

    println!("Sum of ordered midpoints: {midpoint_sum}");
}

pub fn part_02() {
    let (orderings, updates) = parse_input(common::puzzle_input("2024-05").unwrap()).unwrap();

    let deny_lists = orderings_to_deny_lists(&orderings);

    let midpoint_sum = updates
        .iter()
        .filter(|u| !is_ordered(u, &deny_lists))
        .map(|u| {
            let ordered = order_update(u, &orderings);

            ordered[ordered.len() / 2]
        })
        .sum::<i32>();

    println!("Sum of unordered midpoints: {midpoint_sum}");
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use super::*;

    fn parsed_test_input() -> (Vec<Ordering>, Updates) {
        let input = r"47|53
97|13
97|61
97|47
75|29
61|13
75|53
29|13
97|29
53|29
61|53
97|53
61|29
47|13
75|47
97|75
47|61
75|61
47|29
75|13
53|13

75,47,61,53,29
97,61,53,29,13
75,29,13
75,97,47,61,53
61,13,29
97,13,75,29,47";

        parse_input(BufReader::new(input.as_bytes())).unwrap()
    }

    #[test]
    fn valid_ordering() {
        let (orderings, updates) = parsed_test_input();
        let deny_lists = orderings_to_deny_lists(&orderings);

        let valid_midpoints = updates
            .iter()
            .filter(|u| is_ordered(u, &deny_lists))
            .map(|u| u[u.len() / 2])
            .collect::<Vec<_>>();

        assert_eq!(valid_midpoints, vec![61, 53, 29]);
    }

    #[test]
    fn corrected_ordering() {
        let (orderings, updates) = parsed_test_input();
        let deny_lists = orderings_to_deny_lists(&orderings);

        let valid_midpoints = updates
            .iter()
            .filter(|u| !is_ordered(u, &deny_lists))
            .map(|u| {
                let ordered = order_update(u, &orderings);

                ordered[ordered.len() / 2]
            })
            .collect::<Vec<_>>();

        assert_eq!(valid_midpoints, vec![47, 29, 47]);
    }
}
