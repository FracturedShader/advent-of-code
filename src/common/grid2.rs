use std::{
    fmt::{Display, Write},
    io::Read,
};

use nalgebra::Vector2;

/// Treats a `Vec<T>` as a 2D grid, so long as the data can be treated as a filled rectangle.
/// Increasing `x` and `y` both look further forward in memory, but what that means depends on
/// context that this class does not assume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid2<T> {
    min: Vector2<i32>,
    max: Vector2<i32>,
    stride: usize,
    data: Vec<T>,
}

impl<T> Display for Grid2<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in self.min.y..self.max.y {
            for x in self.min.x..self.max.x {
                self.get(Vector2::new(x, y)).fmt(f)?;
            }

            f.write_char('\n')?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Iter2D<'grid, T> {
    pos: Vector2<i32>,
    grid: &'grid Grid2<T>,
}

impl<'grid, T> Iterator for Iter2D<'grid, T> {
    type Item = &'grid T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.grid.in_bounds(self.pos) {
            let p = self.pos;

            self.pos += Vector2::x();

            if self.pos.x >= self.grid.max.x {
                self.pos.y += 1;
                self.pos.x = self.grid.min.x;
            }

            Some(self.grid.get(p))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Enumerate2D<'grid, T> {
    pos: Vector2<i32>,
    grid: &'grid Grid2<T>,
}

impl<'grid, T> Iterator for Enumerate2D<'grid, T> {
    type Item = (Vector2<i32>, &'grid T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.grid.in_bounds(self.pos) {
            let p = self.pos;

            self.pos += Vector2::x();

            if self.pos.x >= self.grid.max.x {
                self.pos.y += 1;
                self.pos.x = self.grid.min.x;
            }

            Some((p, self.grid.get(p)))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
}

impl Direction {
    pub fn in_set(self, mask: u8) -> bool {
        u8::from(self) & mask != 0
    }
}

impl From<Direction> for u8 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::North => 0b0000_0001,
            Direction::Northeast => 0b0000_0010,
            Direction::East => 0b0000_0100,
            Direction::Southeast => 0b0000_1000,
            Direction::South => 0b0001_0000,
            Direction::Southwest => 0b0010_0000,
            Direction::West => 0b0100_0000,
            Direction::Northwest => 0b1000_0000,
        }
    }
}

impl From<Direction> for Vector2<i32> {
    fn from(value: Direction) -> Self {
        match value {
            Direction::North => Vector2::new(0, -1),
            Direction::Northeast => Vector2::new(1, -1),
            Direction::East => Vector2::x(),
            Direction::Southeast => Vector2::new(1, 1),
            Direction::South => Vector2::y(),
            Direction::Southwest => Vector2::new(-1, 1),
            Direction::West => Vector2::new(-1, 0),
            Direction::Northwest => Vector2::new(-1, -1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidNeighbors<'grid, const N: usize, T> {
    i: usize,
    origin: Vector2<i32>,
    directions: [Direction; N],
    grid: &'grid Grid2<T>,
}

impl<'grid, const N: usize, T> Iterator for ValidNeighbors<'grid, N, T> {
    type Item = (Direction, (Vector2<i32>, &'grid T));

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.i == N {
                return None;
            }

            let dir = self.directions[self.i];
            let p = self.origin + Vector2::from(dir);

            self.i += 1;

            if self.grid.in_bounds(p) {
                return Some((dir, (p, self.grid.get(p))));
            }
        }
    }
}

impl<T> Grid2<T> {
    fn index_to_coord(&self, index: usize) -> Vector2<i32> {
        Vector2::new(
            i32::try_from(index % self.stride).unwrap(),
            i32::try_from(index / self.stride).unwrap(),
        )
    }
}

impl TryFrom<Vec<u8>> for Grid2<u8> {
    type Error = &'static str;

    /// Try to construct an 2D grid from newline-delimited data (CRLF aware). Final newline not
    /// required.
    /// Errors if the data is not rectangular.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let (width, stride) = {
            if let Some(newline_pos) = value.iter().position(|b| *b == b'\n') {
                if value[newline_pos - 1] == b'\r' {
                    (newline_pos - 1, newline_pos + 1)
                } else {
                    (newline_pos, newline_pos + 1)
                }
            } else {
                (value.len(), value.len())
            }
        };

        let height = value.len().div_ceil(stride);

        let max = Vector2::new(
            i32::try_from(width).expect("grid should be less than `i32::MAX` wide"),
            i32::try_from(height).expect("grid should be less than `i32::MAX` tall"),
        );

        if (height * stride) - value.len() <= (stride - width) {
            Ok(Self {
                min: Vector2::zeros(),
                max,
                stride,
                data: value,
            })
        } else {
            Err("Input is not a rectangular grid")
        }
    }
}

impl TryFrom<&[u8]> for Grid2<u8> {
    type Error = &'static str;

    /// Try to construct an 2D grid from newline-delimited data (CRLF aware). Final newline not
    /// required.
    /// Errors if the data is not rectangular.
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from(value.to_vec())
    }
}

