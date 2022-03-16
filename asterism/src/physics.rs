//! # Physics logics
//!
//! Physics logics communicate that physical laws govern the movement of some in-game entities. They update and honor objects' physical properties like position, velocity, density, etc., according to physical laws integrated over time.

use crate::{Logic, Reaction};
use macroquad::math::Vec2;

/// A physics logic using 2d points.
pub struct PointPhysics {
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub accelerations: Vec<Vec2>,
    pub events: Vec<Vec<Option<PhysicsEvent>>>,
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
        let events = vec![None; self.positions.len()];
        self.events.push(events);
        for events in self.events.iter_mut() {
            events.push(None);
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

    pub fn iter(&self) -> PointPhysicsIter {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> PointPhysicsIterMut {
        self.into_iter()
    }
}

impl Logic for PointPhysics {
    type Reaction = PhysicsReaction;

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
            PhysicsReaction::AddBody { pos, vel, acc } => {
                self.add_physics_entity(*pos, *vel, *acc);
            }
        }
    }
}

pub struct PointPhysData<'logic> {
    pub pos: &'logic Vec2,
    pub vel: &'logic Vec2,
    pub acc: &'logic Vec2,
    pub events: &'logic [Option<PhysicsEvent>],
}

pub struct PointPhysDataMut<'logic> {
    pub pos: &'logic mut Vec2,
    pub vel: &'logic mut Vec2,
    pub acc: &'logic mut Vec2,
    pub events: &'logic mut [Option<PhysicsEvent>],
}

pub struct PointPhysicsIter<'logic> {
    physics: &'logic PointPhysics,
    index: usize,
}

impl<'logic> IntoIterator for &'logic PointPhysics {
    type Item = PointPhysData<'logic>;
    type IntoIter = PointPhysicsIter<'logic>;

    fn into_iter(self) -> Self::IntoIter {
        PointPhysicsIter {
            physics: self,
            index: 0,
        }
    }
}

impl<'logic> Iterator for PointPhysicsIter<'logic> {
    type Item = PointPhysData<'logic>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.index;
        self.index += 1;
        if i >= self.physics.positions.len() {
            return None;
        }
        Some(PointPhysData {
            pos: &self.physics.positions[i],
            vel: &self.physics.velocities[i],
            acc: &self.physics.accelerations[i],
            events: &self.physics.events[i],
        })
    }
}

pub struct PointPhysicsIterMut<'logic> {
    physics: &'logic mut PointPhysics,
    index: usize,
}

impl<'logic> IntoIterator for &'logic mut PointPhysics {
    type Item = PointPhysDataMut<'logic>;
    type IntoIter = PointPhysicsIterMut<'logic>;

    fn into_iter(self) -> Self::IntoIter {
        PointPhysicsIterMut {
            physics: self,
            index: 0,
        }
    }
}

impl<'logic> Iterator for PointPhysicsIterMut<'logic> {
    type Item = PointPhysDataMut<'logic>;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.index;
        self.index += 1;
        if i >= self.physics.positions.len() {
            return None;
        }
        let pos = self.physics.positions.as_mut_ptr();
        let vel = self.physics.velocities.as_mut_ptr();
        let acc = self.physics.accelerations.as_mut_ptr();
        let events = self.physics.events.as_mut_ptr();

        // safety: mutability doesn't overlap when calling next(). *pos.add(i) is a *mut Vec2 and taking a reference to it is a &mut Vec2. adapted from: https://stackoverflow.com/questions/63437935/in-rust-how-do-i-create-a-mutable-iterator
        unsafe {
            Some(PointPhysDataMut {
                pos: &mut *pos.add(i),
                vel: &mut *vel.add(i),
                acc: &mut *acc.add(i),
                events: &mut *events.add(i),
            })
        }
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PhysicsEvent {
    VelChange,
    PosChange,
}
