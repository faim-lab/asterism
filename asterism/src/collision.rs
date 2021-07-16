//! # Collision logics
//!
//! Collision logics offer an illusion of physical space is provided by the fact that some game objects occlude the movement of others. They detect overlaps between subsets of entities and/or regions of space, and automatically trigger reactions when such overlaps occur.
//!
//! Note: Collision is hard and may be broken.

use crate::{tables::OutputTable, Event, EventType, Logic, Reaction};
use macroquad::math::Vec2;

/// Information for each contact. If the entities at the indices `i` and `j` are both unfixed or both fixed, then `i < j`. If one is unfixed and the other is fixed, `i` will be the index of the unfixed entity.
#[derive(PartialEq, Copy, Clone)]
pub struct Contact {
    /// The index of the first contact in `centers`, `half_sizes`, `velocities`, `metadata`, and `displacements`.
    pub i: usize,
    /// The index of the second contact in `centers`, `half_sizes`, `velocities`, `metadata`, and `displacements`.
    pub j: usize,
    /// The projected displacement of each contact---not actual restituted displacement. If both colliding bodies are fixed, or one of them is **not** solid, defaults to a `Vec2` with a magnitude of 0.0.
    pub displacement: Vec2,
}

impl Contact {
    /// Returns how much the contact should be restituted, not taking into account other possible contacts.
    fn get_restitution(&self) -> Vec2 {
        if self.displacement.x.abs() < self.displacement.y.abs() {
            Vec2::new(self.displacement.x, 0.0)
        } else if self.displacement.y.abs() < self.displacement.x.abs() {
            Vec2::new(0.0, self.displacement.y)
        } else {
            Vec2::ZERO
        }
    }
}

/// Metadata of each collision entity.
#[derive(Default, Clone, Copy)]
pub struct CollisionData<ID: Copy + Eq> {
    /// True if the entity is solid, i.e. can stop other entities.
    ///
    /// For example, a wall or player character might be solid, while a section of the ground that applies an effect on the player character when they walk over it (colliding with it) might not be.
    pub solid: bool,
    /// True if the entity is fixed, i.e. does _not_ participate in restitution.
    ///
    /// Pushable entities are _not_ fixed, while entities that shouldn't be pushable, such as walls or moving platforms, are.
    pub fixed: bool,
    pub id: ID,
}

/// A collision logic for axis-aligned bounding boxes.
pub struct AabbCollision<ID: Copy + Eq> {
    /// A vector of the centers of the bounding box.
    pub centers: Vec<Vec2>,
    /// A vector of half the width and half the height of the bounding box.
    pub half_sizes: Vec<Vec2>,
    /// A vector of the velocity of the entities.
    pub velocities: Vec<Vec2>,
    /// A vector of entity metadata.
    pub metadata: Vec<CollisionData<ID>>,
    /// A vector of all entities that are touching.
    ///
    /// Indices do _not_ run parallel with those in the above vectors.
    pub contacts: Vec<Contact>,
}

impl<ID: Copy + Eq> AabbCollision<ID> {
    pub fn new() -> Self {
        Self {
            centers: Vec::new(),
            half_sizes: Vec::new(),
            velocities: Vec::new(),
            metadata: Vec::new(),
            contacts: Vec::new(),
        }
    }

    /// Checks collisions every frame and handles restitution.
    ///
    /// Code is somewhat stolen from the CS181G engine3d collision starter code by Prof Osborn. Shoutouts
    pub fn update(&mut self) {
        self.contacts.clear();

        // check contacts
        for i in 0..self.centers.len() {
            for j in i + 1..self.centers.len() {
                if intersects(
                    self.centers[i],
                    self.half_sizes[i],
                    self.centers[j],
                    self.half_sizes[j],
                ) {
                    // if i is fixed and other is unfixed, swap places
                    let mut i = i;
                    let mut j = j;

                    if self.metadata[i].fixed && !self.metadata[j].fixed {
                        std::mem::swap(&mut i, &mut j);
                    }

                    let displacement = if self.metadata[i].solid
                        && self.metadata[j].solid
                        && !self.metadata[i].fixed
                    {
                        find_displacement(
                            self.centers[i],
                            self.half_sizes[i],
                            self.centers[j],
                            self.half_sizes[j],
                        )
                    } else {
                        Vec2::ZERO
                    };
                    let contact = Contact { i, j, displacement };
                    self.contacts.push(contact);
                }
            }
        }

        self.contacts.sort_unstable_by(|a, b| {
            b.displacement
                .length_squared()
                .partial_cmp(&a.displacement.length_squared())
                .unwrap()
        });

        for contact in self.contacts.iter_mut() {
            let i = contact.i;
            let j = contact.j;
            if !self.metadata[i].solid || !self.metadata[j].solid || self.metadata[i].fixed {
                continue;
            }
            if intersects(
                self.centers[i],
                self.half_sizes[i],
                self.centers[j],
                self.half_sizes[j],
            ) {
                contact.displacement = find_displacement(
                    self.centers[i],
                    self.half_sizes[i],
                    self.centers[j],
                    self.half_sizes[j],
                );
                let disp = contact.get_restitution();
                let speed_ratio = if !self.metadata[j].fixed {
                    get_speed_ratio(self.velocities[i], self.velocities[j])
                } else {
                    Vec2::ONE
                };
                self.centers[i] += disp * speed_ratio;
                self.centers[j] -= disp * (Vec2::ONE - speed_ratio);
            }
        }
    }

