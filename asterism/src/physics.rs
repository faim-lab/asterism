//! # Physics logics
//!
//! Physics logics communicate that physical laws govern the movement of some in-game entities. They update and honor objects' physical properties like position, velocity, density, etc., according to physical laws integrated over time.

use crate::{tables::QueryTable, Event, EventType, Logic, Reaction};
use macroquad::math::Vec2;

/// A physics logic using 2d points.
pub struct PointPhysics {
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub accelerations: Vec<Vec2>,
}

pub struct PointPhysData {
    pub pos: Vec2,
    pub vel: Vec2,
    pub acc: Vec2,
}

impl Logic for PointPhysics {
    type Reaction = PhysicsReaction;
    type Event = PhysicsEvent;

    type Ident = usize;
    type IdentData = PointPhysData;

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
            }
            PhysicsReaction::AddBody { pos, vel, acc } => {
                self.add_physics_entity(*pos, *vel, *acc);
            }
        }
    }

    fn get_synthesis(&self, ident: Self::Ident) -> Self::IdentData {
        PointPhysData {
            pos: self.positions[ident],
            vel: self.velocities[ident],
            acc: self.accelerations[ident],
        }
    }

    fn update_synthesis(&mut self, ident: Self::Ident, data: Self::IdentData) {
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
    }

    /// Clears vecs from last frame
    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.accelerations.clear();
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PhysicsReaction {
    SetPos(usize, Vec2),
    SetVel(usize, Vec2),
    SetAcc(usize, Vec2),
    RemoveBody(usize),
    AddBody { pos: Vec2, vel: Vec2, acc: Vec2 },
}
impl Reaction for PhysicsReaction {}

#[derive(PartialEq, Eq, Debug)]
pub struct PhysicsEvent {
    ent: usize,
    event_type: PhysicsEventType,
}

impl Event for PhysicsEvent {
    type EventType = PhysicsEventType;
    fn get_type(&self) -> &Self::EventType {
        &self.event_type
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PhysicsEventType {
    VelChange,
    PosChange,
}
impl EventType for PhysicsEventType {}

type QueryOver = (
    <PointPhysics as Logic>::Ident,
    <PointPhysics as Logic>::IdentData,
);
impl QueryTable<QueryOver> for PointPhysics {
    fn check_predicate(&self, predicate: impl Fn(&QueryOver) -> bool) -> Vec<bool> {
        (0..self.positions.len())
            .map(|i| {
                let query_over = (i, self.get_synthesis(i));
                predicate(&query_over)
            })
            .collect()
    }
}

type QueryEvent = <PointPhysics as Logic>::Event;

impl QueryTable<QueryEvent> for PointPhysics {
    fn check_predicate(&self, predicate: impl Fn(&QueryEvent) -> bool) -> Vec<bool> {
        let mut events = Vec::new();
        self.accelerations.iter().enumerate().for_each(|(i, acc)| {
            // velocity changes if acceleration != 0.0
            if *acc != Vec2::ZERO {
                let event = PhysicsEvent {
                    ent: i,
                    event_type: PhysicsEventType::VelChange,
                };
                events.push(predicate(&event));
            }
        });
        self.velocities.iter().enumerate().for_each(|(i, vel)| {
            // position changes if velocity != 0.0
            if *vel != Vec2::ZERO {
                let event = PhysicsEvent {
                    ent: i,
                    event_type: PhysicsEventType::PosChange,
                };
                events.push(predicate(&event));
            }
        });
        events
    }
}
