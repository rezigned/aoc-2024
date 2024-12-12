use reactive_stores::Store;
use std::{collections::HashSet, fmt::Display};

/// Represents the game states.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum State {
    #[default]
    Ready,
    Running,
    Paused,
    Done,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum Part {
    #[default]
    One,
    Two,
}

/// Represents the direction of the guard.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Up => "up",
            Direction::Right => "right",
            Direction::Down => "down",
            Direction::Left => "left",
        }
    }
}

#[derive(Clone, Store)]
pub(crate) struct Game {
    pub grid: Grid,
    width: usize,
    height: usize,
    state: State,
    mode: Part,
    guard: Position,
    direction: Direction,
    visited: HashSet<Position>,
    loops: HashSet<(Position, Direction)>,
    total_loops: usize,
    context: Vec<Context>,
}

pub(crate) type Position = (i8, i8);

#[derive(Clone)]
pub(crate) struct Context {
    guard: Position,
    symbol: char,
    direction: Direction,
    visited: HashSet<Position>,
}

impl Game {
    pub fn new(input: &str, mode: Part) -> Self {
        let grid = Grid::new(input);
        let guard = grid.find(GUARD);

        Self {
            width: grid.width,
            height: grid.height,
            grid: Grid::new(input),
            state: Default::default(),
            direction: Default::default(),
            visited: HashSet::from([guard]),
            loops: HashSet::new(),
            total_loops: 0,
            context: Vec::with_capacity(1),
            mode,
            guard,
        }
    }

    /// Moves the guard by one step.
    ///
    /// If the new position has an obstacle, turn right and repeat the whole process again
    /// until it reaches the exit.
    pub fn update(&mut self) {
        self.state = State::Running;

        let next = next(self.guard, &self.direction);
        let Some(char) = self.grid.get(next) else {
            self.restore();
            return;
        };

        // If it's an obstacle, turn the guard to the new direction.
        if is_obstacle(char) {
            if self.is_loop(next) {
                return;
            }

            self.loops.insert((next, self.direction));
            self.update_direction();
        } else {
            // Save the context and switch to part 2.
            if self.mode == Part::Two && self.context.is_empty() {
                self.save(next);
                return;
            }
            self.update_guard(next);
        }
    }