    /// Adds a collision entity to the logic, taking two Vec2s with the center and half the dimensions of the AABB. `solid` represents if the entity can stop other entities, and `fixed` represents if it can participate in restitution, i.e. be moved by the collision logic or not. See [CollisionData] for further explanation.
    pub fn add_collision_entity(
        &mut self,
        center: Vec2,
        half_size: Vec2,
        vel: Vec2,
        solid: bool,
        fixed: bool,
        id: ID,
    ) {
        self.centers.push(center);
        self.half_sizes.push(half_size);
        self.velocities.push(vel);
        self.metadata.push(CollisionData { solid, fixed, id });
    }

    /// Adds a collision entity to the logic, taking the x and y positions, width, and height of the AABB as well as its velocity and some metadata. See [add_collision_entity][AabbCollision::add_collision_entity] for details on what the other fields represent.
    pub fn add_entity_as_xywh(
        &mut self,
        pos: Vec2,
        size: Vec2,
        vel: Vec2,
        solid: bool,
        fixed: bool,
        id: ID,
    ) {
        let hs = size / 2.0;
        let center = pos + hs;
        self.add_collision_entity(center, hs, vel, solid, fixed, id);
    }

    /// Returns unit vector of normal of displacement for the entity of the given ID in the given contact. I.e., if a contact is moved in a positive x direction after restitution _because of_ the other entity involved in collision, `sides_touched` will return `Vec2::new(1.0, 0.0)`. Panics if the given EntityID isn't that of either entity in the contact.
    pub fn sides_touched(&self, i: usize, j: usize) -> Vec2 {
        let should_swap = self.metadata[i].fixed && !self.metadata[j].fixed;
        let mut i = i;
        let mut j = j;
        if should_swap {
            std::mem::swap(&mut i, &mut j);
        }
        let displacement = find_displacement(
            self.centers[i],
            self.half_sizes[i],
            self.centers[j],
            self.half_sizes[j],
        );
        if displacement.x.abs() < displacement.y.abs() {
            Vec2::new(1.0, 0.0)
        } else if displacement.x.abs() > displacement.y.abs() {
            Vec2::new(0.0, 1.0)
        } else {
            Vec2::ZERO
        }
    }

    /// Clears vecs from last frame
    pub fn clear(&mut self) {
        self.centers.clear();
        self.half_sizes.clear();
        self.velocities.clear();
    }

    pub fn get_ids(&self, contact: &Contact) -> (ID, ID) {
        (self.metadata[contact.i].id, self.metadata[contact.j].id)
    }
}

#[derive(Clone)]
pub struct AabbColData<ID: Copy + Eq> {
    pub center: Vec2,
    pub half_size: Vec2,
    pub vel: Vec2,
    pub fixed: bool,
    pub solid: bool,
    pub id: ID,
}

impl<ID: Copy + Eq> Logic for AabbCollision<ID> {
    type Event = CollisionEvent;
    type Reaction = CollisionReaction<ID>;

