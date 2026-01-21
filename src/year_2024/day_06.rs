use nalgebra::Vector2;

use crate::common::{self, Grid2};

/// Explicit representation of the four cardinal directions in 2D.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Facing {
    Up,
    Right,
    Down,
    Left,
}

impl Facing {
    /// Takes one step in the current `Facing` direction from `p`.
    /// Returns `Some(Vector2<i32>)` when not over/underflowing `u32`.
    fn step(self, p: Vector2<i32>) -> Option<Vector2<i32>> {
        match self {
            Facing::Up => p.y.checked_sub(1).map(|y| Vector2::new(p.x, y)),
            Facing::Right => p.x.checked_add(1).map(|x| Vector2::new(x, p.y)),
            Facing::Down => p.y.checked_add(1).map(|y| Vector2::new(p.x, y)),
            Facing::Left => p.x.checked_sub(1).map(|x| Vector2::new(x, p.y)),
        }
    }

    fn turn_right(&mut self) {
        *self = match *self {
            Facing::Up => Facing::Right,
            Facing::Right => Facing::Down,
            Facing::Down => Facing::Left,
            Facing::Left => Facing::Up,
        };
    }
}

/// Converts each `Facing` value to a flag for easy composition
impl From<Facing> for u8 {
    fn from(value: Facing) -> Self {
        match value {
            Facing::Up => 0b0001,
            Facing::Right => 0b0010,
            Facing::Down => 0b0100,
            Facing::Left => 0b1000,
        }
    }
}

/// Represents all possible states that a cell in a walked lab map can have. `MapCell::Visited(u8)`
/// is a union of all `Facing` directions that the cell was visited with.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum MapCell {
    Empty,
    Obstruction,
    Visited(u8),
}

/// Provides direct conversion from puzzle input to `MapCell`
impl From<u8> for MapCell {
    fn from(value: u8) -> Self {
        match value {
            b'#' => Self::Obstruction,
            b'^' => Self::Visited(Facing::Up.into()),
            _ => Self::Empty,
        }
    }
}

impl MapCell {
    fn is_empty(self) -> bool {
        matches!(self, Self::Empty)
    }

    fn is_obstruction(self) -> bool {
        matches!(self, Self::Obstruction)
    }

    fn is_visited_facing(self, facing: Facing) -> bool {
        match self {
            Self::Visited(v) => v & u8::from(facing) != 0,
            _ => false,
        }
    }

    /// Panics if `self == MapCell::Obstruction` since visiting an obstructed cell is not possible.
    fn visit(&mut self, facing: Facing) {
        *self = match *self {
            Self::Empty => Self::Visited(u8::from(facing)),
            Self::Visited(v) => Self::Visited(v | u8::from(facing)),
            Self::Obstruction => panic!("Cannot visit an obstructed cell!"),
        };
    }
}

/// Representation of a guard walking the lab. Has a `position` in 2D space and a `facing`
/// direction.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Guard {
    position: Vector2<i32>,
    facing: Facing,
}

/// Attempts to create a Guard from the representation ('^') in the puzzle input.
impl TryFrom<&Grid2<u8>> for Guard {
    type Error = &'static str;

    fn try_from(value: &Grid2<u8>) -> Result<Self, Self::Error> {
        let position = value.position(&b'^').ok_or("No guard found in the lab.")?;

        Ok(Self {
            position,
            facing: Facing::Up,
        })
    }
}

/// All the different possible outcomes for when a `Guard` tries to take a step on its patrol. May
/// get `StepResult::Stuck` if completely boxed-in.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum StepResult {
    Stepped,
    Stuck,
    LeftLab,
}

impl StepResult {
    fn is_stepped(self) -> bool {
        matches!(self, Self::Stepped)
    }
}

impl Guard {
    /// Helper method for patrolling different representations of the lab. The only difference is
    /// what counts as an obstruction.
    fn step_pred<P, T>(&mut self, lab_map: &Grid2<T>, is_obstruction: P) -> StepResult
    where
        P: FnMut(&T) -> bool,
    {
        if self.turn_to_face_empty_pred(lab_map, is_obstruction) {
            if let Some(pos) = self.facing.step(self.position) {
                if lab_map.in_bounds(pos) {
                    self.position = pos;

                    return StepResult::Stepped;
                }
            }

            return StepResult::LeftLab;
        }

        StepResult::Stuck
    }

    /// Same `step` process as normal, but updates `vis_map` with every position/orientation of the
    /// guard during the step. The `vis_map` will only be updated with where this `Guard` has been,
    /// not where it ends up at the end of the step.
    fn step_tracked(&mut self, vis_map: &mut Grid2<MapCell>) -> StepResult {
        let initial_facing = self.facing;

        vis_map.get_mut(self.position).visit(self.facing);

        while let Some(pos) = self.facing.step(self.position) {
            // Check for leaving the lab
            if !vis_map.in_bounds(pos) {
                return StepResult::LeftLab;
            }

            if vis_map.get(pos).is_obstruction() {
                self.facing.turn_right();
                vis_map.get_mut(self.position).visit(self.facing);

                if self.facing == initial_facing {
                    return StepResult::Stuck;
                }

                continue;
            }

            self.position = pos;

            return StepResult::Stepped;
        }

        // Would leave the lab
        StepResult::LeftLab
    }

