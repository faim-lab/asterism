//! # Physics logics
//!
//! Physics logics communicate that physical laws govern the movement of some in-game entities. They update and honor objects' physical properties like position, velocity, density, etc., according to physical laws integrated over time.

use crate::{Event, Logic, Reaction};
use glam::Vec2;

/// A physics logic using 2d points.
pub struct PointPhysics {
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub accelerations: Vec<Vec2>,
}

impl Logic for PointPhysics {
    type Reaction = PhysicsReaction;
    type Event = PhysicsEvent;
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
    SetVel(usize, Vec2),
    SetAcc(usize, Vec2),
}
#[derive(PartialEq, Eq, Debug)]
pub enum PhysicsEvent {}

impl Reaction for PhysicsReaction {}

impl Event for PhysicsEvent {}
