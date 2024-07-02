use std::ops::{Add, Rem};

pub type CoordType = u32;

/// Single coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl PartialOrd for Coord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for Coord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
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

impl Pos {
    pub fn new(x: Coord, y: Coord) -> Self {
        Self { x, y }
    }
    /// Add position with wrapping inside some rectangle
    pub fn wrapping_add(self, rhs: Self, rect: Self) -> Self {
        Self::new((self.x + rhs.x) % rect.x, (self.y + rhs.y) % rect.y)
    }
    /// `x * y`
    pub(crate) fn area(&self) -> CoordType {
        self.x.0 * self.y.0
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
