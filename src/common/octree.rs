use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
    fmt::Debug,
    ops::{Add, Div, Range, Sub},
};

use nalgebra::Vector3;

use crate::common::octree_num_traits::{ConstTwo, OctreeNum};

#[derive(Debug, Clone, PartialEq, Eq)]
enum BuilderCellContents {
    /// Collection of indices of entries contained within this cell
    Values(Vec<usize>),
    /// Relative index offset of eight the cells that this cell is subdivided into
    Cells(usize),
}

impl Default for BuilderCellContents {
    fn default() -> Self {
        BuilderCellContents::Values(Vec::default())
    }
}

impl BuilderCellContents {
    /// Check if this cell should be considered "full" (contains at least eight entries).
    fn is_full(&self) -> bool {
        match self {
            BuilderCellContents::Values(items) => items.len() > 7,
            BuilderCellContents::Cells(_) => false,
        }
    }

    /// Convenievce method for pushing an index. Returns `true` if this `is_leaf()` and the `value`
    /// could be pushed.
    fn push(&mut self, value: usize) -> bool {
        match self {
            BuilderCellContents::Values(items) => {
                items.push(value);
                true
            }
            BuilderCellContents::Cells(_) => false,
        }
    }

    /// Try to take the values out of this cell. Returns `Some` when this cell `is_leaf()`.
    fn take_values(&mut self) -> Option<Vec<usize>> {
        if let BuilderCellContents::Values(items) = self {
            Some(std::mem::take(items))
        } else {
            None
        }
    }

    /// Try to get a reference to the values in this cell. Returns `Some` when this cell
    /// `is_leaf()`.
    fn values(&self) -> Option<&[usize]> {
        match self {
            BuilderCellContents::Values(items) => Some(items),
            BuilderCellContents::Cells(_) => None,
        }
    }
}

/// An entry in an octree that associates a 3D coordinate with some data. The relationship between
/// coordinate and data need not be unique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OctreeEntry<N, T> {
    pub coord: Vector3<N>,
    pub value: T,
    cell_index: usize,
}

impl<Coord, N, T> From<(Coord, T)> for OctreeEntry<N, T>
where
    Coord: Into<Vector3<N>>,
{
    fn from(value: (Coord, T)) -> Self {
        let (coord, data) = value;

        Self {
            coord: coord.into(),
            value: data,
            cell_index: 0,
        }
    }
}

/// While building the octree it is nice to be able to organize the entries even if the data itself
/// has no logical ordering. This wrapper prefers to maintain source ordering and grouping by
/// [[`OctreeEntry::cell_index`]].
#[derive(Debug, Clone)]
struct OctreeBuilderEntry<N, T> {
    source_index: usize,
    entry: OctreeEntry<N, T>,
}

impl<N, T> PartialEq for OctreeBuilderEntry<N, T> {
    fn eq(&self, other: &Self) -> bool {
        self.entry.cell_index == other.entry.cell_index && self.source_index == other.source_index
    }
}

impl<N, T> Eq for OctreeBuilderEntry<N, T> {}

impl<N, T> Ord for OctreeBuilderEntry<N, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.entry.cell_index.cmp(&other.entry.cell_index) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        self.source_index.cmp(&other.source_index)
    }
}

impl<N, T> PartialOrd for OctreeBuilderEntry<N, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<N, T> OctreeBuilderEntry<N, T> {
    /// Extract the [[`OctreeEntry`]] from within this builder.
    fn into_inner(self) -> OctreeEntry<N, T> {
        self.entry
    }
}