    /// Try to turn this `Guard` to face an empty cell. Can fail if completely surrounded by
    /// obstructions.
    fn turn_to_face_empty_pred<P, T>(&mut self, lab_map: &Grid2<T>, mut is_obstruction: P) -> bool
    where
        P: FnMut(&T) -> bool,
    {
        let initial_facing = self.facing;

        while let Some(pos) = self.facing.step(self.position) {
            // Check for leaving the lab
            if !lab_map.in_bounds(pos) {
                return true;
            }

            if !is_obstruction(lab_map.get(pos)) {
                return true;
            }

            self.facing.turn_right();

            // Blocked in on all sides
            if self.facing == initial_facing {
                return false;
            }
        }

        // Would leave the lab
        true
    }
}

/// Helper trait for stepping different kinds of grids.
trait NavigateGrid<T> {
    fn step(&mut self, lab_map: &Grid2<T>) -> StepResult;
}

impl NavigateGrid<u8> for Guard {
    fn step(&mut self, lab_map: &Grid2<u8>) -> StepResult {
        self.step_pred(lab_map, |&b| b == b'#')
    }
}

impl NavigateGrid<MapCell> for Guard {
    fn step(&mut self, lab_map: &Grid2<MapCell>) -> StepResult {
        self.step_pred(lab_map, |c| c.is_obstruction())
    }
}

/// Counts the number of cells patrolled by a guard when following the rules of the puzzle.
fn num_cells_patrolled(lab_map: &Grid2<u8>, guard: Guard) -> usize {
    let mut grid = lab_map.clone();
    let mut guard = guard;

    loop {
        grid.set(guard.position, b'X');

        if !guard.step(lab_map).is_stepped() {
            break;
        }
    }

    bytecount::count(grid.inner(), b'X')
}

/// Answers the question of "would the `guard` get stuck in a loop if we blocked the next space
/// it's supposed to visit?"
/// Returns `true` if the guard would step into an unvisited map cell and blocking that cell would
/// cause the guard to patrol a loop.
fn would_loop_if_blocked(vis_map: &Grid2<MapCell>, guard: &Guard) -> bool {
    let mut step_ahead = *guard;

    // Next step would leave the map
    if !step_ahead.step(vis_map).is_stepped() {
        return false;
    }

    // We'd be blocking somewhere in the past, which would prevent getting here in the first place
    if !vis_map.get(step_ahead.position).is_empty() {
        return false;
    }

    let mut predicted_map = vis_map.clone();

    *predicted_map.get_mut(step_ahead.position) = MapCell::Obstruction;

    let mut future_guard = *guard;

    loop {
        match future_guard.step_tracked(&mut predicted_map) {
            StepResult::Stepped => {}
            StepResult::Stuck => {
                return true;
            }
            StepResult::LeftLab => {
                return false;
            }
        }

        if predicted_map
            .get(future_guard.position)
            .is_visited_facing(future_guard.facing)
        {
            return true;
        }
    }
}

/// Given a lab map and a `Guard`, answers how many different obstructions can be created to cause
/// the guard to patrol in a loop.
fn num_obstruction_loop_options(lab_map: &Grid2<u8>, mut guard: Guard) -> usize {
    let mut vis_map = Grid2::from_vec(
        lab_map.iter().copied().map(MapCell::from).collect(),
        lab_map.width().try_into().unwrap(),
    )
    .unwrap();

    let mut loops_count = 0;

    loop {
        if would_loop_if_blocked(&vis_map, &guard) {
            loops_count += 1;
        }

        if !guard.step_tracked(&mut vis_map).is_stepped() {
            break;
        }
    }

    loops_count
}

pub fn part_01() {
    let lab_map = Grid2::try_from_reader(common::puzzle_input("2024-06").unwrap())
        .expect("problem input should be a valid map");
    let guard = Guard::try_from(&lab_map).expect("input map should have a guard");

    let cells_visited = num_cells_patrolled(&lab_map, guard);

    println!("Cells visited: {cells_visited}");
}

pub fn part_02() {
    let lab_map = Grid2::try_from_reader(common::puzzle_input("2024-06").unwrap())
        .expect("problem input should be a valid map");
    let guard = Guard::try_from(&lab_map).expect("input map should have a guard");

    let loop_options = num_obstruction_loop_options(&lab_map, guard);

    println!("Options for loops: {loop_options}");
}

#[cfg(test)]
mod test {
    use super::*;

    fn parsed_test_input() -> Grid2<u8> {
        let input = r"....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
#.........
......#...";

        Grid2::try_from_reader(input.as_bytes()).unwrap()
    }

    #[test]
    fn patrol() {
        let lab_map = parsed_test_input();
        let guard = Guard::try_from(&lab_map).unwrap();

        let cells_visited = num_cells_patrolled(&lab_map, guard);

        assert_eq!(cells_visited, 41);
    }

    #[test]
    fn cause_loop() {
        let lab_map = parsed_test_input();
        let guard = Guard::try_from(&lab_map).unwrap();

        let obstruction_options = num_obstruction_loop_options(&lab_map, guard);

        assert_eq!(obstruction_options, 6);
    }
}
