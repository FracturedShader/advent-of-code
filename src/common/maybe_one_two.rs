/// Intended to be exactly like [[`Option`]], but with up to two values of the same type.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub enum MaybeOneTwo<T> {
    #[default]
    None,
    One(T),
    Two(T, T),
}

impl<T> MaybeOneTwo<T> {
    pub const fn as_mut(&mut self) -> MaybeOneTwo<&mut T> {
        match self {
            Self::None => MaybeOneTwo::None,
            Self::One(ref mut v) => MaybeOneTwo::One(v),
            Self::Two(ref mut v1, ref mut v2) => MaybeOneTwo::Two(v1, v2),
        }
    }

    pub const fn as_ref(&self) -> MaybeOneTwo<&T> {
        match self {
            Self::None => MaybeOneTwo::None,
            Self::One(ref v) => MaybeOneTwo::One(v),
            Self::Two(ref v1, ref v2) => MaybeOneTwo::Two(v1, v2),
        }
    }
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
            (Self::One(v), 0) | (Self::Two(v, _), 0) | (Self::Two(_, v), 1) => Some(v),
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
