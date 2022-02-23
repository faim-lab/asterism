//! # Physics logics
//!
//! Physics logics communicate that physical laws govern the movement of some in-game entities. They update and honor objects' physical properties like position, velocity, density, etc., according to physical laws integrated over time.

use crate::{tables::OutputTable, Logic, Reaction};
use macroquad::math::Vec2;

/// A physics logic using 2d points.
pub struct PointPhysics {
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub accelerations: Vec<Vec2>,
    pub events: Vec<Vec<Option<PhysicsEvent>>>,
}

// doesn't include events bc they happen one time instead of being continuous across states?
#[derive(Clone, Copy)]
pub struct PointPhysData {
    pub pos: Vec2,
    pub vel: Vec2,
    pub acc: Vec2,
}

impl Logic for PointPhysics {
    type Reaction = PhysicsReaction;

    type Ident = usize;
    type IdentData = PointPhysData;
    // type IdentEventData = Vec<Events>

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            PhysicsReaction::SetPos(idx, pos) => {
                self.positions[*idx] = *pos;
            }
            PhysicsReaction::SetVel(idx, vel) => {
                self.velocities[*idx] = *vel;
            }
            PhysicsReaction::SetAcc(idx, acc) => {
                self.accelerations[*idx] = *acc;
            }
            PhysicsReaction::RemoveBody(idx) => {
                self.positions.remove(*idx);
                self.velocities.remove(*idx);
                self.accelerations.remove(*idx);
                self.events.remove(*idx);
                for events in self.events.iter_mut() {
                    events.remove(*idx);
                }
            }
            PhysicsReaction::AddBody(data) => {
                self.add_physics_entity(data.pos, data.vel, data.acc);
            }
        }
    }

    fn get_ident_data(&self, ident: Self::Ident) -> Self::IdentData {
        PointPhysData {
            pos: self.positions[ident],
            vel: self.velocities[ident],
            acc: self.accelerations[ident],
        }
    }

    fn update_ident_data(&mut self, ident: Self::Ident, data: Self::IdentData) {
        self.positions[ident] = data.pos;
        self.velocities[ident] = data.vel;
        self.accelerations[ident] = data.acc;
    }
}

impl PointPhysics {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            events: Vec::new(),
        }
    }
    /// Update the physics logic: changes the velocities of entities based on acceleration, then changes entities' positions based on updated velocities.
    pub fn update(&mut self) {
        for (pos, (vel, acc)) in self
            .positions
            .iter_mut()
            .zip(self.velocities.iter_mut().zip(self.accelerations.iter()))
        {
            *vel += *acc;
            *pos += *vel;
        }
    }

    /// Adds a physics entity to the logic with the given position, velocity, and acceleration.
    pub fn add_physics_entity(&mut self, pos: Vec2, vel: Vec2, acc: Vec2) {
        self.positions.push(pos);
        self.velocities.push(vel);
        self.accelerations.push(acc);

        let mut events = Vec::new();
        events.resize(self.positions.len(), None);
        self.events.push(events);
        for ev in self.events.iter_mut() {
            ev.push(None);
        }
    }

    /// Clears vecs from last frame
    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.accelerations.clear();
        for events in self.events.iter_mut() {
            events.fill(None);
        }
    }
}

#[derive(Clone, Copy)]
pub enum PhysicsReaction {
    SetPos(usize, Vec2),
    SetVel(usize, Vec2),
    SetAcc(usize, Vec2),
    RemoveBody(usize),
    AddBody(PointPhysData),
}
impl Reaction for PhysicsReaction {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PhysicsEvent {
    VelChange,
    PosChange,
}

type QueryIdent = (
    <PointPhysics as Logic>::Ident,
    <PointPhysics as Logic>::IdentData,
);

impl OutputTable<QueryIdent> for PointPhysics {
    fn get_table(&self) -> Vec<QueryIdent> {
        (0..self.positions.len())
            .map(|idx| (idx, self.get_ident_data(idx)))
            .collect()
    }
}
