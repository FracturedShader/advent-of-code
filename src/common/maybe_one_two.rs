/// Intended to be exactly like [[`Option`]], but with up to two values of the same type.
/// Implementation will match [[`Option`]] more closely as different puzzles need more.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum MaybeOneTwo<T> {
    #[default]
    None,
    One(T),
    Two(T, T),
}

impl<T> Iterator for MaybeOneTwo<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let prev = std::mem::replace(self, Self::None);

        match prev {
            Self::None => None,
            Self::One(v) => Some(v),
            Self::Two(v1, v2) => {
                *self = Self::One(v2);

                Some(v1)
            }
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        match self {
            Self::None => 0,
            Self::One(_) => 1,
            Self::Two(_, _) => 2,
        }
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        match self {
            Self::None => None,
            Self::One(v) | Self::Two(_, v) => Some(v),
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let prev = std::mem::replace(self, Self::None);

        match (prev, n) {
            (Self::One(v) | Self::Two(v, _), 0) | (Self::Two(_, v), 1) => Some(v),
            _ => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::None => (0, Some(0)),
            Self::One(_) => (1, Some(1)),
            Self::Two(_, _) => (2, Some(2)),
        }
    }
}
