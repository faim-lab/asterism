use std::collections::BTreeMap;

use asterism::{Event, EventType, Logic, Reaction};
use macroquad::math::IVec2;

struct CollisionData<ID> {
    fixed: bool,
    solid: bool,
    id: ID,
}

pub struct TileMapCollision<TileID, EntID> {
    map: Vec<Vec<Option<TileID>>>,
    tile_solid: BTreeMap<TileID, bool>,
    positions: Vec<IVec2>,
    metadata: Vec<CollisionData<EntID>>,
    amt_moved: Vec<IVec2>,
}

pub enum CollisionEvent<TileID, EntID> {
    EntCollision {
        i: (usize, EntID),
        j: (usize, EntID),
    },
    TileCollision {
        ent: (usize, EntID),
        tile: (IVec2, TileID),
    },
}

use asterism::collision::CollisionEventType;
impl<TileID, EntID> Event for CollisionEvent<TileID, EntID> {
    type EventType = CollisionEventType;

    fn get_type(&self) -> &Self::EventType {
        CollisionEventType::Touching
    }
}

pub enum CollisionReaction<TileID, EntID> {
    SetTileAtPos(IVec2, TileID),
    RemoveTileAtPos(IVec2, TileID),
    SetEntPos(usize),
    RemoveEnt(usize),
    SetEntData(usize, bool, EntID),
}

#[derive(Clone, Copy)]
pub enum ColIdent {
    Position(IVec2),
    EntIdx(usize),
}

pub enum TileMapColData<TileID, EntID> {
    Position {
        pos: IVec2,
        solid: bool,
        id: TileID,
    },
    Ent {
        pos: IVec2,
        fixed: bool,
        solid: bool,
        id: EntID,
    },
}

impl<TileID, EntID> Reaction for CollisionReaction<TileID, EntID> {}

impl<TileID, EntID> Logic for TileMapCollision<TileID, EntID> {
    type Event = CollisionEvent<TileID, EntID>;
    type Reaction = CollisionReaction<TileID, EntID>;
    type Ident = ColIdent;
    type IdentData = TileMapColData<TileID, EntID>;

    fn check_predicate(&self, event: &Self::Event) -> bool {
        todo!()
    }

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        todo!()
    }

    fn get_synthesis(&self, ident: Self::Ident) -> Self::IdentData {
        todo!()
    }

    fn update_synthesis(&mut self, ident: Self::Ident, data: Self::IdentData) {
        todo!()
    }
}

impl<TileID, EntID> TileMapCollision<TileID, EntID> {
    pub fn new() -> Self {
        Self {
            map: Vec::new(),
            tile_solid: BTreeMap::new(),
            positions: Vec::new(),
            metadata: Vec::new(),
            amt_moved: Vec::new(),
        }
    }

    pub fn clear_and_resize_map(&mut self, width: usize, height: usize) {
        self.map.clear();
        self.map.resize_with(height, || {
            let mut vec = Vec::with_capacity(width);
            vec.resize_with(width, || None);
            vec
        });
    }

    pub fn clear_tile_data(&mut self) {
        self.tile_solid.clear();
    }

    pub fn update(&mut self) {}
}

// impl Logic for TileMapCollision {
//     type Event = ;
//     type Reaction = ;
//     type Ident = ;
//     type IdentData = ;
// }
