use itertools::Itertools;
use std::{collections::HashMap, io::BufRead};

#[derive(Debug, Default)]
struct LocationGraph {
    names: Vec<String>,
    distances: HashMap<(usize, usize), usize>,
}

impl LocationGraph {
    fn from_lines<S, I>(lines: I) -> LocationGraph
    where
        S: AsRef<str>,
        I: Iterator<Item = S>,
    {
        let mut name_indices = HashMap::new();
        let mut names = Vec::new();
        let mut distances = HashMap::new();

        for l in lines {
            let l = l.as_ref();
            let mut main_parts = l.split(" = ");
            let from_to = main_parts.next().unwrap().split(" to ");
            let dist = main_parts.next().unwrap().parse::<usize>().unwrap();

            for loc in from_to.clone() {
                if !name_indices.contains_key(loc) {
                    name_indices.insert(loc.to_string(), names.len());
                    names.push(loc.to_string());
                }
            }

            let from_to_indices = from_to.map(|loc| name_indices[loc]).collect::<Vec<_>>();

            distances.insert((from_to_indices[0], from_to_indices[1]), dist);
            distances.insert((from_to_indices[1], from_to_indices[0]), dist);
        }

        LocationGraph { names, distances }
    }

    fn traveling_salesman<F>(&self, initial: usize, cmp: F) -> (Vec<String>, usize)
    where
        F: Fn(usize, usize) -> bool,
    {
        let num_entries = self.names.len();
        let mut route = Vec::new();
        let mut chosen_dist = initial;

        for p in (0..num_entries).permutations(num_entries) {
            let dist: usize = p.windows(2).map(|w| self.distances[&(w[0], w[1])]).sum();

            if cmp(dist, chosen_dist) {
                route = p;
                chosen_dist = dist;
            }
        }

        (
            route.into_iter().map(|i| self.names[i].clone()).collect(),
            chosen_dist,
        )
    }

    fn bad_traveling_salesman(&self) -> (Vec<String>, usize) {
        self.traveling_salesman(0, |curr, sel| curr > sel)
    }

    fn good_traveling_salesman(&self) -> (Vec<String>, usize) {
        self.traveling_salesman(usize::MAX, |curr, sel| curr < sel)
    }
}

pub fn part_01(reader: Option<impl BufRead>) {
    let graph = LocationGraph::from_lines(reader.unwrap().lines().map_while(Result::ok));

    println!("{:?}", graph.good_traveling_salesman());
}

pub fn part_02(reader: Option<impl BufRead>) {
    let graph = LocationGraph::from_lines(reader.unwrap().lines().map_while(Result::ok));

    println!("{:?}", graph.bad_traveling_salesman());
}

#[cfg(test)]
mod test {
    use super::LocationGraph;

    #[test]
    fn shortest_distance() {
        let data = r"London to Dublin = 464
London to Belfast = 518
Dublin to Belfast = 141";

        let graph = LocationGraph::from_lines(data.lines());
        let (_, d) = graph.good_traveling_salesman();

        assert_eq!(605, d);
    }

    #[test]
    fn longest_distance() {
        let data = r"London to Dublin = 464
London to Belfast = 518
Dublin to Belfast = 141";

        let graph = LocationGraph::from_lines(data.lines());
        let (_, d) = graph.bad_traveling_salesman();

        assert_eq!(982, d);
    }
}
