use std::{
    collections::{HashSet, VecDeque},
    io::BufRead,
};

fn container_combinations(to_store: usize, containers: &[usize]) -> (usize, usize) {
    let keys = containers
        .iter()
        .enumerate()
        .map(|(i, &v)| (1 << i, v))
        .collect::<Vec<_>>();

    let mut seen: HashSet<usize> = Default::default();
    let mut combos = 0;

    let mut min_containers = containers.len();
    let mut min_combos = 0;

    let mut candidates: VecDeque<_> = Default::default();

    candidates.push_back((0, to_store));

    while let Some((c, r)) = candidates.pop_front() {
        for &(k, v) in &keys {
            let u = c | k;

            if c & k != 0 || v > r || seen.contains(&u) {
                continue;
            }

            seen.insert(u);

            if r == v {
                combos += 1;

                let cc = u.count_ones() as usize;

                match cc.cmp(&min_containers) {
                    std::cmp::Ordering::Less => {
                        min_containers = cc;
                        min_combos = 1;
                    }
                    std::cmp::Ordering::Equal => min_combos += 1,
                    std::cmp::Ordering::Greater => {}
                }
            } else {
                candidates.push_back((u, r - v));
            }
        }
    }

    (combos, min_combos)
}

pub fn part_01(reader: Option<impl BufRead>) {
    let containers = reader
        .unwrap()
        .lines()
        .filter_map(|l| l.ok())
        .filter_map(|s| s.parse::<usize>().ok())
        .collect::<Vec<_>>();

    let combos = container_combinations(150, &containers).0;

    println!("{}", combos);
}

pub fn part_02(reader: Option<impl BufRead>) {
    let containers = reader
        .unwrap()
        .lines()
        .filter_map(|l| l.ok())
        .filter_map(|s| s.parse::<usize>().ok())
        .collect::<Vec<_>>();

    let min_combos = container_combinations(150, &containers).1;

    println!("{}", min_combos);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn case_count() {
        let containers = "20, 15, 10, 5, 5"
            .split(", ")
            .filter_map(|l| l.parse::<usize>().ok())
            .collect::<Vec<_>>();

        assert_eq!(container_combinations(25, &containers), (4, 3));
    }
}
