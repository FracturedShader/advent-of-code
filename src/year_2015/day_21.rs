use std::{
    iter::Sum,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::common::a_star;

/// Stats common to both creatures and items (`damage` and `armor`)
#[derive(Default, Copy, Debug, Clone, PartialEq, Eq)]
struct CoreStats {
    pub damage: i32,
    pub armor: i32,
}

impl Ord for CoreStats {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher damage + armor is better (favoring damage)
        (other.damage + other.armor)
            .cmp(&(self.damage + self.armor))
            .then(other.damage.cmp(&self.damage))
            .then(other.armor.cmp(&self.armor))
    }
}

impl PartialOrd for CoreStats {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for CoreStats {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            damage: self.damage + rhs.damage,
            armor: self.armor + rhs.armor,
        }
    }
}

impl AddAssign for CoreStats {
    fn add_assign(&mut self, rhs: Self) {
        self.damage += rhs.damage;
        self.armor += rhs.armor;
    }
}

impl Sub for CoreStats {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            damage: self.damage - rhs.damage,
            armor: self.armor - rhs.armor,
        }
    }
}

impl SubAssign for CoreStats {
    fn sub_assign(&mut self, rhs: Self) {
        self.damage -= rhs.damage;
        self.armor -= rhs.armor;
    }
}

impl Sum for CoreStats {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut stats = Self::default();

        for it in iter {
            stats += it;
        }

        stats
    }
}

/// An item is little more than a `name`d collection of `stats` available for a `price`
#[derive(Debug, Clone, PartialEq, Eq)]
struct Item {
    pub name: String,
    pub price: i32,
    pub stats: CoreStats,
}

impl Item {
    fn from_parts(name: &str, price: i32, damage: i32, armor: i32) -> Self {
        Self {
            name: name.to_owned(),
            price,
            stats: CoreStats { damage, armor },
        }
    }
}

/// The `Shop` contains every available [`Item`], by category, but only one of each.
///
/// Items are guaranteed to be sorted by price (low-to-high) within each category.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Shop {
    pub weapons: Vec<Item>,
    pub armor: Vec<Item>,
    pub rings: Vec<Item>,
}

impl Shop {
    /// Creates a brand new, fully-stocked, `Shop` (according to the problem)
    fn new() -> Self {
        Self {
            weapons: vec![
                Item::from_parts("Dagger", 8, 4, 0),
                Item::from_parts("Shortsword", 10, 5, 0),
                Item::from_parts("Warhammer", 25, 6, 0),
                Item::from_parts("Longsword", 40, 7, 0),
                Item::from_parts("Greataxe", 74, 8, 0),
            ],
            armor: vec![
                Item::from_parts("Leather", 13, 0, 1),
                Item::from_parts("Chainmail", 31, 0, 2),
                Item::from_parts("Splintmail", 53, 0, 3),
                Item::from_parts("Bandedmail", 75, 0, 4),
                Item::from_parts("Platemail", 102, 0, 5),
            ],
            rings: vec![
                Item::from_parts("Defense +1", 20, 0, 1),
                Item::from_parts("Damage +1", 25, 1, 0),
                Item::from_parts("Defense +2", 40, 0, 2),
                Item::from_parts("Damage +2", 50, 2, 0),
                Item::from_parts("Defense +3", 80, 0, 3),
                Item::from_parts("Damage +3", 100, 3, 0),
            ],
        }
    }
}

/// A `Character` must have one `weapon`, may have `armor`, and may have up to two unique `rings`
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Character {
    weapon: usize,
    armor: Option<usize>,
    rings: [Option<usize>; 2],
}

impl Character {
    /// Create the cheapest possible character to create
    pub fn cheapest() -> Self {
        Self {
            weapon: 0,
            armor: None,
            rings: [None, None],
        }
    }

    /// Creates the priciest possible character to create for a given `shop`
    pub fn priciest(shop: &Shop) -> Self {
        Self {
            weapon: shop.weapons.len() - 1,
            armor: Some(shop.armor.len() - 1),
            rings: [Some(shop.rings.len() - 2), Some(shop.rings.len() - 1)],
        }
    }

    /// Gets the sum-total price of gold needed to buy all of this character's items from the `shop`
    pub fn get_total_price(&self, shop: &Shop) -> i32 {
        shop.weapons[self.weapon].price
            + self.armor.map(|i| shop.armor[i].price).unwrap_or(0)
            + self
                .rings
                .iter()
                .map(|r| r.map(|i| shop.rings[i].price).unwrap_or(0))
                .sum::<i32>()
    }

