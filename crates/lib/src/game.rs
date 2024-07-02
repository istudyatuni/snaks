use std::{cell::RefCell, collections::VecDeque};

use rand::random;

use crate::types::*;

/// Game state
///
/// `(0, 0)` is at top left position
#[derive(Debug)]
pub struct Game {
    size: Pos,
    snake: RefCell<VecDeque<Pos>>,
    food: RefCell<Pos>,
    direction: RefCell<MoveTo>,
    stats: RefCell<Stats>,
}

impl Game {
    pub fn new(width: Coord, height: Coord, start: Pos) -> Self {
        let size = Pos::new(width, height);
        let mut snake = VecDeque::with_capacity(width.0.saturating_mul(height.0) as usize);
        snake.push_back(start);
        let s = Self {
            size,
            snake: RefCell::new(snake),
            food: RefCell::new((0, 0).into()),
            direction: RefCell::new(MoveTo::Right),
            stats: RefCell::new(Stats::new()),
        };
        s.update_food();
        s
    }
    /// Determinine next position and move snake
    pub fn auto_move(&self) {
        self.move_snake(self.direction());
    }
    /// Move snake to new position
    pub fn move_to(&self, to: MoveTo) {
        // do not move back
        // there is still a problem when changing direction very fast
        if self.direction() == to.opposite() {
            return;
        }
        self.set_direction(to);
    }

    fn move_snake(&self, to: MoveTo) {
        if self.stats().status != GameStatus::Play {
            return;
        }

        let next = self.get_next_pos(to);
        if self.is_in_snake(next) {
            self.set_status(GameStatus::Fail);
            return;
        }
        if next == self.food() {
            self.grow_to_pos(next);
        } else {
            self.move_to_pos(next);
        }
    }

    pub fn size(&self) -> Pos {
        self.size
    }
    pub fn snake(&self) -> Vec<Pos> {
        // todo: optimize clone
        self.snake.borrow().to_owned().into()
    }
    pub fn food(&self) -> Pos {
        self.food.borrow().to_owned()
    }
    pub fn stats(&self) -> Stats {
        self.stats.borrow().clone()
    }

    fn head(&self) -> Pos {
        self.snake
            .borrow()
            .back()
            .expect("snake can't be empty")
            .to_owned()
    }
    fn direction(&self) -> MoveTo {
        self.direction.borrow().to_owned()
    }
    fn is_in_snake(&self, pos: Pos) -> bool {
        self.snake.borrow().contains(&pos)
    }

    fn move_to_pos(&self, to: Pos) {
        self.snake.borrow_mut().push_back(to);
        self.snake.borrow_mut().pop_front();
    }
    fn grow_to_pos(&self, to: Pos) {
        self.snake.borrow_mut().push_back(to);
        self.add_score();
        self.update_food();
    }
    fn update_food(&self) {
        if !self.can_place_new_food() {
            self.set_status(GameStatus::Win);
            return;
        }

        let food = self.get_new_food();
        *self.food.borrow_mut() = food;
    }
    fn set_status(&self, status: GameStatus) {
        self.stats.borrow_mut().status = status;
    }
    fn add_score(&self) {
        self.stats.borrow_mut().score += 1;
    }
    fn set_direction(&self, to: MoveTo) {
        *self.direction.borrow_mut() = to;
    }

    fn get_next_pos(&self, to: MoveTo) -> Pos {
        let (x, y) = (self.size.x.0, self.size.y.0);
        let shift = match to {
            MoveTo::Left => (x - 1, 0),
            MoveTo::Right => (x + 1, 0),
            MoveTo::Up => (0, y - 1),
            MoveTo::Down => (0, y + 1),
        };
        self.head().wrapping_add(shift.into(), self.size)
    }
    fn get_new_food(&self) -> Pos {
        let size = self.size;
        loop {
            let food: Pos = (
                random::<CoordType>() % size.x.0,
                random::<CoordType>() % size.y.0,
            )
                .into();
            if !self.is_in_snake(food) {
                return food;
            }
        }
    }

    fn can_place_new_food(&self) -> bool {
        self.snake.borrow().len() < self.size.area() as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Play,
    Fail,
    Win,
}

#[derive(Debug, Clone)]
pub struct Stats {
    /// Literally snake size - 1
    pub score: usize,
    pub status: GameStatus,
}

impl Stats {
    fn new() -> Self {
        Self {
            score: 0,
            status: GameStatus::Play,
        }
    }
}