    type Ident = usize;
    type IdentData = AabbColData<ID>;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            CollisionReaction::SetPos(idx, pos) => {
                let idx = *idx;
                self.centers[idx] = *pos + self.half_sizes[idx];
            }
            CollisionReaction::SetSize(idx, size) => {
                let idx = *idx;
                self.half_sizes[idx] = *size / 2.0;
            }
            CollisionReaction::SetVel(idx, vel) => {
                let idx = *idx;
                self.velocities[idx] = *vel;
            }
            CollisionReaction::SetMetadata(idx, solid, fixed) => {
                let idx = *idx;
                self.metadata[idx].solid = *solid;
                self.metadata[idx].fixed = *fixed;
            }
            CollisionReaction::RemoveBody(idx) => {
                // this will likely mess up any contacts processing....
                let idx = *idx;
                self.centers.remove(idx);
                self.half_sizes.remove(idx);
                self.metadata.remove(idx);
                self.velocities.remove(idx);
            }
            CollisionReaction::AddBody {
                pos,
                size,
                vel,
                solid,
                fixed,
                id,
            } => {
                self.add_entity_as_xywh(*pos, *size, *vel, *solid, *fixed, *id);
            }
        }
    }

    fn get_synthesis(&self, ident: Self::Ident) -> Self::IdentData {
        AabbColData {
            center: self.centers[ident],
            half_size: self.half_sizes[ident],
            vel: self.velocities[ident],
            fixed: self.metadata[ident].fixed,
            solid: self.metadata[ident].solid,
            id: self.metadata[ident].id,
        }
    }

    fn update_synthesis(&mut self, ident: Self::Ident, data: Self::IdentData) {
        self.centers[ident] = data.center;
        self.half_sizes[ident] = data.half_size;
        self.velocities[ident] = data.vel;
        self.metadata[ident].fixed = data.fixed;
        self.metadata[ident].solid = data.solid;
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CollisionReaction<ID> {
    /// sets the position, _not_ the center
    SetPos(usize, Vec2),
    /// sets the size, _not_ the half size
    SetSize(usize, Vec2),
    SetVel(usize, Vec2),
    SetMetadata(usize, bool, bool), // solid, fixed
    /// NOTE that using this predicate will likely break anything involving contacts until this logic is updated
    RemoveBody(usize),
    AddBody {
        pos: Vec2,
        size: Vec2,
        vel: Vec2,
        solid: bool,
        fixed: bool,
        id: ID,
    },
}

impl<ID> Reaction for CollisionReaction<ID> {}

pub type CollisionEvent = (usize, usize);

impl Event for CollisionEvent {
    type EventType = CollisionEventType;

    fn get_type(&self) -> &Self::EventType {
        &CollisionEventType::Touching
    }
}

pub enum CollisionEventType {
    Touching,
}

impl EventType for CollisionEventType {}

type QueryIdent<ID> = (
    <AabbCollision<ID> as Logic>::Ident,
    <AabbCollision<ID> as Logic>::IdentData,
);

impl<ID: Copy + Eq> OutputTable<QueryIdent<ID>> for AabbCollision<ID> {
    fn get_table(&self) -> Vec<QueryIdent<ID>> {
        (0..self.centers.len())
            .map(|idx| (idx, self.get_synthesis(idx)))
            .collect()
    }
}

impl<ID: Copy + Eq> OutputTable<(usize, usize)> for AabbCollision<ID> {
    fn get_table(&self) -> Vec<(usize, usize)> {
        self.contacts
            .iter()
            .map(|contact| (contact.i, contact.j))
            .collect()
    }
}

// inlined for performance
#[inline(always)]
fn intersects(center_i: Vec2, half_size_i: Vec2, center_j: Vec2, half_size_j: Vec2) -> bool {
    (center_i.x - center_j.x).abs() <= half_size_i.x + half_size_j.x
        && (center_i.y - center_j.y).abs() <= half_size_i.y + half_size_j.y
}

#[inline(always)]
fn find_displacement(center_i: Vec2, half_size_i: Vec2, center_j: Vec2, half_size_j: Vec2) -> Vec2 {
    let displ_abs = Vec2::new(
        half_size_i.x + half_size_j.x - (center_i.x - center_j.x).abs(),
        half_size_i.y + half_size_j.y - (center_i.y - center_j.y).abs(),
    );
    let side_x = if center_i.x - center_j.x < 0.0 {
        -1.0
    } else {
        1.0
    };
    let side_y = if center_i.y - center_j.y < 0.0 {
        -1.0
    } else {
        1.0
    };

    Vec2::new(side_x * displ_abs.x, side_y * displ_abs.y)
}

/// Calculates the speed ratio of the two entities, i.e. the amount of restitution an entity should be responsible for.
///
/// Assumes that the entity at index `i` is unfixed. When the entity at index `j` is fixed, entity `i` will be responsible for all of the restitution. Otherwise, it is responsible for an amount of restitution proportional to the entities' velocity.
///
/// I think this is mostly ripped from this tutorial: https://gamedevelopment.tutsplus.com/series/basic-2d-platformer-physics--cms-998
fn get_speed_ratio(vel_i: Vec2, vel_j: Vec2) -> Vec2 {
    let vxi = vel_i.x.abs();
    let vyi = vel_i.y.abs();
    let vxj = vel_j.x.abs();
    let vyj = vel_j.y.abs();

    let speed_sum = Vec2::new(vxi + vxj, vyi + vyj);
    let mut speed_ratio = if speed_sum.x == 0.0 && speed_sum.y == 0.0 {
        Vec2::new(0.5, 0.5)
    } else if speed_sum.x == 0.0 {
        Vec2::new(0.5, vyi / speed_sum.y)
    } else if speed_sum.y == 0.0 {
        Vec2::new(vxi / speed_sum.x, 0.5)
    } else {
        Vec2::new(vxi / speed_sum.x, vyi / speed_sum.y)
    };

    if speed_ratio.x == 0.0 {
        speed_ratio.x = 1.0;
    }
    if speed_ratio.y == 0.0 {
        speed_ratio.y = 1.0;
    }

    speed_ratio
}
