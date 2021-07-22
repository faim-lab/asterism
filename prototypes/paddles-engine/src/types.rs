use macroquad::{input::KeyCode, math::Vec2};

/// generates identifier structs (i got tired of typing all of them out). example: `id_impl_new!([derive(PartialOrd, Ord)] ScoreID)` expands out to
///
/// ```
/// #[derive(PartialOrd, Ord)]
/// #[derive(Clone, Copy, PartialEq, Eq)]
/// pub struct ScoreID(usize);
/// impl ScoreID {
///     pub fn new(idx: usize) -> Self {
///         Self(idx)
///     }
///     pub fn idx(&self) -> usize {
///         self.0
///     }
/// }
/// ```
macro_rules! id_impl_new {
    ($([$($derive:meta)*] $id_type:ident),*) => {
        $(
            $(#[$derive])*
            #[derive(Clone, Copy, PartialEq, Eq)]
            pub struct $id_type(usize);

            impl $id_type {
                pub fn new(idx: usize) -> Self {
                    Self(idx)
                }

                pub fn idx(&self) -> usize {
                    self.0
                }
            }
        )*
    };
}

id_impl_new!([] PaddleID, [] WallID, [] BallID, [derive(PartialOrd, Ord, Debug)] ScoreID, [derive(PartialOrd, Ord, Debug)] ActionID, [derive(Hash, Debug)] UserQueryID);

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum QueryType {
    CtrlEvent,
    CtrlIdent,
    ColEvent,
    ColIdent,
    PhysEvent,
    PhysIdent,
    RsrcEvent,
    RsrcIdent,
    BallCol,
    User(UserQueryID),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CollisionEnt {
    Paddle,
    Wall,
    Ball,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum RsrcPool {
    Score(ScoreID),
}

#[derive(Default)]
pub struct Paddle {
    pub pos: Vec2,
    pub size: Vec2,
    pub controls: Vec<(ActionID, KeyCode, bool)>,
}

impl Paddle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn add_control_map(&mut self, keycode: KeyCode, valid: bool) -> ActionID {
        let act_id = ActionID(self.controls.len());
        self.controls.push((act_id, keycode, valid));
        act_id
    }
}

#[derive(Default)]
pub struct Ball {
    pub pos: Vec2,
    pub size: Vec2,
    pub vel: Vec2,
}

impl Ball {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn set_vel(&mut self, vel: Vec2) {
        self.vel = vel;
    }
}

#[derive(Default)]
pub struct Wall {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Wall {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }
}

#[derive(Default)]
pub struct Score {
    pub value: u16,
}

impl Score {
    pub(crate) const MIN: u16 = 0;
    pub(crate) const MAX: u16 = u16::MAX;
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_value(&mut self, value: u16) {
        self.value = value;
    }
}

use asterism::collision::CollisionEvent;
use asterism::control::ControlEvent;

pub type CtrlEvent = ControlEvent<ActionID>;
pub type CtrlIdent = (usize, Vec<asterism::control::Action<ActionID, KeyCode>>);
pub type ColEvent = CollisionEvent;
pub type ColIdent = (usize, asterism::collision::AabbColData<CollisionEnt>);
pub type RsrcIdent = (RsrcPool, (u16, u16, u16));
pub type RsrcEvent = asterism::resources::ResourceEvent<RsrcPool>;
pub type PhysIdent = (usize, asterism::physics::PointPhysData);
pub type PhysEvent = asterism::physics::PhysicsEvent;
