use itertools::Itertools;
use std::{io::BufRead, ops::BitOr};
use thiserror::Error;

/// An item in an Elf's rucksack. Guaranteed to be in the range `0..52`
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Item(u32);

impl Item {
    fn priority(self) -> u32 {
        self.0 + 1
    }
}

#[derive(Error, Copy, Clone, Debug, PartialEq, Eq)]
#[error("ASCII character was not one of a-z or A-Z")]
struct NotItemError();

/// Attempts to convert ASCII input `[a-zA-Z]` into an `Item`
impl TryFrom<u8> for Item {
    type Error = NotItemError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'a'..=b'z' => Ok(Item((value - b'a').into())),
            b'A'..=b'Z' => Ok(Item((value - b'A' + 26).into())),
            _ => Err(NotItemError()),
        }
    }
}

#[derive(Error, Copy, Clone, Debug, PartialEq, Eq)]
#[error("Number passed not in the range 0..52")]
struct ItemRangeError();

/// Essentially a compact hash set of an item as the range of possible item values allows perfect
/// hashing in the bits of a `u64`
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct ItemSet(u64);

impl ItemSet {
    fn intersection(self, other: Self) -> Self {
        ItemSet(self.0 & other.0)
    }

    fn union(self, other: Self) -> Self {
        ItemSet(self.0 | other.0)
    }
}

#[derive(Error, Copy, Clone, Debug, PartialEq, Eq)]
#[error("Number passed has value greater than 1 << 51")]
struct SetRangeError();

/// Essentially hash an `Item` into an `ItemSet`
impl From<Item> for ItemSet {
    fn from(value: Item) -> Self {
        ItemSet(1u64 << value.0)
    }
}

impl FromIterator<Item> for ItemSet {
    fn from_iter<T: IntoIterator<Item = Item>>(iter: T) -> Self {
        ItemSet(
            iter.into_iter()
                .map(|i| 1u64 << i.0)
                .reduce(BitOr::bitor)
                .unwrap_or(0),
        )
    }
}

#[derive(Error, Copy, Clone, Debug, PartialEq, Eq)]
#[error("ItemSet does not have exactly one bit set.")]
struct SetNotSingleItemError();

/// Converts an `ItemSet` representing a single `Item` into that `Item`
impl TryFrom<ItemSet> for Item {
    type Error = anyhow::Error;

    fn try_from(value: ItemSet) -> Result<Self, Self::Error> {
        if value.0.count_ones() != 1 {
            Err(SetNotSingleItemError().into())
        } else {
            Ok(Item((value.0 - 1).count_ones()))
        }
    }
}

fn common_rucksack_item<I>(rucksack_items: I, set_every: usize) -> Option<Item>
where
    I: Iterator<Item = Item>,
{
    rucksack_items
        .chunks(set_every)
        .into_iter()
        .map(ItemSet::from_iter)
        .reduce(ItemSet::intersection)
        .map(|ii| Item::try_from(ii).ok())
        .flatten()
}

pub fn part_01(reader: Option<impl BufRead>) {
    let priority_sum = reader
        .expect("This problem requires data input")
        .lines()
        .flatten()
        .filter_map(|l| {
            let bytes = l.bytes();
            let len = bytes.len();

            if len == 0 {
                return None;
            }

            let items = bytes.map(|b| Item::try_from(b).expect("Invalid value for rucksack item"));

            Some(
                common_rucksack_item(items, len / 2)
                    .expect("No common item found between the two halves")
                    .priority(),
            )
        })
        .sum::<u32>();

    println!(
        "Sum of priorities present in both compartments of each rucksack: {}",
        priority_sum
    );
}

pub fn part_02(reader: Option<impl BufRead>) {
    let priority_sum = reader
        .expect("This problem requires data input")
        .lines()
        .flatten()
        .filter_map(|l| {
            let bytes = l.bytes();
            let len = bytes.len();

            if len == 0 {
                return None;
            }

            Some(
                bytes
                    .map(|b| Item::try_from(b).expect("Invalid value for rucksack item"))
                    .collect::<ItemSet>(),
            )
        })
        .chunks(3)
        .into_iter()
        .map(|i| {
            Item::try_from(i.reduce(ItemSet::intersection).unwrap())
                .unwrap()
                .priority()
        })
        .sum::<u32>();

    println!("Sum of priorities common within groups: {}", priority_sum);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn index_fill() {
        let full_set: ItemSet = (b'a'..=b'z')
            .chain(b'A'..=b'Z')
            .flat_map(Item::try_from)
            .collect();

        assert_eq!(full_set.0, (1u64 << 52) - 1);
    }

    #[test]
    fn rucksack_common_item() {
        let input = r#"vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw"#;

        let common = input
            .lines()
            .map(|l| {
                common_rucksack_item(l.bytes().map(|b| Item::try_from(b).unwrap()), l.len() / 2)
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let expected = vec![b'p', b'L', b'P', b'v', b't', b's']
            .into_iter()
            .flat_map(Item::try_from)
            .collect::<Vec<_>>();

        assert_eq!(common, expected);
    }

    #[test]
    fn group_common_item() {
        let input = r#"vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw"#;

        let group_items = input
            .lines()
            .map(|l| l.bytes().flat_map(Item::try_from).collect::<ItemSet>())
            .chunks(3)
            .into_iter()
            .map(|i| Item::try_from(i.reduce(ItemSet::intersection).unwrap()).unwrap())
            .collect::<Vec<_>>();

        let expected = vec![b'r', b'Z']
            .into_iter()
            .flat_map(Item::try_from)
            .collect::<Vec<_>>();

        assert_eq!(group_items, expected);
    }
}