impl<Coord, N, T> From<(usize, (Coord, T))> for OctreeBuilderEntry<N, T>
where
    Coord: Into<Vector3<N>>,
{
    fn from(value: (usize, (Coord, T))) -> Self {
        let (source_index, entry) = value;

        Self {
            source_index,
            entry: entry.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum CellContents {
    #[default]
    None,
    /// Collection of indices of entries contained within this cell
    Values(Range<usize>),
    /// Relative index offset of the eight cells that this cell is subdivided into
    Cells(usize),
}

/// Gets the index of the child of the cube at `center` which contains `coord`.
fn child_offset_index<N>(center: &Vector3<N>, coord: &Vector3<N>) -> usize
where
    N: Copy + PartialOrd + nalgebra::Scalar,
{
    usize::from(coord.x > center.x)
        + (usize::from(coord.y > center.y) << 1)
        + (usize::from(coord.z > center.z) << 2)
}

/// Produces the eight cubes, as `(center, extent)`, contained within a subdivided cube when used as
/// an iterator. Makes working with octree child nodes more convenient and consistent.
#[derive(Debug, Clone)]
struct ContainedCubes<N> {
    index: usize,
    center: Vector3<N>,
    half_extent: N,
}

impl<N> ContainedCubes<N> {
    pub fn new(center: Vector3<N>, half_extent: N) -> Self {
        Self {
            index: 0,
            center,
            half_extent,
        }
    }
}

impl<N> Iterator for ContainedCubes<N>
where
    N: Copy + Add<Output = N> + Sub<Output = N> + nalgebra::Scalar,
{
    type Item = (Vector3<N>, N);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > 7 {
            None
        } else {
            let idx = self.index;

            self.index += 1;

            Some((
                Vector3::new(
                    if idx & 1 == 0 {
                        self.center.x - self.half_extent
                    } else {
                        self.center.x + self.half_extent
                    },
                    if idx & 2 == 0 {
                        self.center.y - self.half_extent
                    } else {
                        self.center.y + self.half_extent
                    },
                    if idx & 4 == 0 {
                        self.center.z - self.half_extent
                    } else {
                        self.center.z + self.half_extent
                    },
                ),
                self.half_extent,
            ))
        }
    }
}

/// Fundamentally a cube that is either a leaf node, or subdivided into eight smaller cubes
/// (divided in half along each axis).
#[derive(Debug, Clone, PartialEq, Eq)]
struct OctreeCell<N, Contents> {
    center: Vector3<N>,
    extent: N,
    contents: Contents,
}

/// Produces the absolute component-wise difference between `a` and `b`. Useful when `N` cannot
/// support negative numbers, but the absolute difference is still useful.
fn abs_diff<N>(a: &Vector3<N>, b: &Vector3<N>) -> Vector3<N>
where
    N: Copy + nalgebra::ClosedSubAssign + nalgebra::Scalar + nalgebra::SimdPartialOrd,
{
    let (mins, maxs) = a.inf_sup(b);

    maxs - mins
}

impl<N, Contents> OctreeCell<N, Contents> {
    /// Produces an iterable of cube definitions for the eight children of this cell.
    fn contained_cubes(&self) -> ContainedCubes<N>
    where
        N: Copy + Div<Output = N> + ConstTwo,
    {
        ContainedCubes::new(self.center, self.extent / N::TWO)
    }

    /// Get the squared distance of `point` to the surface of the cell. This is achieved by
    /// localizing `point` to the cell and then collapsing the space of the cell to a single point.
    /// Any point inside the cell will therefore have zero distance
    fn surface_exterior_distance_squared(&self, point: &Vector3<N>) -> N
    where
        N: OctreeNum,
    {
        let surface_diff = {
            let diff = abs_diff(point, &self.center);

            // Subtract out the cube's extent, but clamp at zero
            diff - diff.inf(&Vector3::from_element(self.extent))
        };

        surface_diff.dot(&surface_diff)
    }
}

impl<N> OctreeCell<N, BuilderCellContents> {
    /// Flatten cells containing distributed values to contiguous values. Updates `entries_offset`
    /// based on the number of values directly contained.
    fn build(self, entries_offset: &mut usize) -> OctreeCell<N, CellContents> {
        OctreeCell {
            center: self.center,
            extent: self.extent,
            contents: match self.contents {
                BuilderCellContents::Values(items) => {
                    let n = items.len();

                    if n == 0 {
                        CellContents::None
                    } else {
                        let start = *entries_offset;
                        let end = start + n;

                        *entries_offset = end;

                        CellContents::Values(start..end)
                    }
                }
                BuilderCellContents::Cells(offset) => CellContents::Cells(offset),
            },
        }
    }
}

impl<N, Contents> From<(Vector3<N>, N)> for OctreeCell<N, Contents>
where
    Contents: Default,
{
    fn from(value: (Vector3<N>, N)) -> Self {
        let (center, extent) = value;

        Self {
            center,
            extent,
            contents: Contents::default(),
        }
    }
}

/// Cells in a sparse octree are only subdivided once they are at capacity. This means that each
/// "layer" of the octree only has the cells actuially in use (and any empty neighbors for the
/// subdivision). Data is stored intornally such that entries in the same cell are grouped together
/// in memory. The cells themselves are be grouped by depth and arranged in Z/Lebesgue/Morton order
/// for better locality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseOctree<N, T> {
    entries: Vec<OctreeEntry<N, T>>,
    cells: Vec<OctreeCell<N, CellContents>>,
}

impl<N, T> SparseOctree<N, T> {
    /// Build a sparse collection of octree cells from the builder `entries` while limiting it to
    /// `max_depth`. All `entries` are to be located within `extent` of `center`. The `entries`
    /// will be modified as they are added to the tree and then sorted so that all entries in the
    /// same cell are located together in memory. The cells produced will be grouped by depth and
    /// arranged in Z/Lebesgue/Morton order for better locality.
    fn build_cell_tree(
        center: Vector3<N>,
        extent: N,
        entries: &mut [OctreeBuilderEntry<N, T>],
        max_depth: usize,
    ) -> Vec<OctreeCell<N, CellContents>>
    where
        N: OctreeNum,
    {
        // Build the intermediate, ordered, sparse, tree
        let cells = Self::builder_tree_from_entries(center, extent, entries, max_depth);

        // Tag each entry with the index of the cell that contains it
        for (cell_index, contents) in cells
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.contents.values().map(|v| (i, v)))
        {
            for &i in contents {
                entries[i].entry.cell_index = cell_index;
            }
        }

        // Group the entries by the cell they are in, but otherwise retain original order
        entries.sort_unstable();

        let mut entries_offset = 0;

        // Convert builder cells with scattered entries to co-located form
        cells
            .into_iter()
            .map(|c| c.build(&mut entries_offset))
            .collect()
    }

    /// Build the [[`Octree`]] from the builder `entries` while limiting it to
    /// `max_depth`. All `entries` are to be located within `extent` of `center`. The `entries`
    /// will be grouped based on the [[`OctreeCell`]] they end up in.
    fn build_from_parts(
        center: Vector3<N>,
        extent: N,
        mut entries: Vec<OctreeBuilderEntry<N, T>>,
        max_depth: usize,
    ) -> Self
    where
        N: OctreeNum,
    {
        // If all the entries are at the same point, don't bother subdividing
        let cells = if extent.is_zero() {
            vec![OctreeCell {
                center,
                extent,
                contents: CellContents::Values(0..(entries.len())),
            }]
        } else {
            Self::build_cell_tree(center, extent, &mut entries, max_depth)
        };

        let entries = entries
            .into_iter()
            .map(OctreeBuilderEntry::into_inner)
            .collect();

        Self { entries, cells }
    }

    /// Locate the deepest builder cell in `cells` that contains `coord`. Returns `(cell_index,
    /// cell_depth)`.
    fn builder_cell_for_coord(
        coord: &Vector3<N>,
        cells: &[OctreeCell<N, BuilderCellContents>],
    ) -> (usize, usize)
    where
        N: Copy + PartialOrd + nalgebra::Scalar,
    {
        let mut depth = 0;
        let mut idx = 0;

        loop {
            match &cells[idx].contents {
                BuilderCellContents::Values(_) => break (idx, depth),
                BuilderCellContents::Cells(base_offset) => {
                    depth += 1;
                    idx += base_offset + child_offset_index(&cells[idx].center, coord);
                }
            }
        }
    }

    /// Build a sparse collection of octree cells from the builder `entries` while limiting it to
    /// `max_depth`. All `entries` are to be located within `extent` of `center`. The cells
    /// produced will be grouped by depth and arranged in Z/Lebesgue/Morton order for better
    /// locality.
    fn builder_tree_from_entries(
        center: Vector3<N>,
        extent: N,
        entries: &[OctreeBuilderEntry<N, T>],
        max_depth: usize,
    ) -> Vec<OctreeCell<N, BuilderCellContents>>
    where
        N: OctreeNum,
    {
        // Start with an empty root cell
        let mut dynamic_cells = vec![OctreeCell {
            center,
            extent,
            contents: BuilderCellContents::default(),
        }];

        // Add every entry to the tree, subdividing cells when they fill up (if allowed).
        for builder_entry in entries {
            let coord = &builder_entry.entry.coord;

            loop {
                let (cell_idx, depth) = Self::builder_cell_for_coord(coord, &dynamic_cells);
                let cell = &mut dynamic_cells[cell_idx];

                if cell.contents.is_full() && depth < max_depth && cell.extent.can_subdivide() {
                    Self::subdivide_builder_cell(cell_idx, entries, &mut dynamic_cells);
                } else if let BuilderCellContents::Values(v) = &mut cell.contents {
                    v.push(builder_entry.source_index);
                    break;
                }
            }
        }

        // Group cells by depth and arrange in Z/Lebesgue/Morton order
        let mut ordered_cells: Vec<_> = dynamic_cells.into_iter().enumerate().collect();
        let mut queue = VecDeque::with_capacity(ordered_cells.len());

        queue.push_back(0);

        let mut new_len = 1;

        while let Some(old_idx) = queue.pop_front() {
            let (new_idx, cell) = &mut ordered_cells[old_idx];

            if let BuilderCellContents::Cells(offset) = cell.contents {
                let old_base = old_idx + offset;
                let new_base = new_len;

                new_len += 8;

                cell.contents = BuilderCellContents::Cells(new_base - *new_idx);

                for i in 0..8 {
                    ordered_cells[old_base + i].0 = new_base + i;
                    queue.push_back(old_base + i);
                }
            }
        }

        ordered_cells.sort_by(|(li, _), (ri, _)| li.cmp(ri));

        ordered_cells.into_iter().map(|(_, c)| c).collect()
    }

    /// Subdivides the cell at `index` into eight sub-cells and distributes its contents into the
    /// newly created cells.
    fn subdivide_builder_cell(
        index: usize,
        entries: &[OctreeBuilderEntry<N, T>],
        cells: &mut Vec<OctreeCell<N, BuilderCellContents>>,
    ) where
        N: OctreeNum,
    {
        if let Some(contents) = cells[index].contents.take_values() {
            cells[index].contents = BuilderCellContents::Cells(cells.len());

            cells.extend(
                cells[index]
                    .contained_cubes()
                    .map(OctreeCell::<N, BuilderCellContents>::from),
            );

            for (i, coord) in contents.into_iter().map(|i| (i, &entries[i].entry.coord)) {
                let (target_idx, _) = Self::builder_cell_for_coord(coord, cells);

                cells[target_idx].contents.push(i);
            }
        }
    }

    /// Converts the `entries` into into a `Vec<OctreeBuilderEntry<N, T>>` while tracking the
    /// extents. Returns `(extent_min, extent_max, builder_entries)`
    fn to_builder_entries<I, Coord>(
        entries: I,
    ) -> (Vector3<N>, Vector3<N>, Vec<OctreeBuilderEntry<N, T>>)
    where
        I: IntoIterator<Item = (Coord, T)>,
        Coord: Into<Vector3<N>>,
        N: OctreeNum,
    {
        let mut extent_min: Option<Vector3<N>> = None;
        let mut extent_max: Option<Vector3<N>> = None;

        let builder_entries: Vec<OctreeBuilderEntry<N, T>> = entries
            .into_iter()
            .enumerate()
            .map(|(i, (c, d))| {
                let c = c.into();

                if let (Some(min), Some(max)) = (extent_min, extent_max) {
                    extent_min = nalgebra::partial_min(&min, &c).copied();
                    extent_max = nalgebra::partial_max(&max, &c).copied();
                } else {
                    extent_min = Some(c);
                    extent_max = Some(c);
                }

                (i, (c, d)).into()
            })
            .collect();

        (
            extent_min.unwrap_or_default(),
            extent_max.unwrap_or_default(),
            builder_entries,
        )
    }
}

