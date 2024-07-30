use std::{cell::RefCell, collections::VecDeque};

use rand::random;

use crate::types::*;

/// Game state
///
/// `(0, 0)` is at top left position
#[derive(Debug, Default)]
pub struct Game {
    /// Field's size
    size: Pos,
    /// List of snake's blocks
    snake: RefCell<VecDeque<Pos>>,
    food: RefCell<Pos>,
    /// Current snake's direction
    direction: RefCell<MoveTo>,
    stats: RefCell<Stats>,
    last_event: RefCell<Option<GameEvent>>,
}

impl Game {
    pub fn new(size: Pos) -> Self {
        let mut snake = VecDeque::with_capacity(size.x.0.saturating_mul(size.y.0) as usize);
        snake.push_back((size.x / 2, size.y / 2).into());
        let s = Self {
            size,
            snake: RefCell::new(snake),
            last_event: RefCell::new(Some(GameEvent::GameStart)),
            ..Default::default()
        };
        s.update_food();
        s
    }
    /// Move snake
    pub fn move_snake(&self) {
        self.move_snake_to(self.direction());
    }
    /// Rotate snake to new direction
    pub fn rotate_to(&self, to: MoveTo) {
        // do not move back if snake has > 1 cells
        if self.direction() == to.opposite() && self.stats().score > 0 {
            return;
        }
        self.set_direction(to);
    }

    fn move_snake_to(&self, to: MoveTo) {
        if self.stats().status != GameStatus::Play {
            return;
        }

        let next = self.get_next_pos(to);
        if self.is_in_snake(next) {
            self.set_status(GameStatus::Fail);
            self.set_event(GameEvent::Fail);
            return;
        }
        if next == self.food() {
            self.grow_to_pos(next);
            self.set_event(GameEvent::FoodEat);
        } else {
            self.move_to_pos(next);
        }
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
    pub fn direction(&self) -> MoveTo {
        self.direction.borrow().to_owned()
    }
    pub fn head(&self) -> Pos {
        self.snake
            .borrow()
            .back()
            .expect("snake can't be empty")
            .to_owned()
    }
    /// Last occured event
    pub fn last_event(&self) -> Option<GameEvent> {
        self.last_event.borrow().to_owned()
    }
    /// Mark event processed
    pub fn forgot_event(&self, event: GameEvent) {
        let mut e = self.last_event.borrow_mut();
        if e.is_some_and(|e| e.eq(&event)) {
            e.take();
        }
    }

    fn is_in_snake(&self, pos: Pos) -> bool {
        self.snake.borrow().contains(&pos)
    }

    /// Move snake to position
    fn move_to_pos(&self, to: Pos) {
        self.snake.borrow_mut().push_back(to);
        self.snake.borrow_mut().pop_front();
    }
    /// Move snake to position and increase snake length
    fn grow_to_pos(&self, to: Pos) {
        self.snake.borrow_mut().push_back(to);
        self.add_score();
        self.update_food();
    }
    /// Calculate position for new food and place it
    fn update_food(&self) {
        if !self.can_place_new_food() {
            self.set_status(GameStatus::Win);
            self.set_event(GameEvent::Win);
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
    fn set_event(&self, event: GameEvent) {
        *self.last_event.borrow_mut() = Some(event)
    }

    /// Calculate next position depending on field size and direction
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
    /// Generate random position for food
    fn get_new_food(&self) -> Pos {
        let size = self.size;
        loop {
            let x = random::<CoordType>() % size.x.0;
            let y = random::<CoordType>() % size.y.0;
            let food = (x, y).into();
            if !self.is_in_snake(food) {
                return food;
            }
        }
    }

    /// Check if field has empty cells for food
    fn can_place_new_food(&self) -> bool {
        self.snake.borrow().len() < self.size.area() as usize
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    #[default]
    Play,
    Fail,
    Win,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    #[default]
    GameStart,
    FoodEat,
    Fail,
    Win,
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    /// Count of eaten food
    pub score: usize,
    pub status: GameStatus,
}
