use std::io::BufRead;

use itertools::Itertools;
use rayon::prelude::*;

/// Represents which cells are visible from outside the grid in row-major order
struct VisibilityMap(Vec<bool>);

impl VisibilityMap {
    fn num_visible(&self) -> usize {
        self.0.iter().copied().map(usize::from).sum()
    }
}

/// Collection of the scenic scores of every cell in row-major order
struct ScenicMap(Vec<u32>);

impl ScenicMap {
    fn highest_score(&self) -> u32 {
        *self.0.iter().max().unwrap()
    }
}

/// A row-major collection of tree heights for a rectangular grid
struct TreeMap {
    shape: (usize, usize),
    data: Vec<u8>,
}

/// Helper to perform a marching depth test.
/// Current height is only visible if it is the tallest seen so far.
fn depth_op(height: u8, visible: &mut bool, tallest: &mut u8) {
    if height > *tallest {
        *visible = true;
        *tallest = height;
    }
}

/// Helper method to march along a zipped iterator tracking the visibility of entries by checking
/// if each entry is taller than all of those before it.
fn propagate_visible<'a, 'b, 'c, I, const HEIGHT_LIMIT: usize>(iter: I)
where
    I: Iterator<Item = &'a mut (&'b u8, &'c mut bool)>,
    'b: 'a,
    'c: 'a,
{
    // The first item is always visible, even if it is zero height
    let mut tallest_seen = 0;

    for (&height, visible) in iter {
        depth_op(height, visible, &mut tallest_seen);

        // Everything past this point must be hidden from this direction
        if height as usize == HEIGHT_LIMIT - 1 {
            break;
        }
    }
}

/// Computes the scenic score based on the marched visible distance along an axis and uses the
/// current cell height to update the visible distances.
fn scenic_op<const HEIGHT_LIMIT: usize>(
    height: u8,
    score: &mut u32,
    distances: &mut [u32; HEIGHT_LIMIT],
) {
    assert!((height as usize) < HEIGHT_LIMIT);

    let idx = height as usize;

    *score *= distances[idx];

    for (i, d) in distances.iter_mut().enumerate() {
        *d = if i > idx { *d + 1 } else { 1 };
    }
}

/// Marches along an axis updating scenic scores and tracking maximum viewing distance at all
/// heights.
fn propagate_view_dist<'a, 'b, 'c, I, const HEIGHT_LIMIT: usize>(iter: I)
where
    I: Iterator<Item = &'a mut (&'b u8, &'c mut u32)>,
    'b: 'a,
    'c: 'a,
{
    let mut dist = [0; HEIGHT_LIMIT];

    for (&height, score) in iter {
        scenic_op::<HEIGHT_LIMIT>(height, score, &mut dist);
    }
}

impl TreeMap {
    /// Compute which cells are visible along any axis from outside the grid. A cell is visible if
    /// all cells between it and an edge are shorter.
    fn compute_visibility(&self) -> VisibilityMap {
        let mut tallest: Vec<u8> = vec![0; self.shape.0];

        // TODO: This is just four orthographic depth map tests. A prime candidate for the GPU.

        let data = self.transform_grid(
            false,
            |b| {
                *b = true;
            },
            |mut v| {
                propagate_visible::<_, 10>(v.iter_mut());
                propagate_visible::<_, 10>(v.iter_mut().rev());
            },
            &mut tallest,
            |&h, v, t| {
                depth_op(h, v, t);
            },
            |r| {
                r.fill(0);
            },
        );

        VisibilityMap(data)
    }

    /// Computes the scenic score for every cell in the map. The scenic score is a multiplication
    /// of how many cells can be traveled along each axis before reaching a cell of greater or
    /// equal height (or the edge of the map).
    fn compute_scenic_score(&self) -> ScenicMap {
        const HEIGHT_LIMIT: usize = 10;

        let mut vis_dist = vec![[0u32; HEIGHT_LIMIT]; self.shape.0];

        let data = self.transform_grid(
            1u32,
            |d| {
                *d = 0;
            },
            |mut v| {
                propagate_view_dist::<_, HEIGHT_LIMIT>(v.iter_mut());
                propagate_view_dist::<_, HEIGHT_LIMIT>(v.iter_mut().rev());
            },
            &mut vis_dist,
            |&h, s, d| {
                scenic_op::<HEIGHT_LIMIT>(h, s, d);
            },
            |r| {
                for da in r.iter_mut() {
                    da.fill(0);
                }
            },
        );

        ScenicMap(data)
    }

