use std::{
    collections::{HashMap, HashSet},
    io::BufRead,
};

use glam::IVec2;

use crate::common;

/// A sparse map of antennas from the puzsle input. Antennas are grouped by frequency.
#[derive(Debug, Clone)]
struct AntennaMap {
    size: IVec2,
    groupings: HashMap<char, Vec<IVec2>>,
}

impl AntennaMap {
    fn in_bounds(&self, p: IVec2) -> bool {
        (0..self.size.x).contains(&p.x) && (0..self.size.y).contains(&p.y)
    }

    /// Counts the number of unique antinode positions that lie within the bounds of the map.
    /// Antinodes are one equidistant step away from each par of antennas.
    fn count_unique_antinodes(&self) -> usize {
        let mut node_positions = HashSet::new();

        for antennas in self.groupings.values() {
            let n = antennas.len();

            for (i, p0) in antennas.iter().take(n - 1).enumerate() {
                for p1 in &antennas[(i + 1)..] {
                    let v = p1 - p0;

                    for ap in [p0 - v, p1 + v] {
                        if self.in_bounds(ap) {
                            node_positions.insert(ap);
                        }
                    }
                }
            }
        }

        node_positions.len()
    }

    /// Counts the nmuber of unique harmonic antinode positions that lie within the bounds of the
    /// map. For every pair of antennas:
    /// - Each antenna is an antinode
    /// - The difference `d` between the antennas is repeatedly applied in both directions to
    ///   produce a line of antinodes
    fn count_unique_harmonic_antinodes(&self) -> usize {
        let mut node_positions = HashSet::new();

        for antennas in self.groupings.values() {
            let n = antennas.len();

            if n == 1 {
                continue;
            }

            node_positions.extend(antennas.iter());

            for (i, p0) in antennas.iter().take(n - 1).enumerate() {
                for p1 in &antennas[(i + 1)..] {
                    let v = p1 - p0;

                    for (start, step) in [(*p0, -v), (*p1, v)] {
                        let mut p = start + step;

                        while self.in_bounds(p) {
                            node_positions.insert(p);

                            p += step;
                        }
                    }
                }
            }
        }

        node_positions.len()
    }
}

/// Produces a sparse map of antennas (`AntennaMap`) from the puzzle input. Any non-period
/// character is treated as representing a unique frequency.
impl<R> From<R> for AntennaMap
where
    R: BufRead,
{
    fn from(value: R) -> Self {
        let mut size = IVec2::ZERO;
        let mut groupings: HashMap<_, Vec<_>> = HashMap::new();

        for (y, l) in value
            .lines()
            .map_while(Result::ok)
            .filter(|l| !l.is_empty())
            .enumerate()
        {
            size.y += 1;
            size.x = size.x.max(l.chars().count().try_into().unwrap());

            for (x, c) in l.chars().enumerate().filter(|&(_, c)| c != '.') {
                let pos = IVec2::new(x.try_into().unwrap(), y.try_into().unwrap());

                groupings.entry(c).or_default().push(pos);
            }
        }

        Self { size, groupings }
    }
}

pub fn part_01() {
    let map: AntennaMap = common::puzzle_input("2024-08").unwrap().into();

    let antinode_count = map.count_unique_antinodes();

    println!("Unique antinodes: {antinode_count}");
}

pub fn part_02() {
    let map: AntennaMap = common::puzzle_input("2024-08").unwrap().into();

    let antinode_count = map.count_unique_harmonic_antinodes();

    println!("Unique harmonic antinodes: {antinode_count}");
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use super::*;

    fn parsed_test_input() -> AntennaMap {
        let input = r"............
........0...
.....0......
.......0....
....0.......
......A.....
............
............
........A...
.........A..
............
............";

        let map: AntennaMap = BufReader::new(input.as_bytes()).into();

        assert_eq!(map.size, IVec2::new(12, 12));

        map
    }

    #[test]
    fn antinode_count() {
        let count = parsed_test_input().count_unique_antinodes();

        assert_eq!(count, 14);
    }

    #[test]
    fn harmonic_antinode_count() {
        let count = parsed_test_input().count_unique_harmonic_antinodes();

        assert_eq!(count, 34);
    }
}
