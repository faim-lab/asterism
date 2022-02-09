use std::collections::BTreeMap;
use std::fmt::Debug;

use asterism::{Event, Logic, OutputTable, Reaction};
use macroquad::math::IVec2;

pub struct CollisionData<ID> {
    pub solid: bool,
    pub fixed: bool,
    pub id: ID,
}

impl<ID> CollisionData<ID> {
    pub fn new(solid: bool, fixed: bool, id: ID) -> Self {
        Self { solid, fixed, id }
    }
}

pub struct TileMapCollision<TileID: Debug, EntID> {
    pub map: Vec<Vec<Option<TileID>>>,
    pub tile_solid: BTreeMap<TileID, bool>,
    pub positions: Vec<IVec2>,
    pub metadata: Vec<CollisionData<EntID>>,
    pub amt_moved: Vec<IVec2>,
    pub contacts: Vec<Contact>,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Contact {
    Ent(usize, usize),
    Tile(usize, IVec2),
}

use asterism::collision::CollisionEventType;
impl Event for Contact {
    type EventType = CollisionEventType;

    fn get_type(&self) -> &Self::EventType {
        &CollisionEventType::Touching
    }
}

pub enum CollisionReaction<TileID, EntID> {
    SetTileAtPos(IVec2, TileID),
    RemoveTileAtPos(IVec2),
    SetEntPos(usize, IVec2),
    SetEntVel(usize, IVec2),
    SetEntData(usize, bool, bool, EntID), // idx, solid, fixed, id
    RemoveEnt(usize),
}

#[derive(Clone, Copy)]
pub enum ColIdent {
    Position(IVec2),
    EntIdx(usize),
}

#[derive(Clone)]
pub enum TileMapColData<TileID, EntID> {
    Position {
        pos: IVec2,
        solid: bool,
        id: TileID,
    },
    Ent {
        pos: IVec2,
        amt_moved: IVec2,
        solid: bool,
        fixed: bool,
        id: EntID,
    },
}

impl<TileID, EntID> Reaction for CollisionReaction<TileID, EntID> {}

impl<TileID: Copy + Eq + Ord + Debug, EntID: Copy> Logic for TileMapCollision<TileID, EntID> {
    type Event = Contact;
    type Reaction = CollisionReaction<TileID, EntID>;
    type Ident = ColIdent;
    type IdentData = TileMapColData<TileID, EntID>;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            CollisionReaction::SetTileAtPos(pos, id) => {
                self.map[pos.y as usize][pos.x as usize] = Some(*id);
            }
            CollisionReaction::RemoveTileAtPos(pos) => {
                self.map[pos.y as usize][pos.x as usize] = None;
            }
            CollisionReaction::SetEntPos(idx, pos) => {
                self.positions[*idx] = *pos;
            }
            CollisionReaction::SetEntVel(idx, vel) => {
                self.amt_moved[*idx] = *vel;
            }
            CollisionReaction::SetEntData(idx, solid, fixed, id) => {
                self.metadata[*idx].solid = *solid;
                self.metadata[*idx].fixed = *fixed;
                self.metadata[*idx].id = *id;
            }
            CollisionReaction::RemoveEnt(idx) => {
                self.positions.remove(*idx);
                self.amt_moved.remove(*idx);
                self.metadata.remove(*idx);
            }
        };
    }

    fn get_ident_data(&self, ident: Self::Ident) -> Self::IdentData {
        match ident {
            ColIdent::Position(pos) => {
                let id = self.map[pos.y as usize][pos.x as usize]
                    .unwrap_or_else(|| panic!("no tile at position {}", pos));
                TileMapColData::Position {
                    pos,
                    solid: self.tile_solid(&id),
                    id,
                }
            }
            ColIdent::EntIdx(idx) => {
                let meta = &self.metadata[idx];
                TileMapColData::Ent {
                    pos: self.positions[idx],
                    amt_moved: self.amt_moved[idx],
                    solid: meta.solid,
                    fixed: meta.fixed,
                    id: meta.id,
                }
            }
        }
    }

    fn update_ident_data(&mut self, ident: Self::Ident, data: Self::IdentData) {
        match (ident, data) {
            (ColIdent::Position(pos_ident), TileMapColData::Position { pos, solid, id }) => {
                if pos_ident != pos {
                    *self.tile_at_pos_mut(&pos_ident) = None;
                }
                *self.tile_at_pos_mut(&pos) = Some(id);
                self.tile_solid.insert(id, solid);
            }
            (
                ColIdent::EntIdx(idx),
                TileMapColData::Ent {
                    pos,
                    amt_moved,
                    solid,
                    fixed,
                    id,
                },
            ) => {
                self.positions[idx] = pos;
                self.amt_moved[idx] = amt_moved;
                let meta = &mut self.metadata[idx];
                meta.solid = solid;
                meta.fixed = fixed;
                meta.id = id;
            }
            (ColIdent::Position(_), _) => {
                unreachable!("cannot update tile information for an entity")
            }
            (ColIdent::EntIdx(_), _) => {
                unreachable!("cannot update entity information for a tile")
            }
        }
    }
}

