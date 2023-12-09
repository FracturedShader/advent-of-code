use std::{collections::HashMap, io::BufRead};

use itertools::Itertools;

fn parse_happiness_map<S, I>(lines: I) -> HashMap<String, HashMap<String, i64>>
where
    S: AsRef<str>,
    I: Iterator<Item = S>,
{
    let mut m = HashMap::<String, HashMap<String, i64>>::default();

    for l in lines {
        let mut parts = l.as_ref().split_ascii_whitespace();
        let name = parts.next().unwrap();
        let lg = parts.nth(1).unwrap();
        let amount = parts.next().unwrap().parse::<i64>().unwrap();
        let other = parts.last().unwrap().trim_matches('.');

        let amount = if lg == "gain" { amount } else { -amount };

        m.entry(name.to_owned())
            .or_default()
            .insert(other.to_owned(), amount);
    }

    m
}

fn best_seating(hap_map: &HashMap<String, HashMap<String, i64>>) -> i64 {
    hap_map
        .keys()
        .permutations(hap_map.len())
        .map(|perm| {
            perm.iter()
                .zip(perm.iter().cycle().skip(1))
                .map(|(&p, &n)| {
                    hap_map.get(p).unwrap().get(n).unwrap()
                        + hap_map.get(n).unwrap().get(p).unwrap()
                })
                .sum()
        })
        .max()
        .unwrap()
}

pub fn part_01(reader: Option<impl BufRead>) {
    let hap_map = parse_happiness_map(reader.unwrap().lines().map_while(Result::ok));
    let greatest_change = best_seating(&hap_map);

    println!("Total change in happiness: {greatest_change}");
}

pub fn part_02(reader: Option<impl BufRead>) {
    let mut hap_map = parse_happiness_map(reader.unwrap().lines().map_while(Result::ok));
    let me = "Me".to_owned();

    for k in hap_map.keys().map(String::to_owned).collect_vec() {
        hap_map.get_mut(&k).unwrap().insert(me.clone(), 0);

        hap_map.entry(me.clone()).or_default().insert(k, 0);
    }

    let greatest_change = best_seating(&hap_map);

    println!("Total change in happiness: {greatest_change}");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn happiness() {
        let data = r"Alice would gain 54 happiness units by sitting next to Bob.
Alice would lose 79 happiness units by sitting next to Carol.
Alice would lose 2 happiness units by sitting next to David.
Bob would gain 83 happiness units by sitting next to Alice.
Bob would lose 7 happiness units by sitting next to Carol.
Bob would lose 63 happiness units by sitting next to David.
Carol would lose 62 happiness units by sitting next to Alice.
Carol would gain 60 happiness units by sitting next to Bob.
Carol would gain 55 happiness units by sitting next to David.
David would gain 46 happiness units by sitting next to Alice.
David would lose 7 happiness units by sitting next to Bob.
David would gain 41 happiness units by sitting next to Carol.";

        let hap_map = parse_happiness_map(data.lines());

        assert_eq!(best_seating(&hap_map), 330);
    }
}