    /// Unifies this character's gear into a single [`CoreStats`] block
    pub fn build_stats(&self, shop: &Shop) -> CoreStats {
        shop.weapons[self.weapon].stats
            + self
                .armor
                .map(|i| shop.armor[i].stats)
                .into_iter()
                .chain(
                    self.rings
                        .iter()
                        .filter_map(|r| r.map(|i| shop.rings[i].stats)),
                )
                .sum::<CoreStats>()
    }
}

/// State machine for generating all possible neighboring [`Character`]s from a starting
/// [`Character`] in a single direction (always incrementing, or always decrementing)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CharacterPermutation {
    Weapon,
    Armor,
    Rings0,
    Rings1,
    Done,
}

/// Generates all possible positive alterations to a [`Character`] by systematically incrementing
/// each piece of gear's [`Shop`] index independently
#[derive(Debug, Clone, PartialEq, Eq)]
struct CharacterForwardNeighbors<'shop> {
    start: Character,
    shop: &'shop Shop,
    state: CharacterPermutation,
}

impl<'shop> CharacterForwardNeighbors<'shop> {
    pub fn new(start: &Character, shop: &'shop Shop) -> Self {
        Self {
            start: start.clone(),
            shop,
            state: CharacterPermutation::Weapon,
        }
    }
}

impl<'shop> Iterator for CharacterForwardNeighbors<'shop> {
    type Item = (i32, Character);

    fn next(&mut self) -> Option<Self::Item> {
        let start_price = self.start.get_total_price(self.shop);

        loop {
            match self.state {
                CharacterPermutation::Weapon => {
                    self.state = CharacterPermutation::Armor;

                    let next_idx = self.start.weapon + 1;

                    if next_idx < self.shop.weapons.len() {
                        let mut next = self.start.clone();
                        next.weapon = next_idx;

                        break Some((next.get_total_price(self.shop) - start_price, next));
                    }
                }
                CharacterPermutation::Armor => {
                    self.state = CharacterPermutation::Rings0;

                    let next_idx = self.start.armor.clone().map(|i| i + 1).unwrap_or(0);

                    if next_idx < self.shop.armor.len() {
                        let mut next = self.start.clone();
                        next.armor = Some(next_idx);

                        break Some((next.get_total_price(self.shop) - start_price, next));
                    }
                }
                CharacterPermutation::Rings0 => {
                    self.state = CharacterPermutation::Rings1;

                    let mut next_idx = if let Some(r0) = self.start.rings[0] {
                        r0 + 1
                    } else {
                        0
                    };

                    if let Some(r1) = self.start.rings[1] {
                        if next_idx == r1 {
                            next_idx += 1;
                        }
                    }

                    if next_idx < self.shop.rings.len() {
                        let mut next = self.start.clone();
                        next.rings[0] = Some(next_idx);

                        break Some((next.get_total_price(self.shop) - start_price, next));
                    }
                }
                CharacterPermutation::Rings1 => {
                    self.state = CharacterPermutation::Done;

                    let mut next_idx = if let Some(r1) = self.start.rings[1] {
                        r1 + 1
                    } else {
                        0
                    };

                    if let Some(r0) = self.start.rings[0] {
                        if next_idx == r0 {
                            next_idx += 1;
                        }
                    }

                    if next_idx < self.shop.rings.len() {
                        let mut next = self.start.clone();
                        next.rings[1] = Some(next_idx);

                        break Some((next.get_total_price(self.shop) - start_price, next));
                    }
                }
                CharacterPermutation::Done => break None,
            }
        }
    }
}

/// Generates all possible negative alterations to a [`Character`] by systematically decrementing
/// each piece of gear's [`Shop`] index independently
#[derive(Debug, Clone, PartialEq, Eq)]
struct CharacterBackwardNeighbors<'shop> {
    start: Character,
    shop: &'shop Shop,
    state: CharacterPermutation,
}

impl<'shop> CharacterBackwardNeighbors<'shop> {
    pub fn new(start: &Character, shop: &'shop Shop) -> Self {
        Self {
            start: start.clone(),
            shop,
            state: CharacterPermutation::Weapon,
        }
    }
}

impl<'shop> Iterator for CharacterBackwardNeighbors<'shop> {
    type Item = (i32, Character);

