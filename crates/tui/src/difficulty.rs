use std::{fmt::Display, str::FromStr, time::Duration};

use super::app::{dur2fps, fps};

#[derive(Debug, Default, Clone)]
pub struct Difficulty {
    pub prev: DifficultyKind,
    pub kind: DifficultyKind,
    pub fps: DifficultyFps,
}

impl Difficulty {
    pub fn update_fps(&mut self) {
        self.fps = self.kind.to_fps()
    }
}

#[derive(Debug, Clone)]
pub struct DifficultyFps(Duration);

impl DifficultyFps {
    pub fn duration(&self) -> Duration {
        self.0
    }
}

impl Default for DifficultyFps {
    fn default() -> Self {
        DifficultyKind::default().to_fps()
    }
}

impl Display for DifficultyFps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", dur2fps(self.0))
    }
}

pub const DIFFICULTIES: [DifficultyKind; 5] = [
    DifficultyKind::Easy,
    DifficultyKind::Normal,
    DifficultyKind::Medium,
    DifficultyKind::Hard,
    DifficultyKind::Impossible,
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DifficultyKind {
    Easy,
    #[default]
    Normal,
    Medium,
    Hard,
    Impossible,
    Secret,
}

impl DifficultyKind {
    fn to_fps(self) -> DifficultyFps {
        let f = match self {
            Self::Easy => 5,
            Self::Normal => 10,
            Self::Medium => 15,
            Self::Hard => 30,
            Self::Impossible => 60,
            Self::Secret => 100,
        };
        DifficultyFps(fps(f))
    }
    /// Use in selector
    pub fn next(self) -> Self {
        match self {
            Self::Easy => Self::Normal,
            Self::Normal => Self::Medium,
            Self::Medium => Self::Hard,
            Self::Hard => Self::Impossible,
            Self::Impossible => Self::Easy,
            Self::Secret => Self::Easy,
        }
    }
    /// Use in selector
    pub fn prev(self) -> Self {
        match self {
            Self::Easy => Self::Impossible,
            Self::Normal => Self::Easy,
            Self::Medium => Self::Normal,
            Self::Hard => Self::Medium,
            Self::Impossible => Self::Hard,
            Self::Secret => Self::Impossible,
        }
    }
}

impl Display for DifficultyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Easy => "Easy",
            Self::Normal => "Normal",
            Self::Medium => "Medium",
            Self::Hard => "Hard",
            Self::Impossible => "Impossible",
            Self::Secret => "Secret",
        };
        f.pad(s)
    }
}

impl FromStr for DifficultyKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let d = match s.as_str() {
            "easy" => Self::Easy,
            "normal" => Self::Normal,
            "medium" => Self::Medium,
            "hard" => Self::Hard,
            "impossible" => Self::Impossible,
            "secret" => Self::Secret,
            _ => return Err("unknown difficulty"),
        };
        Ok(d)
    }
}