    /// Checks whether the next postion is a loop (part 2).
    fn is_loop(&mut self, next: Position) -> bool {
        match self.mode {
            Part::One => false,
            Part::Two => {
                if self.loops.contains(&(next, self.direction)) {
                    self.total_loops += 1;
                    self.restore();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn update_guard(&mut self, next: Position) {
        self.visited.insert(next);
        self.guard = next;
    }

    fn update_direction(&mut self) {
        self.direction = turn(&self.direction);
    }

    pub fn reset(&mut self) {
        self.state = Default::default();
        self.guard = self.grid.find(GUARD);
        self.visited = HashSet::from([self.guard]);
        self.direction = Default::default();
    }

    fn save(&mut self, next: Position) {
        self.context.push(Context {
            guard: next,
            symbol: self.grid.get(next).expect("symbol should not be empty"),
            direction: self.direction,
            visited: self.visited.clone(),
        });

        self.grid.set(next, FIRE);
    }

    fn restore(&mut self) {
        match self.mode {
            Part::One => self.state = State::Done,
            Part::Two => {
                self.state = State::Running;

                if let Some(ctx) = self.context.pop() {
                    self.guard = ctx.guard;
                    self.direction = ctx.direction;
                    self.visited = ctx.visited;
                    self.visited.insert(self.guard);

                    // Remove fire symbol
                    self.grid.set(ctx.guard, ctx.symbol);
                    self.loops = HashSet::new();
                }
            }
        }
    }

    pub fn is_guard(&self, (x, y): Position) -> bool {
        x == self.guard.0 && y == self.guard.1
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, State::Running)
    }

    pub fn play(&mut self) {
        self.state = State::Running;
    }

    pub fn pause(&mut self) {
        self.state = State::Paused;
    }

    pub fn total_visits(&self) -> usize {
        self.visited.len()
    }

    pub fn total_loops(&self) -> usize {
        self.total_loops
    }

    pub fn is_visited(&self, position: &Position) -> bool {
        self.visited.contains(position)
    }
}

#[derive(Clone, Store)]
pub(crate) struct Grid {
    pub cells: Vec<char>,
    width: usize,
    height: usize,
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self
            .cells
            .as_slice()
            .chunks(self.width)
            .map(|s| s.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");

        write!(f, "{}", data)
    }
}

impl Grid {
    fn new(input: &str) -> Self {
        let rows = input.split("\n").collect::<Vec<_>>();

        Self {
            width: rows[0].len(),
            height: rows.len(),
            cells: rows.into_iter().flat_map(|s| s.chars()).collect(),
        }
    }

    fn set(&mut self, position: Position, symbol: char) {
        let i = self.to_index(position);
        self.cells[i] = symbol;
    }

    fn get(&self, (x, y): Position) -> Option<char> {
        if x < 0 || x >= self.width as i8 || y < 0 || y >= self.height as i8 {
            return None;
        }

        self.cells.get(self.to_index((x, y))).copied()
    }

    fn find(&self, symbol: char) -> Position {
        self.to_position(
            self.cells
                .iter()
                .position(|c| *c == symbol)
                .expect("position should be found"),
        )
    }

    pub fn to_index(&self, (x, y): Position) -> usize {
        (y * self.width as i8 + x) as usize
    }

    pub fn to_position(&self, i: usize) -> Position {
        (i as i8 % self.width as i8, i as i8 / self.width as i8)
    }
}

fn add(src: Position, dst: Position) -> Position {
    (src.0 + dst.0, src.1 + dst.1)
}

fn next(src: Position, direction: &Direction) -> Position {
    add(src, dir(direction))
}

fn dir(direction: &Direction) -> Position {
    match direction {
        Direction::Up => (0, -1),
        Direction::Down => (0, 1),
        Direction::Left => (-1, 0),
        Direction::Right => (1, 0),
    }
}

fn turn(direction: &Direction) -> Direction {
    match direction {
        Direction::Up => Direction::Right,
        Direction::Down => Direction::Left,
        Direction::Left => Direction::Up,
        Direction::Right => Direction::Down,
    }
}

pub(crate) const GUARD: char = '^';
const FIRE: char = 'O';
const TREE: char = '#';
const OBSTACLES: [char; 2] = [FIRE, TREE];
const GUARDS: [char; 4] = ['^', '>', 'v', '<'];

pub(crate) fn is_obstacle(symbol: char) -> bool {
    OBSTACLES.contains(&symbol)
}

pub(crate) fn is_guard(symbol: char) -> bool {
    GUARDS.contains(&symbol)
}

pub(crate) fn avatar(char: char) -> &'static str {
    match char {
        TREE => "🎄",
        FIRE => "🔥",
        _ if is_guard(char) => "⛄",
        _ => " ",
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const INPUT: &str = "....#.....
.........#
..........
..#.......
.......#..
..........
.#..^.....
........#.
......#...";

    fn game(mode: Part) -> Game {
        Game::new(INPUT, mode)
    }

    #[test]
    fn test_grid() {
        let mut grid = Grid::new(INPUT);

        // Properties
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 9);

        // Basic ops
        assert_eq!(grid.find(GUARD), (4, 6));
        grid.set((4, 6), '0');
        assert_eq!(grid.find('0'), (4, 6));
        assert_eq!(grid.get((4, 6)), Some('0'));

        // Bound check
        assert_eq!(grid.get((grid.width as i8 + 1, 0)), None);
        assert_eq!(grid.get((0, grid.height as i8 + 1)), None);
    }

    #[test]
    fn test_update() {
        let mut game = game(Part::One);

        assert_eq!(game.guard, (4, 6));

        // Move forward before reaching the tree
        (0..5).for_each(|_| game.update());
        assert_eq!(game.guard, (4, 1));
        assert_eq!(game.direction, Direction::Up);

        // Move one more step will turn right
        game.update();
        assert_eq!(game.guard, (4, 1));
        assert_eq!(game.direction, Direction::Right);

        // Walk in to new cell
        game.update();
        assert_eq!(game.guard, (5, 1));
        assert_eq!(game.direction, Direction::Right);
    }
}