impl<TileID: Eq + Ord + Copy + Debug, EntID> TileMapCollision<TileID, EntID> {
    pub fn new(width: usize, height: usize) -> Self {
        let mut collision = Self {
            map: Vec::new(),
            tile_solid: BTreeMap::new(),
            positions: Vec::new(),
            metadata: Vec::new(),
            amt_moved: Vec::new(),
            contacts: Vec::new(),
        };
        collision.clear_and_resize_map(width, height);
        collision
    }

    pub fn clear_and_resize_map(&mut self, width: usize, height: usize) {
        self.map.clear();

        self.map.resize_with(height, || {
            let mut vec = Vec::with_capacity(width);
            vec.resize_with(width, || None);
            vec
        });
    }

    pub fn clear_entities(&mut self) {
        self.positions.clear();
        self.amt_moved.clear();
        self.metadata.clear();
    }

    pub fn clear_tile_data(&mut self) {
        self.tile_solid.clear();
    }

    pub fn update(&mut self) {
        self.contacts.clear();

        // check for contacts
        // ent vs tile
        for (i, pos_i) in self.positions.iter().enumerate() {
            if self.tile_at_pos(pos_i).is_some() {
                self.contacts.push(Contact::Tile(i, *pos_i));
            }
        }

        // ent vs ent
        for (i, (pos_i, meta_i)) in self.positions.iter().zip(self.metadata.iter()).enumerate() {
            for (j, (pos_j, meta_j)) in self
                .positions
                .iter()
                .zip(self.metadata.iter())
                .enumerate()
                .skip(i + 1)
            {
                if pos_i == pos_j {
                    let mut i = i;
                    let mut j = j;

                    if meta_i.fixed && !meta_j.fixed {
                        std::mem::swap(&mut i, &mut j);
                    }

                    self.contacts.push(Contact::Ent(i, j));
                }
            }
        }

        // restitute
        for contact in self.contacts.iter() {
            match contact {
                Contact::Tile(i, pos) => {
                    if self.positions[*i] != *pos {
                        continue;
                    }
                    if !self.metadata[*i].solid || self.metadata[*i].fixed {
                        continue;
                    }
                    if let Some(tile_id) = self.tile_at_pos(pos) {
                        if self.tile_solid(tile_id) {
                            let moved = normalize(self.amt_moved[*i]);
                            let mut pos = self.positions[*i];
                            pos -= moved;
                            self.restitute_ent(&mut pos, moved);
                            self.positions[*i] = pos;
                        }
                    }
                }

                Contact::Ent(i, j) => {
                    if self.positions[*i] != self.positions[*j] {
                        continue;
                    }
                    if !self.metadata[*i].solid
                        || self.metadata[*i].fixed
                        || !self.metadata[*j].solid
                    {
                        continue;
                    }
                    let moved = normalize(self.amt_moved[*i]);
                    let mut pos = self.positions[*i];
                    pos -= moved;
                    self.restitute_ent(&mut pos, moved);
                    self.positions[*i] = pos;

                    if !self.metadata[*j].fixed {
                        let moved = normalize(self.amt_moved[*j]);
                        let mut pos = self.positions[*j];
                        self.restitute_ent(&mut pos, moved);
                        self.positions[*j] = pos;
                    }
                }
            }
        }
    }

