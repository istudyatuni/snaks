use std::{
    cell::RefCell,
    collections::VecDeque,
    ops::{Add, Rem},
};

use rand::random;

pub type CoordType = i32;

/// Game state
///
/// (0, 0) is at top left position
#[derive(Debug)]
pub struct Game {
    size: Pos,
    snake: RefCell<VecDeque<Pos>>,
    food: RefCell<Pos>,
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
        };
        s.update_food();
        s
    }
    /// Move snake to new position
    pub fn move_to(&self, to: MoveTo) {
        let next = self.get_next_pos(to);
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

    fn head(&self) -> Pos {
        self.snake.borrow().back().expect("empty").to_owned()
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

    fn get_next_pos(&self, to: MoveTo) -> Pos {
        let shift = match to {
            MoveTo::Left => (-1, 0).into(),
            MoveTo::Right => (1, 0).into(),
            MoveTo::Up => (0, -1).into(),
            MoveTo::Down => (0, 1).into(),
        };
        self.head() + shift
    }
    fn get_new_food(&self) -> Pos {
        let size = self.size;
        let snake = self.snake();
        loop {
            let food: Pos = (
                random::<CoordType>() % size.x.0,
                random::<CoordType>() % size.y.0,
            )
                .into();
            if !snake.contains(&food) {
                return food;
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new(20.into(), 10.into(), (10, 5).into())
    }
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

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTo {
    Left,
    Right,
    Up,
    Down,
}
