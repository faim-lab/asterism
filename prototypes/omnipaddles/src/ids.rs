#![allow(clippy::upper_case_acronyms)]
use asterism::resources::PoolInfo;

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Debug)]
pub enum Player {
    P1,
    P2,
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum ActionID {
    MoveUp,
    MoveDown,
    Serve,
    Quit,
}

impl Default for ActionID {
    fn default() -> Self {
        Self::MoveDown
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub enum CollisionID {
    Paddle(Player),
    Ball(usize),
    BounceWall,
    BreakWall(usize),
    ScoreWall(Player),
}

impl Default for CollisionID {
    fn default() -> Self {
        Self::Ball(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub enum PoolID {
    Points(Player),
}

impl PoolInfo for PoolID {
    fn max_value(&self) -> f64 {
        match self {
            Self::Points(_) => std::u8::MAX as f64,
        }
    }

    fn min_value(&self) -> f64 {
        match self {
            Self::Points(_) => std::u8::MIN as f64,
        }
    }
}
