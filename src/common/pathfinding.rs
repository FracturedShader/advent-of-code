use std::{
    collections::{BinaryHeap, HashMap},
    hash::Hash,
};

/// Determines the priority of a node based on the total `expected` cost of a path through it and
/// the current known `cost` at time of processing.
///
/// Ordering:
/// - Lower `expected` have higher priority
/// - When the `expected` cost is the same, the furthest path (higher `cost`) takes precedence.
///   This results in a depth-first ordering by picking the node that's closer to the goal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PathHeuristic<C> {
    expected: C,
    cost: C,
}

impl<C> PartialOrd for PathHeuristic<C>
where
    C: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match other.expected.partial_cmp(&self.expected) {
            Some(core::cmp::Ordering::Equal) => self.cost.partial_cmp(&other.cost),
            ord => ord,
        }
    }
}

impl<C> Ord for PathHeuristic<C>
where
    C: Ord + Eq,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match other.expected.cmp(&self.expected) {
            core::cmp::Ordering::Equal => self.cost.cmp(&other.cost),
            ord => ord,
        }
    }
}

/// Entry for a queue where `node`s can be ordered. This can significantly speed up a search when
/// the search space is very large and neighbors overwhelmingly make nearly identical progress.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct OrderedQueueEntry<T, C> {
    path_heuristic: PathHeuristic<C>,
    node: T,
}

/// Entry for a queue where `node`s cannot have any form of ordering. This should only be the case
/// for procedurally generated `node`s in an unbounded multi-dimensional space.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UnorderedQueueEntry<T, C> {
    path_heuristic: PathHeuristic<C>,
    node: T,
}

impl<T, C> PartialOrd for UnorderedQueueEntry<T, C>
where
    C: PartialOrd,
    T: Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.path_heuristic.partial_cmp(&other.path_heuristic)
    }
}

impl<T, C> Ord for UnorderedQueueEntry<T, C>
where
    C: Ord + Eq,
    T: Eq,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.path_heuristic.cmp(&other.path_heuristic)
    }
}

/// Keeps track of the `cost` of reaching a particular node, the `previous` node in the path, and
/// whether or not this node has been `visited` since its `cost` was last updated.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SearchData<T, C> {
    previous: Option<T>,
    cost: C,
    visited: bool,
}

/// Contains a `node` in the path found by A* as well as the cost of reaching that node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathPoint<T, C> {
    pub node: T,
    pub cost: C,
}

/// Iterator which produces the shortest path from the goal node to the starting node. While this
/// is the opposite of what A* is tasked with producing, this avoids extra allocations when the
/// specifics of the path are not terribly relevant. The forward path can be produced by reversing
/// this iterator.
pub struct ReversePathBuilder<T, C> {
    back_edges: HashMap<T, SearchData<T, C>>,
    focus: Option<T>,
}

impl<T, C> ReversePathBuilder<T, C>
where
    T: Hash + Eq,
    C: Copy + Default,
{
    fn from_search(data: HashMap<T, SearchData<T, C>>, end: T) -> Self {
        Self {
            back_edges: data,
            focus: Some(end),
        }
    }
}

impl<T, C> Iterator for ReversePathBuilder<T, C>
where
    T: Clone + Hash + Eq + std::fmt::Debug,
    C: Default,
{
    type Item = PathPoint<T, C>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.focus.take() {
            let cost =
                if let Some(SearchData { previous, cost, .. }) = self.back_edges.remove(&node) {
                    self.focus = previous;

                    cost
                } else {
                    C::default()
                };

            Some(PathPoint { node, cost })
        } else {
            None
        }
    }
}

/// Tracks the `start` node in `data` and inserts it into the `queue` for processing. This method
/// exists primarily to uphold tracking invariants.
fn initialize<T, D, C, Q, M>(
    start: T,
    data: &mut HashMap<T, D>,
    queue: &mut BinaryHeap<Q>,
    mut make_entries: M,
) where
    T: Clone + Hash + Eq,
    C: Default,
    Q: Ord,
    M: FnMut(T, Option<T>, C, C) -> (D, Q),
{
    let (de, qe) = make_entries(start.clone(), None, C::default(), C::default());

    data.insert(start, de);
    queue.push(qe);
}

/// The heart of the A* algorithm. Won't process the `node` if it has already been visited since
/// its last `cost` update. When visiting, enqueues all of `node`'s neighbors if their total `cost`
/// is improved through this `node`.
fn visit_node<T, C, Q, N, I, H, M>(
    node: &T,
    data: &mut HashMap<T, SearchData<T, C>>,
    queue: &mut BinaryHeap<Q>,
    mut neighbors: N,
    mut heuristic: H,
    mut make_entries: M,
) where
    T: Clone + Hash + Eq,
    C: Copy + PartialOrd + core::ops::Add<C, Output = C>,
    Q: Ord,
    N: FnMut(&T) -> Option<I>,
    I: IntoIterator<Item = (C, T)>,
    H: FnMut(&T) -> C,
    M: FnMut(T, Option<T>, C, C) -> (SearchData<T, C>, Q),
{
    let data_entry = data.get_mut(node).unwrap();

    if data_entry.visited {
        return;
    }

    data_entry.visited = true;

    let cost = data_entry.cost;

    if let Some(iter) = neighbors(node) {
        queue.extend(iter.into_iter().filter_map(|(dist, neighbor)| {
            let neighbor_cost = cost + dist;

            if let Some(entry) = data.get(&neighbor) {
                if neighbor_cost >= entry.cost {
                    return None;
                }
            }

            let (de, qe) = make_entries(
                neighbor.clone(),
                Some(node.clone()),
                neighbor_cost + heuristic(&neighbor),
                neighbor_cost,
            );

            data.insert(neighbor, de);

            Some(qe)
        }));
    }
}

