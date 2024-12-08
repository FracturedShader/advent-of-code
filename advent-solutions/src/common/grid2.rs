use glam::UVec2;

/// Treats a `Vec<u8>` as a 2D grid, so long as the data can be treated as a filled rectangle.
/// Increasing `x` and `y` both look further forward in memory, but what that means depends on
/// context that this class does not assume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid2<T> {
    size: UVec2,
    stride: usize,
    data: Vec<T>,
}

#[derive(Debug, Clone)]
pub struct Iter2D<'grid, T> {
    pos: UVec2,
    grid: &'grid Grid2<T>,
}

impl<'grid, T> Iterator for Iter2D<'grid, T> {
    type Item = &'grid T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.grid.in_bounds(self.pos) {
            let p = self.pos;

            self.pos += UVec2::X;

            if self.pos.x >= self.grid.size.x {
                self.pos.y += self.pos.x / self.grid.size.x;
                self.pos.x %= self.grid.size.x;
            }

            Some(self.grid.get(p))
        } else {
            None
        }
    }
}

impl<T> Grid2<T> {
    fn index_to_coord(&self, index: usize) -> UVec2 {
        UVec2::new(
            u32::try_from(index % self.stride).unwrap(),
            u32::try_from(index / self.stride).unwrap(),
        )
    }
}

impl Grid2<u8> {
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

        let size = (
            u32::try_from(width).expect("grid should be less than `u32::MAX` wide"),
            u32::try_from(height).expect("grid should be less than `u32::MAX` tall"),
        )
            .into();

        if (height * stride) - data.len() <= (stride - width) {
            Ok(Self { size, stride, data })
        } else {
            Err("Input is not a rectangular grid")
        }
    }
}

impl<T> Grid2<T> {
    /// Attempts to construct a rectangular 2D grid from a flat `Vec<T>`. Length of `data` must be
    /// exactly divisible by `width` and fit within a `u32`.
    pub fn from_vec(data: Vec<T>, width: u32) -> Option<Self> {
        if data.len() % (width as usize) == 0 {
            u32::try_from(data.len() / width as usize)
                .ok()
                .map(|height| Self {
                    size: UVec2::new(width, height),
                    stride: width as usize,
                    data,
                })
        } else {
            None
        }
    }

    pub fn get(&self, p: UVec2) -> &T {
        &self.get_row(p.y)[p.x as usize]
    }

    pub fn get_mut(&mut self, p: UVec2) -> &mut T {
        &mut self.get_row_mut(p.y)[p.x as usize]
    }

    pub fn get_row(&self, y: u32) -> &[T] {
        let idx = (y as usize) * self.stride;

        &self.data[idx..(idx + self.size.x as usize)]
    }

    pub fn get_row_mut(&mut self, y: u32) -> &mut [T] {
        let idx = (y as usize) * self.stride;

        &mut self.data[idx..(idx + self.size.x as usize)]
    }

    pub fn in_bounds(&self, p: UVec2) -> bool {
        (0..self.size.x).contains(&p.x) && (0..self.size.y).contains(&p.y)
    }

    pub fn inner(&self) -> &[T] {
        &self.data
    }

    pub fn iter(&self) -> Iter2D<'_, T> {
        self.into_iter()
    }

    pub fn set(&mut self, p: UVec2, v: T) {
        *self.get_mut(p) = v;
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }
}

impl<T> Grid2<T>
where
    T: Eq,
{
    pub fn position(&self, value: &T) -> Option<UVec2> {
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
            pos: UVec2::ZERO,
            grid: self,
        }
    }
}
