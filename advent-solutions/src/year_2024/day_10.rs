use std::{
    collections::{HashSet, VecDeque},
    io::BufRead,
};

use glam::UVec2;

use crate::common::Grid2;

/// Produces `(coord, score)` pairs for every trailhead ('0')
fn trailhead_scores(grid: &Grid2<u8>) -> impl Iterator<Item = (UVec2, usize)> + use<'_> {
    grid.enumerate().filter_map(|(p, &v)| {
        if v == b'0' {
            let mut queue = VecDeque::with_capacity(32);
            let mut visited = HashSet::new();
            let mut count = 0;

            queue.push_back((p, v));

            // Concurrent breadth-first search, but bails early on already visited nodes
            while let Some((p, v)) = queue.pop_front() {
                if !visited.insert(p) {
                    continue;
                }

                if v == b'9' {
                    count += 1;
                } else {
                    let n = v + 1;

                    queue.extend(grid.valid_neighbors4(p).filter_map(|(p, &v)| {
                        if v == n {
                            Some((p, v))
                        } else {
                            None
                        }
                    }));
                }
            }

            Some((p, count))
        } else {
            None
        }
    })
}

/// Produces `(coord, rating)` pairs for every trailhead ('0')
fn trailhead_ratings(grid: &Grid2<u8>) -> impl Iterator<Item = (UVec2, usize)> + use<'_> {
    grid.enumerate().filter_map(|(p, &v)| {
        if v == b'0' {
            let mut queue = VecDeque::with_capacity(128);
            let mut count = 0;

            queue.push_back((p, v));

            // Concurrent breadth-first search of all potential paths
            while let Some((p, v)) = queue.pop_front() {
                if v == b'9' {
                    count += 1;
                } else {
                    let n = v + 1;

                    queue.extend(grid.valid_neighbors4(p).filter_map(|(p, &v)| {
                        if v == n {
                            Some((p, v))
                        } else {
                            None
                        }
                    }));
                }
            }

            Some((p, count))
        } else {
            None
        }
    })
}

#[allow(clippy::needless_pass_by_value)]
pub fn part_01(reader: Option<impl BufRead>) {
    let mut buf = Vec::new();

    let _ = reader
        .expect("2024-10.txt should be in the data folder")
        .read_to_end(&mut buf);

    let grid = Grid2::try_from_file_data(buf).expect("file should be a topographic map");
    let trailhead_scores_sum = trailhead_scores(&grid).map(|t| t.1).sum::<usize>();

    println!("Sum of trailhead scores: {trailhead_scores_sum}");
}

#[allow(clippy::needless_pass_by_value)]
pub fn part_02(reader: Option<impl BufRead>) {
    let mut buf = Vec::new();

    let _ = reader
        .expect("2024-10.txt should be in the data folder")
        .read_to_end(&mut buf);

    let grid = Grid2::try_from_file_data(buf).expect("file should be a topographic map");
    let trailhead_ratings_sum = trailhead_ratings(&grid).map(|t| t.1).sum::<usize>();

    println!("Sum of trailhead ratings: {trailhead_ratings_sum}");
}

#[cfg(test)]
mod test {
    use crate::common::Grid2;

    use super::*;

    fn parsed_input() -> Grid2<u8> {
        let input = r"89010123
78121874
87430965
96549874
45678903
32019012
01329801
10456732";

        Grid2::try_from_file_data(input.bytes().collect()).unwrap()
    }

    #[test]
    fn score_trailheads() {
        let grid = parsed_input();
        let trailhead_scores = trailhead_scores(&grid).map(|t| t.1).collect::<Vec<_>>();

        assert_eq!(trailhead_scores, vec![5, 6, 5, 3, 1, 3, 5, 3, 5]);
    }

    #[test]
    fn rate_trailheads() {
        let grid = parsed_input();
        let trailhead_ratings = trailhead_ratings(&grid).map(|t| t.1).collect::<Vec<_>>();

        assert_eq!(trailhead_ratings, vec![20, 24, 10, 4, 1, 4, 5, 8, 5]);
    }
}
