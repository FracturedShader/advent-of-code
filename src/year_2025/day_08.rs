use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    io::BufRead,
};

use nalgebra::Vector3;

use crate::common::{self, IterByNearest, OctreeEntry, OctreeNum, SparseOctree, UndirectedEdge};

fn coord_from_line<S>(line: S) -> nalgebra::Vector3<u64>
where
    S: AsRef<str>,
{
    const NOT_ENOUGH_NUMBERS_ERR: &str = "Lines should be three numbers separated by commas";

    let mut numbers = line
        .as_ref()
        .split(',')
        .map(|p| p.parse::<u64>().expect(NOT_ENOUGH_NUMBERS_ERR));

    nalgebra::Vector3::new(
        numbers.next().expect(NOT_ENOUGH_NUMBERS_ERR),
        numbers.next().expect(NOT_ENOUGH_NUMBERS_ERR),
        numbers.next().expect(NOT_ENOUGH_NUMBERS_ERR),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OrderedOctree<N>(SparseOctree<N, usize>);

impl<N> OrderedOctree<N> {
    fn new<I, Coord>(iter: I, max_depth: usize) -> Self
    where
        I: IntoIterator<Item = Coord>,
        Coord: Into<Vector3<N>>,
        N: OctreeNum,
    {
        Self(SparseOctree::new(
            iter.into_iter().enumerate().map(|(i, c)| (c, i)),
            max_depth,
        ))
    }
}

#[derive(Debug, Clone)]
struct GreaterOctreeNeighbors<'o, N, DistSq> {
    focus_index: usize,
    inner: IterByNearest<'o, N, usize, DistSq>,
}

impl<'o, N, DistSq> GreaterOctreeNeighbors<'o, N, DistSq> {
    fn new(ordered_octree: &'o OrderedOctree<N>, coord: Vector3<N>, coord_index: usize) -> Self
    where
        DistSq: Ord + Default + TryFrom<N>,
    {
        Self {
            focus_index: coord_index,
            inner: ordered_octree.0.iter_by_nearest(coord),
        }
    }
}

impl<'o, N, DistSq> Iterator for GreaterOctreeNeighbors<'o, N, DistSq>
where
    N: OctreeNum,
    DistSq: Ord + TryFrom<N>,
{
    type Item = <IterByNearest<'o, N, usize, DistSq> as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .by_ref()
            .find(|entry| entry.1.value > self.focus_index)
    }
}

#[derive(Debug, Clone)]
struct EdgeDescriptors<'o, N, DistSq> {
    end: (DistSq, &'o OctreeEntry<N, usize>),
    inner: GreaterOctreeNeighbors<'o, N, DistSq>,
}

impl<N, DistSq> Ord for EdgeDescriptors<'_, N, DistSq>
where
    N: Eq,
    DistSq: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.end.0.cmp(&other.end.0) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.end.1.value.cmp(&other.end.1.value) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.inner.focus_index.cmp(&other.inner.focus_index)
    }
}

impl<N, DistSq> PartialOrd for EdgeDescriptors<'_, N, DistSq>
where
    N: PartialEq,
    DistSq: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.end.0.partial_cmp(&other.end.0) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.end.1.value.partial_cmp(&other.end.1.value) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.inner.focus_index.partial_cmp(&other.inner.focus_index)
    }
}

impl<N, DistSq> PartialEq for EdgeDescriptors<'_, N, DistSq>
where
    N: PartialEq,
    DistSq: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.end == other.end && self.inner.focus_index == other.inner.focus_index
    }
}

impl<N, DistSq> Eq for EdgeDescriptors<'_, N, DistSq>
where
    N: Eq,
    DistSq: Eq,
{
}

#[derive(Debug)]
struct TotalEdgesByLength<'o, N, DistSq = N>(BinaryHeap<Reverse<EdgeDescriptors<'o, N, DistSq>>>);

impl<'o, N, DistSq> TotalEdgesByLength<'o, N, DistSq> {
    fn new(octree: &'o OrderedOctree<N>) -> Self
    where
        N: OctreeNum,
        DistSq: Default + Ord + TryFrom<N>,
    {
        Self(
            octree
                .0
                .iter()
                .filter_map(|entry| {
                    let mut iter = GreaterOctreeNeighbors::new(octree, entry.coord, entry.value);

                    if let Some(end) = iter.next() {
                        Some(Reverse(EdgeDescriptors { end, inner: iter }))
                    } else {
                        None
                    }
                })
                .collect(),
        )
    }
}

impl<N, DistSq> Iterator for TotalEdgesByLength<'_, N, DistSq>
where
    N: OctreeNum,
    DistSq: Ord + TryFrom<N>,
{
    type Item = UndirectedEdge<(Vector3<N>, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Reverse(EdgeDescriptors {
            end: current,
            inner: mut rest,
        })) = self.0.pop()
        {
            let origin = *rest.inner.focus();
            let index = rest.focus_index;

            if let Some(next) = rest.next() {
                self.0.push(Reverse(EdgeDescriptors {
                    end: next,
                    inner: rest,
                }));
            }

            Some(UndirectedEdge(
                (origin, index),
                (current.1.coord, current.1.value),
            ))
        } else {
            None
        }
    }
}

