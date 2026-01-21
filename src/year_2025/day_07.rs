use std::{
    collections::HashMap,
    fmt::{Display, Write},
    io::Read,
};

use nalgebra::Vector2;

use crate::common;

/// Represents all the valid cell types for the problem
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
enum CellType {
    #[default]
    Empty,
    Start,
    Splitter,
}

impl TryFrom<u8> for CellType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'.' => Ok(CellType::Empty),
            b'S' => Ok(CellType::Start),
            b'^' => Ok(CellType::Splitter),
            _ => Err("Byte is not a valid puzzle cell"),
        }
    }
}

impl Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            CellType::Empty => '.',
            CellType::Start => 'S',
            CellType::Splitter => '^',
        };

        f.write_char(c)
    }
}

/// Try to build a manifold [[`common::Grid2`]] from the puzzle input `bytes`. Only succeeds when
/// every non-newline `byte` is convertible to [[`CellType`]] and the input is rectangular when
/// split at newlines.
fn try_manifold_from_bytes<I>(bytes: I) -> Result<common::Grid2<CellType>, &'static str>
where
    I: Iterator<Item = u8>,
{
    let mut width = None;
    let data: Vec<_> = bytes
        .enumerate()
        .filter_map(|(i, b)| match b {
            b'\r' | b'\n' => {
                width = width.or(Some(i));
                None
            }
            _ => Some(CellType::try_from(b)),
        })
        .collect::<Result<_, _>>()?;

    let width =
        u32::try_from(width.unwrap_or(0)).map_err(|_| "Width of lines should fit within `u32`")?;

    common::Grid2::from_vec(data, width).ok_or("Input is not a rectangular grid")
}

/// Tries to find the [[`CellType::Start`]] in the first row of the `manifold`.
fn start_point(manifold: &common::Grid2<CellType>) -> Option<Vector2<i32>> {
    (0..manifold.width()).find_map(|x| {
        if manifold.get(Vector2::new(x, 0)) == &CellType::Start {
            Some(Vector2::new(x, 0))
        } else {
            None
        }
    })
}

/// Fires the tachyon beam from the [[`start_point`]] of the `manifold` and progresses the beam
/// through the `manifold` while tracking the total number of splitters hit, and how many timelines
/// the beam traverses.
///
/// Returns: `(splitters_hit, num_timelines)`
fn fire_beam(manifold: &common::Grid2<CellType>) -> (u32, u64) {
    let mut beam_positions: HashMap<i32, u64> = start_point(manifold)
        .into_iter()
        .map(|p| (p.x, 1))
        .collect();
    let mut splitters_hit = 0;

    for y in 1..manifold.height() {
        let mut next = HashMap::<i32, u64>::new();
        let stepped = beam_positions.into_iter().flat_map(|(x, c)| {
            let next_pos = Vector2::new(x, y);

            if manifold.get(next_pos) == &CellType::Splitter {
                splitters_hit += 1;

                common::MaybeOneTwo::Two((x - 1, c), (x + 1, c))
            } else {
                common::MaybeOneTwo::One((x, c))
            }
        });

        for (x, c) in stepped {
            *next.entry(x).or_default() += c;
        }

        beam_positions = next;
    }

    (splitters_hit, beam_positions.values().sum())
}

pub fn part_01() {
    let bytes = common::puzzle_input("2025-07")
        .unwrap()
        .bytes()
        .map_while(Result::ok);

    let manifold = try_manifold_from_bytes(bytes).unwrap();
    let (hit_count, _) = fire_beam(&manifold);

    println!("Splitters hit: {hit_count}");
}

pub fn part_02() {
    let bytes = common::puzzle_input("2025-07")
        .unwrap()
        .bytes()
        .map_while(Result::ok);

    let manifold = try_manifold_from_bytes(bytes).unwrap();
    let (_, timelines) = fire_beam(&manifold);

    println!("Tachyon timelines: {timelines}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn data() -> &'static str {
        r#".......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
...............
"#
    }

    #[test]
    fn count_splits() {
        let manifold = try_manifold_from_bytes(data().bytes()).unwrap();
        let (hit_count, _) = fire_beam(&manifold);

        assert_eq!(hit_count, 21);
    }

    #[test]
    fn count_timelines() {
        let manifold = try_manifold_from_bytes(data().bytes()).unwrap();
        let (_, timelines) = fire_beam(&manifold);

        assert_eq!(timelines, 40);
    }
}
