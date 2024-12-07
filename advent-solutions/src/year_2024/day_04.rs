use std::io::BufRead;

use crate::common::ByteGrid2D;

/// Starting from `(x + dx, y + dy)`, is the entire `pattern` in the `grid` when moving by
/// `(dx, dy)` per `u8`?
fn rest_in_grid(
    pattern: &[u8],
    grid: &ByteGrid2D,
    x: usize,
    y: usize,
    dx: isize,
    dy: isize,
) -> bool {
    let mut x = x;
    let mut y = y;

    for b in pattern {
        x = x
            .checked_add_signed(dx)
            .expect("grid should always be within range of `usize`");
        y = y
            .checked_add_signed(dy)
            .expect("grid should always be within range of `usize`");

        if grid.get(x, y) != b {
            return false;
        }
    }

    true
}

/// Counts the number times "XMAS" can be found starting or ending at this cell when looking only
/// forward in memory (right and/or down in the grid). To be used when scanning the entire grid.
fn count_xmas_cell(grid: &ByteGrid2D, x: usize, y: usize) -> u32 {
    let rest = match *grid.get(x, y) {
        b'S' => b"AMX",
        b'X' => b"MAS",
        _ => {
            return 0;
        }
    };

    let rest = &rest[..];
    let needed = rest.len();

    let room_before = x >= needed;
    let room_after = (grid.width() - x) > needed;
    let room_below = (grid.height() - y) > needed;

    let mut count = 0;

    if room_after {
        count += u32::from(rest_in_grid(rest, grid, x, y, 1, 0));
    }

    if room_below {
        if room_before {
            count += u32::from(rest_in_grid(rest, grid, x, y, -1, 1));
        }

        count += u32::from(rest_in_grid(rest, grid, x, y, 0, 1));

        if room_after {
            count += u32::from(rest_in_grid(rest, grid, x, y, 1, 1));
        }
    }

    count
}

/// Counts the number of occurences of "XMAS" appearing in the `grid` in any direction (forwards,
/// backwards, up, down, diagonally).
fn count_xmas_grid(grid: &ByteGrid2D) -> u32 {
    let mut count = 0;

    for row in 0..grid.height() {
        for col in 0..grid.width() {
            count += count_xmas_cell(grid, col, row);
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

/// Is this cell the top-left corner of an "X-MAS" box? Limits scanning to be memory-forward and
/// this is only intended to be used when scanning the whole `grid` minus padding for the box.
fn is_x_mas_cell(grid: &ByteGrid2D, x: usize, y: usize) -> bool {
    if let Some(o1) = x_mas_opposite(*grid.get(x, y)) {
        if let Some(o2) = x_mas_opposite(*grid.get(x + 2, y)) {
            return *grid.get(x + 1, y + 1) == b'A'
                && *grid.get(x, y + 2) == o2
                && *grid.get(x + 2, y + 2) == o1;
        }
    }

    false
}

/// Counts the number of times "X-MAS" boxes can be found within the grid.
fn count_x_mas_grid(grid: &ByteGrid2D) -> u32 {
    let mut count = 0;

    for row in 0..(grid.height() - 2) {
        for col in 0..(grid.width() - 2) {
            count += u32::from(is_x_mas_cell(grid, col, row));
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
#[allow(clippy::needless_pass_by_value)]
pub fn part_01(reader: Option<impl BufRead>) {
    let mut buf = Vec::with_capacity(20 * 1024);

    let _ = reader
        .expect("2024-04.txt should exist in the data directory")
        .read_to_end(&mut buf)
        .unwrap();

    let grid = ByteGrid2D::try_from_file_data(buf).unwrap();
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
#[allow(clippy::needless_pass_by_value)]
pub fn part_02(reader: Option<impl BufRead>) {
    let mut buf = Vec::with_capacity(20 * 1024);

    let _ = reader
        .expect("2024-04.txt should exist in the data directory")
        .read_to_end(&mut buf)
        .unwrap();

    let grid = ByteGrid2D::try_from_file_data(buf).unwrap();
    let match_count = count_x_mas_grid(&grid);

    println!("X-MAS appearances: {match_count}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn parsed_test_input() -> ByteGrid2D {
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

        ByteGrid2D::try_from_file_data(input.bytes().collect()).unwrap()
    }

    #[test]
    fn parse_input() {
        let grid = parsed_test_input();

        assert_eq!(grid.width(), 10);
        assert_eq!(grid.height(), 10);
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

        assert!(is_x_mas_cell(&grid, 1, 0));

        let match_count = count_x_mas_grid(&grid);

        assert_eq!(match_count, 9);
    }
}
