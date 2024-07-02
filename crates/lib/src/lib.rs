use std::{
    cell::RefCell,
    collections::VecDeque,
    ops::{Add, Rem},
};

use rand::random;

pub type CoordType = u32;

/// Game state
///
/// `(0, 0)` is at top left position
#[derive(Debug)]
pub struct Game {
    size: Pos,
    snake: RefCell<VecDeque<Pos>>,
    food: RefCell<Pos>,
    status: RefCell<GameStatus>,
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
            status: RefCell::new(GameStatus::Play),
        };
        s.update_food();
        s
    }
    /// Move snake to new position
    pub fn move_to(&self, to: MoveTo) {
        if self.status() == GameStatus::Fail {
            return;
        }

        let next = self.get_next_pos(to);
        if self.is_in_snake(next) {
            self.set_fail_status();
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
    pub fn status(&self) -> GameStatus {
        self.status.borrow().to_owned()
    }

    fn head(&self) -> Pos {
        self.snake
            .borrow()
            .back()
            .expect("snake can't be empty")
            .to_owned()
    }

    fn move_to_pos(&self, to: Pos) {
        self.snake.borrow_mut().push_back(to);
        self.snake.borrow_mut().pop_front();
    }
    fn grow_to_pos(&self, to: Pos) {
        self.snake.borrow_mut().push_back(to);
        self.update_food();
    }
    fn update_food(&self) {
        let food = self.get_new_food();
        *self.food.borrow_mut() = food;
    }
    fn set_fail_status(&self) {
        *self.status.borrow_mut() = GameStatus::Fail;
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

    fn is_in_snake(&self, pos: Pos) -> bool {
        self.snake.borrow().contains(&pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Play,
    Fail,
    // todo: win?
}

/// Single coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Coord(CoordType);

impl From<CoordType> for Coord {
    fn from(value: CoordType) -> Self {
        Self(value)
    }
}

impl Add for Coord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl Rem for Coord {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0.rem(rhs.0))
    }
}

/// Zero based when used as coordinate
///
/// One based when used as size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    /// Coordinate on horizontal axis
    pub x: Coord,
    /// Coordinate on vertical axis
    pub y: Coord,
}

/*#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub w: Coord,
    pub h: Coord,
}*/

impl Pos {
    pub fn new(x: Coord, y: Coord) -> Self {
        Self { x, y }
    }
    pub fn wrapping_add(self, rhs: Self, wrap: Self) -> Self {
        Self::new((self.x + rhs.x) % wrap.x, (self.y + rhs.y) % wrap.y)
    }
}

impl From<(CoordType, CoordType)> for Pos {
    fn from((x, y): (CoordType, CoordType)) -> Self {
        Self::new(x.into(), y.into())
    }
}

impl From<Pos> for (usize, usize) {
    fn from(v: Pos) -> Self {
        (v.x.0 as usize, v.y.0 as usize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTo {
    Left,
    Right,
    Up,
    Down,
}
