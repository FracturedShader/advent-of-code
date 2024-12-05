use std::io::BufRead;

/// Treats a `Vec<u8>` as a 2D grid so long as the data can be treated as a filled rectangle.
#[derive(Debug, Clone)]
struct ByteGrid2D {
    width: usize,
    height: usize,
    stride: usize,
    data: Vec<u8>,
}

impl ByteGrid2D {
    /// Try to construct an 2D grid from newline-delimited data (CRLF aware). Final newline not
    /// required.
    /// Errors if the data is not rectangular.
    pub fn try_from_file_data(data: Vec<u8>) -> Result<Self, &'static str> {
        let (width, stride) = {
            if let Some(newline_pos) = data.iter().position(|b| *b == b'\n') {
                if data[newline_pos - 1] == b'\r' {
                    (newline_pos - 1, newline_pos + 1)
                } else {
                    (newline_pos, newline_pos + 1)
                }
            } else {
                (data.len(), data.len())
            }
        };

        let height = data.len().div_ceil(stride);

        if (height * stride) - data.len() <= (stride - width) {
            Ok(Self {
                width,
                height,
                stride,
                data,
            })
        } else {
            Err("Input is not a rectangular grid")
        }
    }

    pub fn get(&self, x: usize, y: usize) -> &u8 {
        &self.get_row(y)[x]
    }

    pub fn get_row(&self, y: usize) -> &[u8] {
        let idx = y * self.stride;

        &self.data[idx..(idx + self.width)]
    }
}

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
    let room_after = (grid.width - x) > needed;
    let room_below = (grid.height - y) > needed;

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

    for row in 0..grid.height {
        for col in 0..grid.width {
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
/// this is only intended to be used when scanning the whole `grid`.
fn is_x_mas_cell(grid: &ByteGrid2D, x: usize, y: usize) -> bool {
    let room_after = (grid.width - x) > 2;
    let room_below = (grid.height - y) > 2;

    if !room_after || !room_below {
        return false;
    }

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

    for row in 0..grid.height {
        for col in 0..grid.width {
            count += u32::from(is_x_mas_cell(grid, col, row));
        }
    }

    count
}

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

    #[test]
    fn word_search() {
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

        let grid = ByteGrid2D::try_from_file_data(input.bytes().collect()).unwrap();

        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);

        let match_count = count_xmas_grid(&grid);

        assert_eq!(match_count, 18);
    }

    #[test]
    fn block_search() {
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

        let grid = ByteGrid2D::try_from_file_data(input.bytes().collect()).unwrap();

        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);

        assert!(is_x_mas_cell(&grid, 1, 0));

        let match_count = count_x_mas_grid(&grid);

        assert_eq!(match_count, 9);
    }
}
