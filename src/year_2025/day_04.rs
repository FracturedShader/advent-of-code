use nalgebra::Vector2;

use crate::common;

/// A roll at point `p` in the `grid` is considered accessible by a forklift if it has less than
/// four rolls around it (including diagonally).
fn is_accessible_roll(p: Vector2<i32>, grid: &common::Grid2<u8>) -> bool {
    grid.valid_neighbors8(p)
        .filter(|(_, (_, n))| **n == b'@')
        .count()
        < 4
}

/// Counts the number of rolls that can be accessed by a forklift in the current `grid`
/// configuration.
fn directly_accessible_rolls(grid: &common::Grid2<u8>) -> usize {
    grid.enumerate()
        .filter(|(p, v)| **v == b'@' && is_accessible_roll(*p, grid))
        .count()
}

/// Repeatedly removes accessible rolls from `grid` until no more accessible rolls remain. Returns
/// the total number of rolls removed.
fn removable_rolls(mut grid: common::Grid2<u8>) -> usize {
    let mut known_rolls: Vec<_> = grid
        .enumerate()
        .filter_map(|(p, v)| if *v == b'@' { Some(p) } else { None })
        .collect();

    let mut removed = 0;

    loop {
        let (removable, rest): (Vec<_>, Vec<_>) = known_rolls
            .into_iter()
            .partition(|p| is_accessible_roll(*p, &grid));

        if removable.is_empty() {
            break;
        }

        removed += removable.len();

        known_rolls = rest;

        for p in removable {
            *grid.get_mut(p) = b'.';
        }
    }

    removed
}

pub fn part_01() {
    let grid = common::Grid2::try_from_reader(common::puzzle_input("2025-04").unwrap()).unwrap();
    let accessible = directly_accessible_rolls(&grid);

    println!("Accessible rolls: {accessible}");
}

pub fn part_02() {
    let grid = common::Grid2::try_from_reader(common::puzzle_input("2025-04").unwrap()).unwrap();
    let removed = removable_rolls(grid);

    println!("Removed rolls: {removed}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn data() -> &'static str {
        r#"..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@."#
    }

    #[test]
    fn max_three_neighbors() {
        let grid = common::Grid2::try_from_bytes(data().bytes()).unwrap();
        let accessible = directly_accessible_rolls(&grid);

        assert_eq!(accessible, 13);
    }

    #[test]
    fn total_removable() {
        let grid = common::Grid2::try_from_bytes(data().bytes()).unwrap();
        let removed = removable_rolls(grid);

        assert_eq!(removed, 43);
    }
}
