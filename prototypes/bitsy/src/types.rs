use macroquad::{color::*, input::KeyCode, math::IVec2};

/// generates identifier structs (i got tired of typing all of them out)
macro_rules! id_impl_new {
    ($([$($derive:meta)*] $id_type:ident),*) => {
        $(
            $(#[$derive])*
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

id_impl_new!([derive(Hash, Ord, PartialOrd)] TileID, [derive(Hash, Ord, PartialOrd)] CharacterID, [derive(Hash, Ord, PartialOrd)] RsrcID, [derive(Ord, PartialOrd)] LinkID, [derive(Hash)] UserQueryID);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) enum QueryType {
    ContactOnly,
    ContactRoom,
    LinkingEvent,
    LinkingIdent,
    TraverseRoom,
    ControlEvent,
    ControlFilter,
    ResourceEvent,
    ResourceIdent,
    User(UserQueryID),
}

pub enum Ent {
    Player(Player),
    Tile(Tile),
    Character(Character),
}

// the stonks meme but it says derive
#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum EntID {
    Player,
    Tile(TileID),
    Character(CharacterID),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum ActionID {
    Left,
    Right,
    Up,
    Down,
}

// players are unfixed
pub struct Player {
    pub pos: IVec2,
    pub amt_moved: IVec2,
    pub color: Color,
    pub inventory: Vec<(RsrcID, Resource)>,
    pub controls: Vec<(ActionID, KeyCode, bool)>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: IVec2::ZERO,
            amt_moved: IVec2::ZERO,
            color: WHITE,
            inventory: Vec::new(),
            controls: vec![
                (ActionID::Up, KeyCode::Up, true),
                (ActionID::Down, KeyCode::Down, true),
                (ActionID::Left, KeyCode::Left, true),
                (ActionID::Right, KeyCode::Right, true),
            ],
        }
    }

    pub fn set_control_map(&mut self, action: ActionID, keycode: KeyCode, valid: bool) {
        let (_, keycode_old, valid_old) = self
            .controls
            .iter_mut()
            .find(|(act_id, ..)| *act_id == action)
            .unwrap();
        *keycode_old = keycode;
        *valid_old = valid;
    }

    pub fn add_inventory_item(&mut self, id: RsrcID, rsrc: Resource) {
        self.inventory.push((id, rsrc));
    }
}

// tiles can be solid or not
#[derive(Clone, Copy)]
pub struct Tile {
    pub solid: bool,
    pub color: Color,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            solid: false,
            // randomly generate tile color using hsl
            color: {
                use macroquad::rand::gen_range;
                hsl_to_rgb(
                    gen_range(0.0, 1.0),
                    gen_range(0.7, 1.0),
                    gen_range(0.3, 0.7),
                )
            },
        }
    }
}

// characters are fixed
pub struct Character {
    pub pos: IVec2,
    pub color: Color,
}

impl Character {
    pub fn new() -> Self {
        Self {
            pos: IVec2::ZERO,
            color: LIME,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Resource {
    pub val: u16,
    pub min: u16,
    pub max: u16,
}

impl Resource {
    pub fn new() -> Self {
        Self {
            val: 0,
            min: u16::MIN,
            max: u16::MAX,
        }
    }
}

use crate::collision::Contact;
use asterism::control::ControlEvent;
use asterism::resources::ResourceEvent;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CollisionEnt {
    Player,
    Character,
}

pub type CtrlEvent = ControlEvent<ActionID>;
pub type ColEvent = Contact;
pub type RsrcEvent = ResourceEvent<RsrcID>;