/// Index for a cell or value with values always ordering before cells.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CellOrValue {
    Cell(usize),
    Value(usize),
}

impl CellOrValue {
    fn inner(&self) -> &usize {
        match self {
            CellOrValue::Cell(i) | CellOrValue::Value(i) => i,
        }
    }
}

impl Ord for CellOrValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (CellOrValue::Cell(_), CellOrValue::Value(_)) => std::cmp::Ordering::Greater,
            (CellOrValue::Value(_), CellOrValue::Cell(_)) => std::cmp::Ordering::Less,
            _ => self.inner().cmp(other.inner()),
        }
    }
}

impl PartialOrd for CellOrValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Produces all entries in a [[`SparseOctree`]] based on their distance from a starting point.
#[derive(Debug, Clone)]
pub struct IterByNearest<'o, N, T, DistSq = N> {
    coord: Vector3<N>,
    octree: &'o SparseOctree<N, T>,
    queue: BinaryHeap<Reverse<(DistSq, CellOrValue)>>,
}

impl<N, T, DistSq> IterByNearest<'_, N, T, DistSq> {
    /// Get a reference to the focal point that produced entries are ordered by distance to.
    pub fn focus(&self) -> &Vector3<N> {
        &self.coord
    }
}

impl<'o, N, T, DistSq> IterByNearest<'o, N, T, DistSq> {
    fn new(coord: Vector3<N>, octree: &'o SparseOctree<N, T>) -> Self
    where
        DistSq: Default + Ord,
    {
        Self {
            coord,
            octree,
            queue: BinaryHeap::from([Reverse((DistSq::default(), CellOrValue::Cell(0)))]),
        }
    }
}

/// Gets the squared Euclidean distance between `a` and `b` while accounting for the fact that `N`
/// may not be able to represent negative numbers.
fn dist_squared<N>(a: &Vector3<N>, b: &Vector3<N>) -> N
where
    N: OctreeNum,
{
    let diff = abs_diff(a, b);

    diff.dot(&diff)
}

impl<'o, N, T, DistSq> Iterator for IterByNearest<'o, N, T, DistSq>
where
    N: OctreeNum,
    DistSq: Ord + TryFrom<N>,
{
    type Item = (DistSq, &'o OctreeEntry<N, T>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(Reverse((dist_sq, entry))) = self.queue.pop() {
                match entry {
                    CellOrValue::Cell(i) => {
                        let cell = &self.octree.cells[i];

                        match &cell.contents {
                            CellContents::None => {}
                            CellContents::Values(range) => {
                                for i in range.clone() {
                                    let entry = &self.octree.entries[i];
                                    let dist_sq = dist_squared(&self.coord, &entry.coord);

                                    // If a NaN is encountered, discard the entry
                                    if let Ok(dist_sq) = dist_sq.try_into() {
                                        self.queue.push(Reverse((dist_sq, CellOrValue::Value(i))));
                                    }
                                }
                            }
                            CellContents::Cells(offset) => {
                                for ci in 0..8 {
                                    let i = offset + ci;
                                    let cell = &self.octree.cells[i];
                                    let dist_sq =
                                        cell.surface_exterior_distance_squared(&self.coord);

                                    if let Ok(dist_sq) = dist_sq.try_into() {
                                        self.queue.push(Reverse((dist_sq, CellOrValue::Cell(i))));
                                    }
                                }
                            }
                        }
                    }
                    CellOrValue::Value(i) => {
                        break Some((dist_sq, &self.octree.entries[i]));
                    }
                }
            } else {
                break None;
            }
        }
    }
}

