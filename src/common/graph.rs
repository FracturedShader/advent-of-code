use std::hash::Hash;

/// Represents an undirected edge where both ends are effectively interchangable. Comparison and
/// ordering operations take this into account. In order to support hashing for non-ordered types,
/// equal edges may not produce equal hashes. If this is necessary, construct the edges according
/// to a global unique ordering.
#[derive(Debug, Copy, Clone, Eq)]
pub struct UndirectedEdge<T>(pub T, pub T);

impl<T> UndirectedEdge<T>
where
    T: PartialOrd,
{
    pub fn max_end(&self) -> &T {
        if self.0 >= self.1 {
            &self.0
        } else {
            &self.1
        }
    }

    pub fn min_end(&self) -> &T {
        if self.0 <= self.1 {
            &self.0
        } else {
            &self.1
        }
    }
}

impl<T> Hash for UndirectedEdge<T>
where
    T: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<T> PartialEq for UndirectedEdge<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
    }
}

impl<T> PartialOrd for UndirectedEdge<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.min_end().partial_cmp(other.min_end()) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        self.max_end().partial_cmp(other.max_end())
    }
}

impl<T> Ord for UndirectedEdge<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.min_end().cmp(other.min_end()) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        self.max_end().cmp(other.max_end())
    }
}
