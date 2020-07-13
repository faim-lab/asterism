use ultraviolet::Vec2;

pub struct PointPhysics {
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub accelerations: Vec<Vec2>,
}

impl PointPhysics {
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
}