    /// Helper method to march the entire grid from each of the four edges and compute a resulting
    /// grid. Rows are operated in parallel working both forwards and backwards while columns are
    /// treated sequentially also in a forwards + backwards manner.
    fn transform_grid<D, E, F, G, T, R>(
        &self,
        initial: D,
        edge_op: E,
        par_row_op: F,
        cell_data_row: &mut [T],
        cell_op: G,
        reset_cell_data: R,
    ) -> Vec<D>
    where
        D: Clone + Send,
        E: Fn(&mut D),
        F: Fn(Vec<(&u8, &mut D)>) + Send + Sync,
        G: Fn(&u8, &mut D, &mut T),
        R: FnOnce(&mut [T]),
    {
        let grid_width = self.shape.0;
        let mut dest = vec![initial; self.data.len()];

        for b in dest.iter_mut().take(self.shape.0) {
            edge_op(b);
        }

        for b in dest.iter_mut().rev().take(self.shape.0) {
            edge_op(b);
        }

        for b in dest.iter_mut().step_by(self.shape.0) {
            edge_op(b);
        }

        for b in dest.iter_mut().skip(self.shape.0 - 1).step_by(self.shape.0) {
            edge_op(b);
        }

        // Technically we want to be cache size aware when splitting, but rayon should handle that
        self.data
            .par_iter()
            .zip(dest.par_iter_mut())
            .chunks(grid_width)
            .for_each(par_row_op);

        // The problem is there is a linear dependence between rows and the data is not oriented
        // in a CPU friendly manner for splitting along column lines. The work is also small enough
        // that transposing the data just to make it cache friendly only to have to transpose back
        // at the end seems wasteful.

        for i in &self.data.iter().zip(dest.iter_mut()).chunks(grid_width) {
            for t in i.zip(cell_data_row.iter_mut()) {
                let ((s, d), c) = t;

                cell_op(s, d, c);
            }
        }

        reset_cell_data(cell_data_row);

        for i in &self
            .data
            .iter()
            .zip(dest.iter_mut())
            .rev()
            .chunks(grid_width)
        {
            for t in i.zip(cell_data_row.iter_mut()) {
                let ((s, d), c) = t;

                cell_op(s, d, c);
            }
        }

        dest
    }
}

impl<S> FromIterator<S> for TreeMap
where
    S: AsRef<str>,
{
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut width = None;
        let mut height = 0;

        let mut data = Vec::new();

        for l in iter {
            let l = l.as_ref();
            let line_width = l.len();

            assert!(
                *width.get_or_insert(line_width) == line_width,
                "All rows must be the same length in a TreeMap"
            );

            data.extend(l.bytes().map(|b| match b {
                b'0'..=b'9' => b - b'0',
                _ => panic!("A TreeMap can only be built from ascii numbers"),
            }));

            height += 1;
        }

        TreeMap {
            shape: (width.unwrap_or(0), height),
            data,
        }
    }
}

pub fn part_01(reader: Option<impl BufRead>) {
    let reader = reader.expect("data should be available for this problem");
    let map = reader.lines().map_while(Result::ok).collect::<TreeMap>();
    let vis = map.compute_visibility();

    println!("Total visible trees: {}", vis.num_visible());
}

pub fn part_02(reader: Option<impl BufRead>) {
    let reader = reader.expect("data should be available for this problem");
    let map = reader.lines().map_while(Result::ok).collect::<TreeMap>();
    let scores = map.compute_scenic_score();

    println!("Highest scenic score: {}", scores.highest_score());
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_DATA: &str = r"30373
25512
65332
33549
35390";

    #[test]
    fn parse_input() {
        let map = TEST_DATA.lines().collect::<TreeMap>();

        assert_eq!(map.shape, (5, 5));
        assert_eq!(map.data.len(), 25);
        assert_eq!(map.data[0], 3);
        assert_eq!(map.data[5], 2);
        assert_eq!(map.data[24], 0);
    }

    #[test]
    fn compute_visibility() {
        let map = TEST_DATA.lines().collect::<TreeMap>();
        let vis = map.compute_visibility();

        assert_eq!(
            vis.0,
            vec![
                true, true, true, true, true, true, true, true, false, true, true, true, false,
                true, true, true, false, true, false, true, true, true, true, true, true,
            ]
        );

        assert_eq!(vis.num_visible(), 21);
    }

    #[test]
    fn best_scenic() {
        let map = TEST_DATA.lines().collect::<TreeMap>();
        let scores = map.compute_scenic_score();

        assert_eq!(
            scores.0,
            vec![0, 0, 0, 0, 0, 0, 1, 4, 1, 0, 0, 6, 1, 2, 0, 0, 1, 8, 3, 0, 0, 0, 0, 0, 0,]
        );

        assert_eq!(scores.highest_score(), 8);
    }
}
