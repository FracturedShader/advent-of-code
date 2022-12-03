use std::io::BufRead;

struct Reindeer {
    _name: String,
    speed: u32,
    fly_time: u32,
    rest_time: u32,
}

impl Reindeer {
    fn from_line(l: &str) -> Self {
        let mut parts = l.split_ascii_whitespace();

        let _name = parts.next().unwrap().to_owned();
        let speed = parts.nth(2).unwrap().parse().unwrap();
        let fly_time = parts.nth(2).unwrap().parse().unwrap();
        let rest_time = parts.nth(6).unwrap().parse().unwrap();

        Self {
            _name,
            speed,
            fly_time,
            rest_time,
        }
    }

    fn traveled(&self, time: u32) -> u32 {
        let cycle_time = self.fly_time + self.rest_time;
        let full_cycles = time / cycle_time;
        let partial_time = time % cycle_time;

        (full_cycles * self.fly_time + partial_time.min(self.fly_time)) * self.speed
    }
}

fn per_second_scoring(reindeer: &[Reindeer]) -> u32 {
    let mut points = vec![0; reindeer.len()];

    // Could I do this by finding the next speed change and treating it as a combination
    // of linear equations and check who would surpass the winner when and how? Sure. I COULD
    for t in 1..=2503 {
        let mut max = 0;
        let mut indices = Vec::with_capacity(reindeer.len());

        for (i, d) in reindeer.iter().map(|r| r.traveled(t)).enumerate() {
            if d >= max {
                if d > max {
                    indices.clear();
                    max = d;
                }

                indices.push(i);
            }
        }

        for i in indices {
            points[i] += 1;
        }
    }

    points.into_iter().max().unwrap()
}

pub fn part_01(reader: Option<impl BufRead>) {
    let max_dist = reader
        .unwrap()
        .lines()
        .filter_map(|l| l.ok())
        .map(|ref l| Reindeer::from_line(l))
        .map(|r| r.traveled(2503))
        .max()
        .unwrap();

    println!("Winning distance: {}", max_dist);
}

pub fn part_02(reader: Option<impl BufRead>) {
    let reindeer = reader
        .unwrap()
        .lines()
        .filter_map(|l| l.ok())
        .map(|ref l| Reindeer::from_line(l))
        .collect::<Vec<_>>();

    let max_points = per_second_scoring(&reindeer);

    println!("Winning points: {}", max_points);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reindeer() {
        let r1 = Reindeer::from_line(
            "Comet can fly 14 km/s for 10 seconds, but then must rest for 127 seconds.",
        );

        let r2 = Reindeer::from_line(
            "Dancer can fly 16 km/s for 11 seconds, but then must rest for 162 seconds.",
        );

        assert_eq!(r1.traveled(1000), 1120);
        assert_eq!(r2.traveled(1000), 1056);
    }
}