    fn next(&mut self) -> Option<Self::Item> {
        let start_price = self.start.get_total_price(self.shop);

        loop {
            match self.state {
                CharacterPermutation::Weapon => {
                    self.state = CharacterPermutation::Armor;

                    if self.start.weapon > 0 {
                        let mut next = self.start.clone();
                        next.weapon -= 1;

                        break Some((start_price - next.get_total_price(self.shop), next));
                    }
                }
                CharacterPermutation::Armor => {
                    self.state = CharacterPermutation::Rings0;

                    if let Some(index) = self.start.armor {
                        let mut next = self.start.clone();

                        if index == 0 {
                            next.armor = None;
                        } else {
                            next.armor = Some(index - 1);
                        }

                        break Some((next.get_total_price(self.shop) - start_price, next));
                    }
                }
                CharacterPermutation::Rings0 => {
                    self.state = CharacterPermutation::Rings1;

                    if let Some(r0) = self.start.rings[0] {
                        let next_ring = if r0 == 0 {
                            None
                        } else {
                            let p = r0 - 1;

                            if Some(p) == self.start.rings[1] {
                                if p == 0 {
                                    None
                                } else {
                                    Some(p - 1)
                                }
                            } else {
                                Some(p)
                            }
                        };

                        let mut next = self.start.clone();
                        next.rings[0] = next_ring;

                        break Some((start_price - next.get_total_price(self.shop), next));
                    }
                }
                CharacterPermutation::Rings1 => {
                    self.state = CharacterPermutation::Done;

                    if let Some(r1) = self.start.rings[1] {
                        let next_ring = if r1 == 0 {
                            None
                        } else {
                            let p = r1 - 1;

                            if Some(p) == self.start.rings[0] {
                                if p == 0 {
                                    None
                                } else {
                                    Some(p - 1)
                                }
                            } else {
                                Some(p)
                            }
                        };

                        let mut next = self.start.clone();
                        next.rings[1] = next_ring;

                        break Some((start_price - next.get_total_price(self.shop), next));
                    }
                }
                CharacterPermutation::Done => break None,
            }
        }
    }
}

/// A creature's stats are just its `health` and underlying [`CoreStats`] (typically assembled from
/// equipped gear)
#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct CreatureStats {
    pub health: i32,
    pub stats: CoreStats,
}

impl CreatureStats {
    pub fn from_parts(health: i32, damage: i32, armor: i32) -> Self {
        Self {
            health,
            stats: CoreStats { damage, armor },
        }
    }

    pub fn from_character(health: i32, character: &Character, shop: &Shop) -> Self {
        Self {
            health,
            stats: character.build_stats(shop),
        }
    }

    /// Checks if this creature can defeat the `other` creature (drop its health to 0 first),
    /// assuming we go first
    pub fn can_defeat(&self, other: &Self) -> bool {
        let our_damage = 1.max(self.stats.damage - other.stats.armor);
        let other_damage = 1.max(other.stats.damage - self.stats.armor);

        let our_turns =
            other.health.div_euclid(our_damage) + 1.min(other.health.rem_euclid(our_damage));
        let other_turns =
            self.health.div_euclid(other_damage) + 1.min(self.health.rem_euclid(other_damage));

        return our_turns <= other_turns;
    }
}

fn cheapest_to_beat(health: i32, shop: &Shop, boss: &CreatureStats) -> Character {
    a_star(
        Character::cheapest(),
        |c| Some(CharacterForwardNeighbors::new(c, shop)),
        |_| Default::default(),
        |c| CreatureStats::from_character(health, c, shop).can_defeat(boss),
    )
    .unwrap()
    .next()
    .unwrap()
    .node
}

fn priciest_to_lose(health: i32, shop: &Shop, boss: &CreatureStats) -> Character {
    a_star(
        Character::priciest(shop),
        |c| Some(CharacterBackwardNeighbors::new(c, shop)),
        |_| Default::default(),
        |c| !CreatureStats::from_character(health, c, shop).can_defeat(boss),
    )
    .unwrap()
    .next()
    .unwrap()
    .node
}

pub fn part_01() {
    let shop = Shop::new();
    let boss = CreatureStats::from_parts(109, 8, 2);
    let cheapest = cheapest_to_beat(100, &shop, &boss);

    println!("Min gold to beat boss: {}", cheapest.get_total_price(&shop));
}

pub fn part_02() {
    let shop = Shop::new();
    let boss = CreatureStats::from_parts(109, 8, 2);
    let priciest = priciest_to_lose(100, &shop, &boss);

    println!(
        "Max gold to lose to boss: {}",
        priciest.get_total_price(&shop)
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn close_win() {
        let player = CreatureStats::from_parts(8, 5, 5);
        let boss = CreatureStats::from_parts(12, 7, 2);

        assert!(player.can_defeat(&boss));
        assert!(boss.can_defeat(&player));
    }
}
