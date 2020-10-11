use std::ops::{Add, AddAssign, Mul};

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

    pub fn update(&mut self) {
        for (pos, (vel, acc)) in self.positions.iter_mut().zip(self.velocities.iter_mut().zip(self.accelerations.iter())) {
            *vel += *acc;
            *pos += *vel;
        }
    }

    pub fn add_physics_entity(&mut self, i: usize, pos: Vec2, vel: Vec2, acc: Vec2) {
        self.positions[i] = pos;
        self.velocities[i] = vel;
        self.accelerations[i] = acc;
    }

}


