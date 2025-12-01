use std::collections::VecDeque;

use glam::IVec2;

use crate::common::{self, Direction, Grid2};

/// A single garden plot with its coordinate and mask of same-type neighbors
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Plot {
    coord: IVec2,
    neighbors_mask: u8,
}

/// Common operations between different region representations
trait FencePricing {
    fn area(&self) -> usize;

    fn price(&self) -> usize;
}

/// The most simple form of region. Every plot contributes independently to calculations.
#[derive(Debug, Clone)]
struct Region(Vec<Plot>);

impl Region {
    fn perimiter(&self) -> u32 {
        self.0
            .iter()
            .map(|p| 4 - p.neighbors_mask.count_ones())
            .sum()
    }
}

impl FencePricing for Region {
    fn area(&self) -> usize {
        self.0.len()
    }

    fn price(&self) -> usize {
        self.area() * self.perimiter() as usize
    }
}

impl From<Vec<Plot>> for Region {
    fn from(value: Vec<Plot>) -> Self {
        Self(value)
    }
}

type EdgeX = (std::ops::Range<i32>, i32);
type EdgeY = (i32, std::ops::Range<i32>);

/// A garden region which seeks to maximize the bulk discount on fencing by combining neighboring
/// sides.
#[derive(Debug, Clone)]
struct BulkRegion {
    plots: Vec<Plot>,
    top_sides: Vec<EdgeX>,
    bottom_sides: Vec<EdgeX>,
    left_sides: Vec<EdgeY>,
    right_sides: Vec<EdgeY>,
}

impl BulkRegion {
    fn sides(&self) -> usize {
        self.top_sides.len()
            + self.bottom_sides.len()
            + self.left_sides.len()
            + self.right_sides.len()
    }
}

impl FencePricing for BulkRegion {
    fn area(&self) -> usize {
        self.plots.len()
    }

    fn price(&self) -> usize {
        self.area() * self.sides()
    }
}

/// Collapses unit-length horizontal edges by leveraging the underlying grid structure. The
/// structure allows for a single sort followed by a single-pass collapse operation to merge
/// neighboring edges.
fn collapse_horizontal_sides(sides: &mut Vec<EdgeX>) {
    sides.sort_by(|left, right| {
        if left.1 == right.1 {
            left.0.start.cmp(&right.0.start)
        } else {
            left.1.cmp(&right.1)
        }
    });

    for i in (1..sides.len()).rev() {
        let left = &sides[i - 1];
        let right = &sides[i];

        if left.1 != right.1 {
            continue;
        }

        if left.0.end != right.0.start {
            continue;
        }

        let extended = left.0.start..right.0.end;

        sides[i - 1].0 = extended;

        sides.swap_remove(i);
    }
}

/// Collapses unit-length vertical edges by leveraging the underlying grid structure. The structure
/// allows for a single sort followed by a single-pass collapse operation to merge neighboring
/// edges.
fn collapse_vertical_sides(sides: &mut Vec<EdgeY>) {
    sides.sort_by(|left, right| {
        if left.0 == right.0 {
            left.1.start.cmp(&right.1.start)
        } else {
            left.0.cmp(&right.0)
        }
    });

    for i in (1..sides.len()).rev() {
        let left = &sides[i - 1];
        let right = &sides[i];

        if left.0 != right.0 {
            continue;
        }

        if left.1.end != right.1.start {
            continue;
        }

        let extended = left.1.start..right.1.end;

        sides[i - 1].1 = extended;

        sides.swap_remove(i);
    }
}

impl From<Vec<Plot>> for BulkRegion {
    fn from(plots: Vec<Plot>) -> Self {
        let capacity = plots.len();

        // Keep each type of side separate to collapse them properly
        let mut top_sides = Vec::with_capacity(capacity);
        let mut bottom_sides = Vec::with_capacity(capacity);
        let mut left_sides = Vec::with_capacity(capacity);
        let mut right_sides = Vec::with_capacity(capacity);

        // Record every segment of fence in the appropriate list
        for p in &plots {
            let nx = p.coord.x + 1;
            let lr = p.coord.x..nx;

            if !Direction::North.in_set(p.neighbors_mask) {
                top_sides.push((lr.clone(), p.coord.y));
            }

            if !Direction::South.in_set(p.neighbors_mask) {
                bottom_sides.push((lr, p.coord.y + 1));
            }

            let ny = p.coord.y + 1;
            let td = p.coord.y..ny;

            if !Direction::West.in_set(p.neighbors_mask) {
                left_sides.push((p.coord.x, td.clone()));
            }

            if !Direction::East.in_set(p.neighbors_mask) {
                right_sides.push((p.coord.x + 1, td));
            }
        }

        // Sort fence pieces and merge them down to create sides
        collapse_horizontal_sides(&mut top_sides);
        collapse_horizontal_sides(&mut bottom_sides);

        collapse_vertical_sides(&mut left_sides);
        collapse_vertical_sides(&mut right_sides);

        Self {
            plots,
            top_sides,
            bottom_sides,
            left_sides,
            right_sides,
        }
    }
}