fn three_largest_circuits(
    coordinates: &[Vector3<u64>],
    max_connections: usize,
) -> (usize, usize, usize) {
    let octree = OrderedOctree::new(coordinates.iter().copied(), 8);

    let mut edges: HashMap<usize, Vec<usize>> = HashMap::with_capacity(max_connections);

    for UndirectedEdge((_, i0), (_, i1)) in
        TotalEdgesByLength::<_>::new(&octree).take(max_connections)
    {
        edges.entry(i0).or_default().push(i1);
        edges.entry(i1).or_default().push(i0);
    }

    let mut visited = HashSet::with_capacity(edges.len());
    let mut visit_queue = VecDeque::with_capacity(max_connections);
    let mut circuit_sizes: Vec<usize> = Vec::with_capacity(max_connections);

    for start_idx in edges.keys() {
        visit_queue.push_back(start_idx);

        let mut circuit_size = 0;

        while let Some(visiting) = visit_queue.pop_front() {
            if visited.contains(visiting) {
                continue;
            }

            circuit_size += 1;

            visited.insert(*visiting);

            visit_queue.extend(&edges[visiting]);
        }

        if circuit_size != 0 {
            circuit_sizes.push(circuit_size);
        }
    }

    circuit_sizes.sort_unstable();

    (
        circuit_sizes.pop().unwrap_or(1),
        circuit_sizes.pop().unwrap_or(1),
        circuit_sizes.pop().unwrap_or(1),
    )
}

fn last_link_to_link_all(coordinates: &[Vector3<u64>]) -> Option<UndirectedEdge<Vector3<u64>>> {
    let mut box_circuit: HashMap<_, _> = (0..coordinates.len()).map(|i| (i, i)).collect();
    let mut circuit_boxes: HashMap<_, _> =
        box_circuit.iter().map(|(&b, &c)| (c, vec![b])).collect();

    let octree = OrderedOctree::new(coordinates.iter().copied(), 8);

    for UndirectedEdge((v0, i0), (v1, i1)) in TotalEdgesByLength::<_>::new(&octree) {
        let c0 = box_circuit.get(&i0).unwrap();
        let c1 = box_circuit.get(&i1).unwrap();

        if c0 == c1 {
            continue;
        }

        let c_new = *c0.min(c1);
        let c_old = *c0.max(c1);
        let old_set = circuit_boxes.remove(&c_old).unwrap();

        circuit_boxes
            .get_mut(&c_new)
            .unwrap()
            .extend(old_set.into_iter().inspect(|i| {
                box_circuit.insert(*i, c_new);
            }));

        if circuit_boxes.len() == 1 {
            return Some(UndirectedEdge(v0, v1));
        }
    }

    None
}

pub fn part_01() {
    let coords: Vec<_> = common::puzzle_input("2025-08")
        .unwrap()
        .lines()
        .map_while(Result::ok)
        .map(coord_from_line)
        .collect();

    let (n0, n1, n2) = three_largest_circuits(&coords, 1000);
    let product = n0 * n1 * n2;

    println!("Product of three largest circuits of shortest 1000 edges: {product}");
}

pub fn part_02() {
    let coords: Vec<_> = common::puzzle_input("2025-08")
        .unwrap()
        .lines()
        .map_while(Result::ok)
        .map(coord_from_line)
        .collect();

    if let Some(edge) = last_link_to_link_all(&coords) {
        let product = edge.0.x * edge.1.x;
        println!("Product of last link X coordinates: {product}");
    } else {
        println!(
            "Problem is ill-formed. Maybe not enough coordinates: {} < 2?",
            coords.len()
        );
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn data() -> &'static str {
        r#"162,817,812
57,618,57
906,360,560
592,479,940
352,342,300
466,668,158
542,29,236
431,825,988
739,650,466
52,470,668
216,146,977
819,987,18
117,168,530
805,96,715
346,949,466
970,615,88
941,993,340
862,61,35
984,92,344
425,690,689"#
    }

    #[test]
    fn link_ten_shortest() {
        let coords: Vec<_> = data().lines().map(coord_from_line).collect();
        let circuit_sizes = three_largest_circuits(&coords, 10);

        assert_eq!(circuit_sizes, (5, 4, 2));
    }

    #[test]
    fn last_edge_to_single_circuit() {
        let coords: Vec<_> = data().lines().map(coord_from_line).collect();
        let edge = last_link_to_link_all(&coords);

        assert_eq!(
            edge,
            Some(UndirectedEdge(
                Vector3::new(216, 146, 977),
                Vector3::new(117, 168, 530)
            ))
        );
    }
}
