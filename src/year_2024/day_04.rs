use nalgebra::Vector2;

use crate::common::{self, Grid2};

/// Starting from `(x + dx, y + dy)`, is the entire `pattern` in the `grid` when moving by
/// `(dx, dy)` per `u8`?
fn rest_in_grid(pattern: &[u8], grid: &Grid2<u8>, p: Vector2<i32>, dp: Vector2<i32>) -> bool {
    let mut p = p;

    for b in pattern {
        p += dp;

        if grid.get(p) != b {
            return false;
        }
    }

    true
}

/// Counts the number times "XMAS" can be found starting or ending at this cell when looking only
/// forward in memory (right and/or down in the grid). To be used when scanning the entire grid.
fn count_xmas_cell(grid: &Grid2<u8>, p: Vector2<i32>) -> u32 {
    let rest = match *grid.get(p) {
        b'S' => b"AMX",
        b'X' => b"MAS",
        _ => {
            return 0;
        }
    };

    let rest = &rest[..];
    let needed = i32::try_from(rest.len()).unwrap();

    let room_before = (p.x - grid.extent_min().x) >= needed;
    let (room_after, room_below) = {
        let area = grid.extent_max() - p;

        (area.x > needed, area.y > needed)
    };

    let mut count = 0;

    if room_after {
        count += u32::from(rest_in_grid(rest, grid, p, Vector2::x()));
    }

    if room_below {
        if room_before {
            count += u32::from(rest_in_grid(rest, grid, p, Vector2::new(-1, 1)));
        }

        count += u32::from(rest_in_grid(rest, grid, p, Vector2::y()));

        if room_after {
            count += u32::from(rest_in_grid(rest, grid, p, Vector2::new(1, 1)));
        }
    }

    count
}

/// Counts the number of occurences of "XMAS" appearing in the `grid` in any direction (forwards,
/// backwards, up, down, diagonally).
fn count_xmas_grid(grid: &Grid2<u8>) -> u32 {
    let mut count = 0;
    let (width, height) = (grid.width(), grid.height());

    for y in 0..height {
        for x in 0..width {
            count += count_xmas_cell(grid, Vector2::new(y, x));
        }
    }

    count
}

/// What the opposing corner of an "X-MAS" box should contain.
fn x_mas_opposite(byte: u8) -> Option<u8> {
    match byte {
        b'M' => Some(b'S'),
        b'S' => Some(b'M'),
        _ => None,
    }
}

/// Is this cell the top-left corner of an "X-MAS" box? Limits scanning to be memory-forward. This
/// is only intended to be used when scanning the whole `grid` minus padding for the box.
fn is_x_mas_cell(grid: &Grid2<u8>, p: Vector2<i32>) -> bool {
    if let Some(o1) = x_mas_opposite(*grid.get(p)) {
        if let Some(o2) = x_mas_opposite(*grid.get(p + Vector2::new(2, 0))) {
            return *grid.get(p + Vector2::new(1, 1)) == b'A'
                && *grid.get(p + Vector2::new(0, 2)) == o2
                && *grid.get(p + Vector2::new(2, 2)) == o1;
        }
    }

    false
}

/// Counts the number of times "X-MAS" boxes can be found within the grid.
fn count_x_mas_grid(grid: &Grid2<u8>) -> u32 {
    let mut count = 0;
    let (width, height) = (grid.width() - 2, grid.height() - 2);

    for y in 0..height {
        for x in 0..width {
            count += u32::from(is_x_mas_cell(grid, Vector2::new(x, y)));
        }
    }

    count
}

/// Part 1 approach:
/// 1. Read the entire file in as bytes
/// 2. Treat the raw bytes as a 2D grid using a wrapper struct (helps with ignoring LF/CRLF)
/// 3. Iterate through all cells in memory order considering them as either the start or end of
///    "XMAS" in exactly half the supported directions due to symmetry
///     - Since we are considering both the forward and backward options we can limit the search
///       for the rest to only look forward in memory (forward along same row or in following rows)
/// 4. Sum up the number of matches found for each cell in the table
pub fn part_01() {
    let grid = Grid2::try_from_reader(common::puzzle_input("2024-04").unwrap()).unwrap();
    let match_count = count_xmas_grid(&grid);

    println!("XMAS appearances: {match_count}");
}

/// Part 2 approach:
/// 1. Read the entire file in as bytes
/// 2. Treat the raw bytes as a 2D grid using a wrapper struct (helps with ignoring LF/CRLF)
/// 3. Iterate through all cells in memory order (minus space for the box) considering them as
///    potentially the top-left corner of the "X-MAS" box
///     - Every box must have a top-left corner
///     - Every box has an 'A' in the middle
///     - Just need to track what the opposites of the top two corners should be and match
/// 4. Count up the number of cells that are the top-left corner of an "X-MAS" box
pub fn part_02() {
    let grid = Grid2::try_from_reader(common::puzzle_input("2024-04").unwrap()).unwrap();
    let match_count = count_x_mas_grid(&grid);

    println!("X-MAS appearances: {match_count}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn parsed_test_input() -> Grid2<u8> {
        let input = r"MMMSXXMASM
MSAMXMSMSA
AMXSXMAAMM
MSAMASMSMX
XMASAMXAMM
XXAMMXXAMA
SMSMSASXSS
SAXAMASAAA
MAMMMXMMMM
MXMXAXMASX";

        Grid2::try_from_reader(input.as_bytes()).unwrap()
    }

    #[test]
    fn parse_input() {
        let grid = parsed_test_input();

        assert_eq!(grid.size(), Vector2::new(10, 10));
    }

    #[test]
    fn word_search() {
        let grid = parsed_test_input();

        let match_count = count_xmas_grid(&grid);

        assert_eq!(match_count, 18);
    }

    #[test]
    fn block_search() {
        let grid = parsed_test_input();

        assert!(is_x_mas_cell(&grid, Vector2::x()));

        let match_count = count_x_mas_grid(&grid);

        assert_eq!(match_count, 9);
    }
}
