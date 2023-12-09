use std::{
    collections::{HashMap, VecDeque},
    io::BufRead,
};

#[derive(Debug, Default)]
struct Ingredient {
    _name: String,
    capacity: i64,
    durability: i64,
    flavor: i64,
    texture: i64,
    calories: i64,
}

impl Ingredient {
    fn parse<S>(data: S) -> Self
    where
        S: AsRef<str>,
    {
        let mut halves = data.as_ref().split(':');
        let mut c = Ingredient {
            _name: halves.next().unwrap().to_owned(),
            ..Default::default()
        };

        for attr in halves.next().unwrap().split(',') {
            let mut parts = attr.split_ascii_whitespace();
            let field = parts.next().unwrap();
            let val = parts.next().unwrap().parse().unwrap();

            match field {
                "capacity" => c.capacity = val,
                "durability" => c.durability = val,
                "flavor" => c.flavor = val,
                "texture" => c.texture = val,
                "calories" => c.calories = val,
                _ => unreachable!(),
            }
        }

        c
    }
}

fn add_valid_cases(base: &[i64], n: usize, q: &mut VecDeque<Vec<i64>>) {
    for i in 0..n {
        for j in 0..n {
            if i == j || base[j] == 0 {
                continue;
            }

            let mut nc = base.to_owned();

            nc[i] += 1;
            nc[j] -= 1;

            q.push_back(nc.clone());
        }
    }
}

fn eval_candidate(c: &[i64], ingredients: &[Ingredient]) -> i64 {
    c.iter()
        .zip(ingredients)
        .map(|(c, i)| {
            [
                c * i.capacity,
                c * i.durability,
                c * i.flavor,
                c * i.texture,
            ]
        })
        .reduce(|a, e| [a[0] + e[0], a[1] + e[1], a[2] + e[2], a[3] + e[3]])
        .unwrap()
        .into_iter()
        .reduce(i64::saturating_mul)
        .unwrap()
}

// Assumes the multi-dimensional evaluation space forms a convex hull and performs gradient-ascent
fn highest_score(teaspoons: i64, ingredients: &[Ingredient]) -> (Vec<i64>, i64) {
    let len = ingredients.len();
    let num_ingredients = i64::try_from(len).expect("number of ingredients should fit in an i64");
    let initial_guess = teaspoons / num_ingredients;
    let mut guesses = vec![initial_guess; len];

    guesses[0] = teaspoons - ((num_ingredients - 1) * initial_guess);

    let mut candidates = VecDeque::with_capacity(64);

    candidates.push_back(guesses);

    let mut best_total = 0;
    let mut best_candidate = vec![];
    let mut tested: HashMap<Vec<i64>, i64> = HashMap::default();

    while let Some(c) = candidates.pop_front() {
        if tested.contains_key(&c) {
            continue;
        }

        let total = eval_candidate(&c, ingredients);

        if total > best_total {
            best_total = total;
            best_candidate = c.clone();

            add_valid_cases(&c, len, &mut candidates);
        }

        tested.insert(c, total);
    }

    (best_candidate, best_total)
}

fn highest_500cal_score(teaspoons: i64, ingredients: &[Ingredient]) -> (Vec<i64>, i64) {
    let (initial, _) = highest_score(teaspoons, ingredients);

    let num_ingredients = ingredients.len();
    let mut candidates = VecDeque::with_capacity(64);

    candidates.push_back(initial);

    let mut best_total = 0;
    let mut best_candidate = vec![];
    let mut tested: HashMap<Vec<i64>, i64> = HashMap::default();

    while let Some(c) = candidates.pop_front() {
        if tested.contains_key(&c) {
            continue;
        }

        let total = eval_candidate(&c, ingredients);

        if total > best_total {
            let calories = c
                .iter()
                .zip(ingredients)
                .map(|(n, c)| n * c.calories)
                .reduce(i64::saturating_add)
                .unwrap();

            if calories == 500 {
                best_total = total;
                best_candidate = c.clone();
            }

            add_valid_cases(&c, num_ingredients, &mut candidates);
        }

        tested.insert(c, total);
    }

    (best_candidate, best_total)
}

pub fn part_01(reader: Option<impl BufRead>) {
    let ingredients = reader
        .unwrap()
        .lines()
        .map_while(Result::ok)
        .map(Ingredient::parse)
        .collect::<Vec<_>>();

    println!("{}", highest_score(100, &ingredients).1);
}

pub fn part_02(reader: Option<impl BufRead>) {
    let ingredients = reader
        .unwrap()
        .lines()
        .map_while(Result::ok)
        .map(Ingredient::parse)
        .collect::<Vec<_>>();

    println!("{}", highest_500cal_score(100, &ingredients).1);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn highest_total_score() {
        let ingredients_data = r"Butterscotch: capacity -1, durability -2, flavor 6, texture 3, calories 8
Cinnamon: capacity 2, durability 3, flavor -2, texture -1, calories 3";

        let ingredients = ingredients_data
            .lines()
            .map(Ingredient::parse)
            .collect::<Vec<_>>();

        assert_eq!(highest_score(100, &ingredients).1, 62_842_880);
    }

    #[test]
    fn highest_cal_score() {
        let ingredients_data = r"Butterscotch: capacity -1, durability -2, flavor 6, texture 3, calories 8
Cinnamon: capacity 2, durability 3, flavor -2, texture -1, calories 3";

        let ingredients = ingredients_data
            .lines()
            .map(Ingredient::parse)
            .collect::<Vec<_>>();

        assert_eq!(highest_500cal_score(100, &ingredients).1, 57_600_000);
    }
}