/// Generate regions from collections of connected plots found within the garden. Marches through
/// the garden and flood-fills from each unvisited cell to determine which plots connect to form a
/// region.
fn regions<R>(garden: &Grid2<u8>) -> Vec<R>
where
    R: From<Vec<Plot>>,
{
    let (width, height) = (garden.width(), garden.height());
    let mut visited =
        Grid2::from_vec(vec![false; (width * height).try_into().unwrap()], width).unwrap();

    garden
        .enumerate()
        .filter_map(|(p, &v)| {
            if *visited.get(p) {
                None
            } else {
                let mut plots = Vec::new();
                let mut queue = VecDeque::new();

                queue.push_back(p);

                while let Some(coord) = queue.pop_front() {
                    if *visited.get(coord) {
                        continue;
                    }

                    *visited.get_mut(coord) = true;

                    let mut neighbors = 0;

                    for (d, (p, &nv)) in garden.valid_neighbors4(coord) {
                        if nv == v {
                            neighbors |= u8::from(d);
                            queue.push_back(p);
                        }
                    }

                    plots.push(Plot {
                        coord,
                        neighbors_mask: neighbors,
                    });
                }

                Some(plots.into())
            }
        })
        .collect()
}

pub fn part_01() {
    let garden = Grid2::try_from_reader(common::puzzle_input("2024-12").unwrap())
        .expect("file should be a garden map");

    let fencing_cost = regions(&garden).iter().map(Region::price).sum::<usize>();

    println!("Total cost: {fencing_cost}");
}

pub fn part_02() {
    let garden = Grid2::try_from_reader(common::puzzle_input("2024-12").unwrap())
        .expect("file should be a garden map");

    let fencing_cost = regions(&garden)
        .iter()
        .map(BulkRegion::price)
        .sum::<usize>();

    println!("Total cost: {fencing_cost}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn small_grid() -> Grid2<u8> {
        let input = r"AAAA
BBCD
BBCC
EEEC";

        Grid2::try_from_data(input.bytes().collect()).unwrap()
    }

    fn enclosed_regions() -> Grid2<u8> {
        let input = r"OOOOO
OXOXO
OOOOO
OXOXO
OOOOO";

        Grid2::try_from_data(input.bytes().collect()).unwrap()
    }

    fn medium_grid() -> Grid2<u8> {
        let input = r"RRRRIICCFF
RRRRIICCCF
VVRRRCCFFF
VVRCCCJFFF
VVVVCJJCFE
VVIVCCJJEE
VVIIICJJEE
MIIIIIJJEE
MIIISIJEEE
MMMISSJEEE";

        Grid2::try_from_data(input.bytes().collect()).unwrap()
    }

    fn big_e() -> Grid2<u8> {
        let input = r"EEEEE
EXXXX
EEEEE
EXXXX
EEEEE";

        Grid2::try_from_data(input.bytes().collect()).unwrap()
    }

    fn abba() -> Grid2<u8> {
        let input = r"AAAAAA
AAABBA
AAABBA
ABBAAA
ABBAAA
AAAAAA";

        Grid2::try_from_data(input.bytes().collect()).unwrap()
    }

    #[test]
    fn small_garden() {
        let garden = small_grid();
        let regions: Vec<Region> = regions(&garden);
        let stats = regions
            .iter()
            .map(|r| (r.area(), r.perimiter(), r.price()))
            .collect::<Vec<_>>();

        assert_eq!(
            stats,
            vec![(4, 10, 40), (4, 8, 32), (4, 10, 40), (1, 4, 4), (3, 8, 24)]
        );
    }

    #[test]
    fn enclosed_garden() {
        let garden = enclosed_regions();
        let regions: Vec<Region> = regions(&garden);
        let stats = regions
            .iter()
            .map(|r| (r.area(), r.perimiter(), r.price()))
            .collect::<Vec<_>>();

        assert_eq!(
            stats,
            vec![(21, 36, 756), (1, 4, 4), (1, 4, 4), (1, 4, 4), (1, 4, 4)]
        );
    }

    #[test]
    fn medium_garden() {
        let garden = medium_grid();
        let regions: Vec<Region> = regions(&garden);
        let stats = regions
            .iter()
            .map(|r| (r.area(), r.perimiter(), r.price()))
            .collect::<Vec<_>>();

        assert_eq!(
            stats,
            vec![
                (12, 18, 216),
                (4, 8, 32),
                (14, 28, 392),
                (10, 18, 180),
                (13, 20, 260),
                (11, 20, 220),
                (1, 4, 4),
                (13, 18, 234),
                (14, 22, 308),
                (5, 12, 60),
                (3, 8, 24),
            ]
        );
    }

    #[test]
    fn bulk_e() {
        let garden = big_e();
        let regions: Vec<BulkRegion> = regions(&garden);
        let stats = regions
            .iter()
            .map(|r| (r.area(), r.sides(), r.price()))
            .collect::<Vec<_>>();

        assert_eq!(stats, vec![(17, 12, 204), (4, 4, 16), (4, 4, 16),]);
    }

    #[test]
    fn abba_price() {
        let garden = abba();
        let regions: Vec<BulkRegion> = regions(&garden);
        let price = regions.iter().map(BulkRegion::price).sum::<usize>();

        assert_eq!(price, 368);
    }

    #[test]
    fn medium_garden_bulk() {
        let garden = medium_grid();
        let regions: Vec<BulkRegion> = regions(&garden);
        let stats = regions
            .iter()
            .map(|r| (r.area(), r.sides(), r.price()))
            .collect::<Vec<_>>();

        assert_eq!(
            stats,
            vec![
                (12, 10, 120),
                (4, 4, 16),
                (14, 22, 308),
                (10, 12, 120),
                (13, 10, 130),
                (11, 12, 132),
                (1, 4, 4),
                (13, 8, 104),
                (14, 16, 224),
                (5, 6, 30),
                (3, 6, 18),
            ]
        );
    }
}