impl Grid2<u8> {
    pub fn try_from_bytes<I>(iter: I) -> Result<Self, &'static str>
    where
        I: Iterator<Item = u8>,
    {
        Self::try_from(iter.collect::<Vec<_>>())
    }

    pub fn try_from_reader<R>(mut reader: R) -> Result<Self, &'static str>
    where
        R: Read,
    {
        let mut buf = Vec::new();

        let _ = reader.read_to_end(&mut buf);

        Self::try_from(&buf[..])
    }
}

impl<T> Grid2<T> {
    pub fn enumerate(&self) -> Enumerate2D<'_, T> {
        Enumerate2D {
            pos: Vector2::zeros(),
            grid: self,
        }
    }

    pub fn extent_max(&self) -> Vector2<i32> {
        self.max
    }

    pub fn extent_min(&self) -> Vector2<i32> {
        self.min
    }

    /// Attempts to construct a rectangular 2D grid from a flat `Vec<T>`. Length of `data` must be
    /// exactly divisible by `width` and fit within a `i32`.
    pub fn from_vec(data: Vec<T>, width: u32) -> Option<Self> {
        if data.len().is_multiple_of(width as usize) {
            let signed_width =
                i32::try_from(width).expect("grid shoud be less than `i32::MAX` wide");

            i32::try_from(data.len() / width as usize)
                .ok()
                .map(|height| Self {
                    min: Vector2::zeros(),
                    max: Vector2::new(signed_width, height),
                    stride: width as usize,
                    data,
                })
        } else {
            None
        }
    }

    pub fn get(&self, p: Vector2<i32>) -> &T {
        &self.get_row(p.y)[usize::try_from(p.x - self.min.x).unwrap()]
    }

    pub fn get_mut(&mut self, p: Vector2<i32>) -> &mut T {
        let min_x = self.min.x;

        &mut self.get_row_mut(p.y)[usize::try_from(p.x - min_x).unwrap()]
    }

    pub fn get_row(&self, y: i32) -> &[T] {
        let idx = usize::try_from(y - self.min.y).unwrap() * self.stride;

        &self.data[idx..(idx + usize::try_from(self.width()).unwrap())]
    }

    pub fn get_row_mut(&mut self, y: i32) -> &mut [T] {
        let width = usize::try_from(self.width()).unwrap();
        let idx = usize::try_from(y - self.min.y).unwrap() * self.stride;

        &mut self.data[idx..(idx + width)]
    }

    pub fn height(&self) -> i32 {
        self.size().y
    }

    pub fn in_bounds(&self, p: Vector2<i32>) -> bool {
        (self.min.x..self.max.x).contains(&p.x) && (self.min.y..self.max.y).contains(&p.y)
    }

    pub fn inner(&self) -> &[T] {
        &self.data
    }

    pub fn iter(&self) -> Iter2D<'_, T> {
        self.into_iter()
    }

    pub fn set(&mut self, p: Vector2<i32>, v: T) {
        *self.get_mut(p) = v;
    }

    pub fn size(&self) -> Vector2<i32> {
        self.max - self.min
    }

    pub fn valid_neighbors4(&self, p: Vector2<i32>) -> ValidNeighbors<'_, 4, T> {
        ValidNeighbors {
            i: 0,
            origin: p,
            directions: [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ],
            grid: self,
        }
    }

    pub fn valid_neighbors8(&self, p: Vector2<i32>) -> ValidNeighbors<'_, 8, T> {
        ValidNeighbors {
            i: 0,
            origin: p,
            directions: [
                Direction::North,
                Direction::Northeast,
                Direction::East,
                Direction::Southeast,
                Direction::South,
                Direction::Southwest,
                Direction::West,
                Direction::Northwest,
            ],
            grid: self,
        }
    }

    pub fn width(&self) -> i32 {
        self.size().x
    }
}

impl<T> Default for Grid2<T> {
    fn default() -> Self {
        Self {
            min: Vector2::zeros(),
            max: Vector2::zeros(),
            stride: 0,
            data: Vec::default(),
        }
    }
}

impl<T> Grid2<T>
where
    T: Default,
{
    pub fn default_filled(
        width: usize,
        height: usize,
    ) -> Result<Self, <u32 as TryFrom<usize>>::Error> {
        let width32 = u32::try_from(width)?;
        let data = (0..(width * height)).map(|_| Default::default()).collect();

        Ok(Self::from_vec(data, width32).unwrap())
    }
}

impl<T> Grid2<T>
where
    T: Eq,
{
    pub fn position(&self, value: &T) -> Option<Vector2<i32>> {
        self.data
            .iter()
            .position(|b| b == value)
            .map(|p| self.index_to_coord(p))
    }
}

impl<'grid, T> IntoIterator for &'grid Grid2<T> {
    type Item = &'grid T;

    type IntoIter = Iter2D<'grid, T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            pos: Vector2::zeros(),
            grid: self,
        }
    }
}
