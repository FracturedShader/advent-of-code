use std::io::BufRead;

struct Point(usize, usize);

impl Point {
    fn from_part(part: &str) -> Point {
        let coords: Vec<_> = part.split(',').map(|p| p.parse().unwrap()).collect();

        Point(coords[0], coords[1])
    }
}

struct RectIter {
    x_min: usize,
    y_min: usize,
    x_range: usize,
    y_range: usize,
    x: usize,
    y: usize,
}

impl RectIter {
    fn new(upper_left: &Point, lower_right: &Point) -> Self {
        RectIter {
            x_min: upper_left.0,
            y_min: upper_left.1,
            x_range: lower_right.0 - upper_left.0,
            y_range: lower_right.1 - upper_left.1,
            x: 0,
            y: 0,
        }
    }
}

impl Iterator for RectIter {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y > self.y_range {
            return None;
        }

        let pt = Point(self.x + self.x_min, self.y + self.y_min);

        self.x += 1;

        if self.x > self.x_range {
            self.x = 0;
            self.y += 1;
        }

        Some(pt)
    }
}

struct Rect {
    upper_left: Point,
    lower_right: Point,
}

impl Rect {
    fn from_parts(parts: &[&str]) -> Self {
        let upper_left = Point::from_part(parts[0]);
        let lower_right = Point::from_part(parts[2]);

        Rect {
            upper_left,
            lower_right,
        }
    }

    fn iter(&self) -> RectIter {
        RectIter::new(&self.upper_left, &self.lower_right)
    }
}

fn array_idx(point: &Point) -> usize {
    point.1 * 1000 + point.0
}

trait LightChanger {
    fn turn_on(&mut self, area: &Rect);

    fn turn_off(&mut self, area: &Rect);

    fn toggle(&mut self, area: &Rect);

    fn count_on(&self) -> usize;
}

struct SimpleLights {
    lights: Vec<usize>,
}

impl Default for SimpleLights {
    fn default() -> Self {
        SimpleLights {
            lights: vec![0; 1000 * 1000],
        }
    }
}

impl LightChanger for SimpleLights {
    fn turn_on(&mut self, area: &Rect) {
        for ref pt in area.iter() {
            self.lights[array_idx(pt)] = 1;
        }
    }

    fn turn_off(&mut self, area: &Rect) {
        for ref pt in area.iter() {
            self.lights[array_idx(pt)] = 0;
        }
    }

    fn toggle(&mut self, area: &Rect) {
        for ref pt in area.iter() {
            let idx = array_idx(pt);
            self.lights[idx] = (self.lights[idx] + 1) & 0x01;
        }
    }

    fn count_on(&self) -> usize {
        self.lights.iter().sum::<usize>()
    }
}

struct VariableLights {
    lights: Vec<usize>,
}

impl Default for VariableLights {
    fn default() -> Self {
        VariableLights {
            lights: vec![0; 1000 * 1000],
        }
    }
}

impl LightChanger for VariableLights {
    fn turn_on(&mut self, area: &Rect) {
        for ref pt in area.iter() {
            self.lights[array_idx(pt)] += 1;
        }
    }

    fn turn_off(&mut self, area: &Rect) {
        for ref pt in area.iter() {
            let idx = array_idx(pt);

            if self.lights[idx] != 0 {
                self.lights[idx] -= 1;
            }
        }
    }

    fn toggle(&mut self, area: &Rect) {
        for ref pt in area.iter() {
            self.lights[array_idx(pt)] += 2;
        }
    }

    fn count_on(&self) -> usize {
        self.lights.iter().sum::<usize>()
    }
}

enum RequestedAction {
    TurnOn,
    TurnOff,
    Toggle,
}

impl RequestedAction {
    fn from_parts<'a, I>(parts: &mut I) -> Self
    where
        I: Iterator<Item = &'a str>,
    {
        match parts.next().unwrap() {
            "turn" => match parts.next().unwrap() {
                "on" => RequestedAction::TurnOn,
                "off" => RequestedAction::TurnOff,
                _ => unreachable!("Turn can only be 'on' or 'off'."),
            },
            "toggle" => RequestedAction::Toggle,
            _ => unreachable!("First word must be 'turn' or 'toggle'."),
        }
    }
}

fn process_instructions(reader: impl BufRead, lights: &mut impl LightChanger) {
    for line in reader.lines().filter_map(|l| l.ok()) {
        let mut parts = line.split_ascii_whitespace();
        let action = RequestedAction::from_parts(&mut parts);
        let area = Rect::from_parts(&parts.collect::<Vec<_>>());

        match action {
            RequestedAction::TurnOn => lights.turn_on(&area),
            RequestedAction::TurnOff => lights.turn_off(&area),
            RequestedAction::Toggle => lights.toggle(&area),
        }
    }
}

pub fn part_01(reader: Option<impl BufRead>) {
    let mut simple_lights: SimpleLights = Default::default();

    process_instructions(reader.unwrap(), &mut simple_lights);

    println!("Lights on: {}", simple_lights.count_on());
}

pub fn part_02(reader: Option<impl BufRead>) {
    let mut var_lights: VariableLights = Default::default();

    process_instructions(reader.unwrap(), &mut var_lights);

    println!("Lights on: {}", var_lights.count_on());
}
