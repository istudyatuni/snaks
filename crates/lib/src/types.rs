use std::{
    fmt::Display,
    ops::{Add, Div, Rem},
};

pub type CoordType = u32;

/// Single coordinate
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Coord(pub(crate) CoordType);

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

impl Div<CoordType> for Coord {
    type Output = CoordType;

    fn div(self, rhs: CoordType) -> Self::Output {
        self.0.div(rhs)
    }
}

/// - 0-based when used as coordinate
/// - 1-based when used as size
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    /// Coordinate on horizontal axis
    pub x: Coord,
    /// Coordinate on vertical axis
    pub y: Coord,
}

impl Pos {
    pub const fn new(x: CoordType, y: CoordType) -> Self {
        Self {
            x: Coord(x),
            y: Coord(y),
        }
    }
    pub(crate) fn new_coord(x: Coord, y: Coord) -> Self {
        Self { x, y }
    }
    /// Add position with wrapping inside some rectangle
    pub fn wrapping_add(self, rhs: Self, rect: Self) -> Self {
        Self::new_coord((self.x + rhs.x) % rect.x, (self.y + rhs.y) % rect.y)
    }
    /// `x * y`
    pub(crate) fn area(&self) -> CoordType {
        self.x.0 * self.y.0
    }
}

impl From<(CoordType, CoordType)> for Pos {
    fn from((x, y): (CoordType, CoordType)) -> Self {
        Self::new(x, y)
    }
}

impl From<Pos> for (usize, usize) {
    fn from(v: Pos) -> Self {
        (v.x.0 as usize, v.y.0 as usize)
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x.0, self.y.0)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MoveTo {
    Left,
    #[default]
    Right,
    Up,
    Down,
}

impl MoveTo {
    pub(crate) fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}
