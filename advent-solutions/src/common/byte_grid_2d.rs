/// Treats a `Vec<u8>` as a 2D grid so long as the data can be treated as a filled rectangle.
#[derive(Debug, Clone)]
pub struct ByteGrid2D {
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

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }
}
