use std::{collections::HashMap, io::BufRead};

struct Aunt(HashMap<String, usize>);

impl Aunt {
    fn parse<S>(data: S) -> Self
    where
        S: AsRef<str>,
    {
        let info = data
            .as_ref()
            .split_once(": ")
            .unwrap()
            .1
            .split(", ")
            .map(|kvp| {
                let (k, v) = kvp.split_once(": ").unwrap();

                (k.to_owned(), v.parse::<usize>().unwrap())
            })
            .collect();

        Aunt(info)
    }
}

fn mfcsam_basic() -> HashMap<String, usize> {
    r#"children: 3
cats: 7
samoyeds: 2
pomeranians: 3
akitas: 0
vizslas: 0
goldfish: 5
trees: 3
cars: 2
perfumes: 1"#
        .lines()
        .map(|kvp| {
            let (k, v) = kvp.split_once(": ").unwrap();

            (k.to_owned(), v.parse::<usize>().unwrap())
        })
        .collect()
}

enum Reading {
    Exactly(usize),
    GreaterThan(usize),
    LessThan(usize),
}

impl Reading {
    fn matches(&self, val: usize) -> bool {
        match *self {
            Reading::Exactly(v) => val == v,
            Reading::GreaterThan(v) => val > v,
            Reading::LessThan(v) => val < v,
        }
    }
}

fn mfcsam_full() -> HashMap<String, Reading> {
    let basic = mfcsam_basic();

    basic
        .into_iter()
        .map(|(k, v)| match k.as_str() {
            "cats" | "trees" => (k, Reading::GreaterThan(v)),
            "pomeranians" | "goldfish" => (k, Reading::LessThan(v)),
            _ => (k, Reading::Exactly(v)),
        })
        .collect()
}

pub fn part_01(reader: Option<impl BufRead>) {
    let aunts = reader
        .unwrap()
        .lines()
        .filter_map(|l| l.ok().map(Aunt::parse))
        .collect::<Vec<_>>();

    let known_info = mfcsam_basic();

    for (idx, aunt) in aunts.iter().enumerate() {
        if aunt
            .0
            .iter()
            .all(|(key, val)| known_info.get(key).unwrap() == val)
        {
            println!("{}", idx + 1);
            break;
        }
    }
}

pub fn part_02(reader: Option<impl BufRead>) {
    let aunts = reader
        .unwrap()
        .lines()
        .filter_map(|l| l.ok().map(Aunt::parse))
        .collect::<Vec<_>>();

    let known_info = mfcsam_full();

    for (idx, aunt) in aunts.iter().enumerate() {
        if aunt
            .0
            .iter()
            .all(|(key, &val)| known_info.get(key).unwrap().matches(val))
        {
            println!("{}", idx + 1);
            break;
        }
    }
}
