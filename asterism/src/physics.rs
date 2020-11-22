//! # Physics logics
//!
//! Physics logics communicate that physical laws govern the movement of some in-game entities.
//! They update and honor objects' physical properties like position, velocity, density, etc.,
//! according to physical laws integrated over time.

use std::ops::{Add, AddAssign, Mul};

/// A physics logic for physics with 2d points.
pub struct PointPhysics<Vec2: Add + AddAssign + Copy + Mul<Output = Vec2>> {
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub accelerations: Vec<Vec2>,
}

impl<Vec2: Add + AddAssign + Copy + Mul<Output = Vec2>> PointPhysics<Vec2> {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
        }
    }

    /// Update the velocities of entities based on acceleration, then update entities' positions
    /// based on updated velocities.
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
}