/// Finds the shortest path (as long as there are no negative edges) from `start` to
/// `is_goal(&node)` via the [A* search
/// algorithm](https://en.wikipedia.org/wiki/A*_search_algorithm).
///
/// - `start`: The node to start the search from.
/// - `neighbors`: Should produce all possible neighbors, and their distance, for any node.
/// - `heuristic`: Estimate of how close a node is to a goal node. If the provided estimate is
///   constant regardless of the node, the algorithm is equivalent to Dijkstra's algorithm.
/// - `is_goal`: Indicates whether the node being visited is a goal node and the algorithm should
///   terminate.
///
/// Returns a [`ReversePathBuilder`] which produces the path from the reached goal back to `start`
/// (inclusive). This avoids extra allocations when the exact path is irrelevant and can be
/// reversed to produce the forward path.
///
/// **Note**: It is not necessary to supply nodes directly. If nodes are expensive to clone, it may
/// be worth using a hashing/indexing scheme. Arguments are `FnMut` to even allow this option with
/// procedurally generated graphs.
pub fn a_star<T, C, N, I, H, G>(
    start: T,
    mut neighbors: N,
    mut heuristic: H,
    mut is_goal: G,
) -> Option<ReversePathBuilder<T, C>>
where
    T: Clone + Hash + Eq + Ord,
    C: Copy + Default + Ord + core::ops::Add<C, Output = C>,
    N: FnMut(&T) -> Option<I>,
    I: IntoIterator<Item = (C, T)>,
    H: FnMut(&T) -> C,
    G: FnMut(&T) -> bool,
{
    let mut data: HashMap<T, SearchData<T, C>> = HashMap::new();
    let mut queue = BinaryHeap::new();

    let make_entries = |node: T, previous: Option<T>, expected: C, cost: C| {
        (
            SearchData {
                previous,
                cost,
                visited: false,
            },
            OrderedQueueEntry {
                path_heuristic: PathHeuristic { expected, cost },
                node,
            },
        )
    };

    initialize(start, &mut data, &mut queue, make_entries);

    while let Some(OrderedQueueEntry { node, .. }) = queue.pop() {
        if is_goal(&node) {
            return Some(ReversePathBuilder::from_search(data, node));
        }

        visit_node(
            &node,
            &mut data,
            &mut queue,
            &mut neighbors,
            &mut heuristic,
            make_entries,
        );
    }

    None
}

/// This is a specialized version of [`a_star`] for when nodes cannot be given any kind of ordering
/// (not even an arbitrary one). It is **never** faster than [`a_star`] and should only be used if
/// absolutely necessary.
pub fn a_star_unordered<T, C, N, H, G, I>(
    start: T,
    mut neighbors: N,
    mut heuristic: H,
    mut is_goal: G,
) -> Option<ReversePathBuilder<T, C>>
where
    T: Clone + Hash + Eq,
    C: Copy + Default + Ord + core::ops::Add<C, Output = C>,
    N: FnMut(&T) -> Option<I>,
    H: FnMut(&T) -> C,
    G: FnMut(&T) -> bool,
    I: IntoIterator<Item = (C, T)>,
{
    let mut data: HashMap<T, SearchData<T, C>> = HashMap::new();
    let mut queue = BinaryHeap::new();

    let make_entries = |node: T, previous: Option<T>, expected: C, cost: C| {
        (
            SearchData {
                previous,
                cost,
                visited: false,
            },
            UnorderedQueueEntry {
                path_heuristic: PathHeuristic { expected, cost },
                node,
            },
        )
    };

    initialize(start, &mut data, &mut queue, make_entries);

    while let Some(UnorderedQueueEntry { node, .. }) = queue.pop() {
        if is_goal(&node) {
            return Some(ReversePathBuilder::from_search(data, node));
        }

        visit_node(
            &node,
            &mut data,
            &mut queue,
            &mut neighbors,
            &mut heuristic,
            make_entries,
        );
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn queue_ordering() {
        let a = UnorderedQueueEntry {
            path_heuristic: PathHeuristic {
                expected: 10,
                cost: 5,
            },
            node: 1,
        };

        let b = UnorderedQueueEntry {
            path_heuristic: PathHeuristic {
                expected: 11,
                cost: 5,
            },
            node: 2,
        };

        let c = UnorderedQueueEntry {
            path_heuristic: PathHeuristic {
                expected: 10,
                cost: 6,
            },
            node: 3,
        };

        assert!(a > b);
        assert!(a < c);

        let mut queue = BinaryHeap::new();
        queue.push(a.clone());
        queue.push(b.clone());
        queue.push(c.clone());

        assert_eq!(queue.pop(), Some(c));
        assert_eq!(queue.pop(), Some(a));
        assert_eq!(queue.pop(), Some(b));
    }
}