impl<N, T> SparseOctree<N, T> {
    /// Create a new [[`SparseOctree`]] with a maximum depth of `max_depth` from `entries`. Entries
    /// ror each cell are co-located with sub-cells always appearing forward in memory.
    pub fn new<I, Coord>(entries: I, max_depth: usize) -> Self
    where
        I: IntoIterator<Item = (Coord, T)>,
        Coord: Into<Vector3<N>>,
        N: OctreeNum,
    {
        let (extent_min, extent_max, entries) = Self::to_builder_entries(entries);
        let center = extent_min + (extent_max - extent_min) / N::TWO;

        // This Octree implementation is a cube for simplicity
        let extent = if N::MUST_BE_POWER_OF_TWO {
            (extent_max - center)
                .max()
                .to_power_of_two()
                .expect("Extent should fit within integer width chosen")
        } else {
            (extent_max - center).max()
        };

        Self::build_from_parts(center, extent, entries, max_depth)
    }

    /// Iterate over all entries in the [[`SparseOctree`]]. While entries are in a deterministic
    /// order, the sparse nature of the octree means that there is no definite relation between
    /// index and where the entry lies within the cell structure.
    pub fn iter(&self) -> impl Iterator<Item = &'_ OctreeEntry<N, T>> {
        self.entries.iter()
    }

    /// Iterate over *all* entries in the [[`SparseOctree`]] based on their distance to `coord`.
    pub fn iter_by_nearest<DistSq>(&self, coord: Vector3<N>) -> IterByNearest<'_, N, T, DistSq>
    where
        DistSq: Ord + Default + TryFrom<N>,
    {
        IterByNearest::new(coord, self)
    }
}