    fn restitute_ent(&self, pos: &mut IVec2, moved: IVec2) {
        if moved == IVec2::ZERO {
            // this is miserable
            if !self.in_bounds(*pos - IVec2::new(0, 1)) {
                *pos -= IVec2::new(0, 1);
            } else if !self.in_bounds(*pos + IVec2::new(0, 1)) {
                *pos += IVec2::new(0, 1);
            } else if !self.in_bounds(*pos - IVec2::new(1, 0)) {
                *pos -= IVec2::new(1, 0);
            } else if !self.in_bounds(*pos + IVec2::new(1, 0)) {
                *pos += IVec2::new(1, 0);
            }
        }
        let mut new_pos = *pos;
        // check collision against map
        while let Some(tile_id) = self.map[new_pos.y as usize][new_pos.x as usize] {
            if self.tile_solid(&tile_id) {
                new_pos -= moved;
                if !self.in_bounds(new_pos) {
                    break;
                }
            } else {
                *pos = new_pos;
                break;
            }
        }
    }

    fn tile_at_pos(&self, pos: &IVec2) -> &Option<TileID> {
        &self.map[pos.y as usize][pos.x as usize]
    }

    fn tile_at_pos_mut(&mut self, pos: &IVec2) -> &mut Option<TileID> {
        &mut self.map[pos.y as usize][pos.x as usize]
    }

    fn tile_solid(&self, tile_id: &TileID) -> bool {
        *self
            .tile_solid
            .get(tile_id)
            .unwrap_or_else(|| panic!("not specified if tile {:?} is solid or not", tile_id))
    }

    fn in_bounds(&self, pos: IVec2) -> bool {
        pos.x < self.map[0].len() as i32
            && pos.y < self.map.len() as i32
            && pos.x >= 0
            && pos.y >= 0
    }
}

fn normalize(vec2: IVec2) -> IVec2 {
    let mut vec2 = vec2;
    if vec2.x > 1 {
        vec2.x = 1;
    }
    if vec2.x < -1 {
        vec2.x = -1;
    }
    if vec2.y > 1 {
        vec2.y = 1;
    }
    if vec2.y < -1 {
        vec2.y = -1;
    }
    vec2
}

type QueryIdent<TileID, EntID> = (
    <TileMapCollision<TileID, EntID> as Logic>::Ident,
    <TileMapCollision<TileID, EntID> as Logic>::IdentData,
);

impl<TileID, EntID> OutputTable<QueryIdent<TileID, EntID>> for TileMapCollision<TileID, EntID>
where
    TileID: Debug + Copy + Eq + Ord,
    EntID: Copy,
{
    fn get_table(&self) -> Vec<QueryIdent<TileID, EntID>> {
        let mut idents = Vec::new();

        // tiles
        for (y, row) in self.map.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if tile.is_some() {
                    let ident = ColIdent::Position(IVec2::new(x as i32, y as i32));
                    idents.push((ident, self.get_ident_data(ident)));
                }
            }
        }

        // entities
        let mut ents = (0..self.positions.len())
            .map(|idx| {
                let ident = ColIdent::EntIdx(idx);
                (ident, self.get_ident_data(ident))
            })
            .collect::<Vec<_>>();
        idents.append(&mut ents);

        idents
    }
}

impl<TileID: Debug, EntID> OutputTable<Contact> for TileMapCollision<TileID, EntID> {
    fn get_table(&self) -> Vec<Contact> {
        self.contacts.to_vec()
    }
}
