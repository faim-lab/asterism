use ultraviolet::{Vec2, Vec3, geometry::Aabb};

pub struct AabbCollision<ID: Copy + Eq> {
    pub bodies: Vec<Aabb>,
    pub velocities: Vec<Vec2>,
    pub metadata: Vec<CollisionData<ID>>,
    pub contacts: Vec<(usize, usize)>,
}

#[derive(Default, Clone, Copy)]
pub struct CollisionData<ID: Copy + Eq> {
    pub solid: bool, // true = participates in restitution, false = no
    pub fixed: bool, // collision system cannot move it
    pub id: ID
}

impl<ID: Copy + Eq> AabbCollision<ID> {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            metadata: Vec::new(),
            velocities: Vec::new(),
            contacts: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.contacts.clear();
        for (i, body) in self.bodies.iter().enumerate() {
            for (j, body2) in self.bodies[i + 1..].iter().enumerate() {
                if body.intersects(body2) {
                    self.contacts.push((i, j + i + 1));
                }
            }
        }

        for (i, j) in self.contacts.iter() {
            let CollisionData { solid: i_solid, fixed: i_fixed, .. } =
                self.metadata[*i];
            let CollisionData { solid: j_solid, fixed: j_fixed, .. } =
                self.metadata[*j];

            if !(i_solid && j_solid) || (i_fixed && j_fixed) {
                continue;
            }

            if !i_fixed && !j_fixed {
                let Vec2 { x: vel_i_x, y: vel_i_y } = self.velocities[*i];
                let Vec2 { x: vel_j_x, y: vel_j_y } = self.velocities[*j];
                let Aabb { min: Vec3 { x: min_i_x, y: min_i_y, .. },
                    max: Vec3 { x: max_i_x, y: max_i_y, ..} } = self.bodies[*i];
                let Aabb { min: Vec3 { x: min_j_x, y: min_j_y, .. },
                    max: Vec3 { x: max_j_x, y: max_j_y, ..} } = self.bodies[*j];

                let ( i_displace, j_displace ) = {
                    let vel_i_x = vel_i_x / (vel_i_x.abs() + vel_j_x.abs());
                    let vel_i_y = vel_i_y / (vel_i_y.abs() + vel_j_y.abs());
                    let vel_j_x = vel_j_x / (vel_i_x.abs() + vel_j_x.abs());
                    let vel_j_y = vel_j_y / (vel_i_y.abs() + vel_j_y.abs());

                    let displacement_x = Self::get_displacement(min_i_x, max_i_x, min_j_x, max_j_x);
                    let displacement_y = Self::get_displacement(min_i_y, max_i_y, min_j_y, max_j_y);

                    ( Vec3::new(displacement_x * vel_i_x, displacement_y * vel_i_y, 0.0),
                        Vec3::new(displacement_x * vel_j_x, displacement_y * vel_j_y, 0.0) )
                };

                self.bodies[*i].min += i_displace;
                self.bodies[*i].max += i_displace;
                self.bodies[*j].min += j_displace;
                self.bodies[*j].max += j_displace;
            } else {
                let i_swap = if !j_fixed { j } else { i };
                let j_swap = if !j_fixed { i } else { j };
                let Aabb { min: Vec3 { x: min_i_x, y: min_i_y, .. },
                    max: Vec3 { x: max_i_x, y: max_i_y, ..} } = self.bodies[*i_swap];
                let Aabb { min: Vec3 { x: min_j_x, y: min_j_y, .. },
                    max: Vec3 { x: max_j_x, y: max_j_y, ..} } = self.bodies[*j_swap];
                let displace = {
                    let displacement_x = Self::get_displacement(min_i_x, max_i_x, min_j_x, max_j_x);
                    let displacement_y = Self::get_displacement(min_i_y, max_i_y, min_j_y, max_j_y);

                    if displacement_x == displacement_y {
                        Vec3::new(displacement_x, displacement_y, 0.0)
                    } else if displacement_x < displacement_y {
                        if min_i_x < min_j_x {
                            Vec3::new(-displacement_x, 0.0, 0.0)
                        } else {
                            Vec3::new(displacement_x, 0.0, 0.0)
                        }
                    } else {
                        if min_i_y < min_j_y {
                            Vec3::new(0.0, -displacement_y, 0.0)
                        } else {
                            Vec3::new(0.0, displacement_y, 0.0)
                        }
                    }
                };

                self.bodies[*i_swap].min += displace;
                self.bodies[*i_swap].max += displace;
            }
        }
    }

    pub fn add_collision_entity(&mut self, x: f32, y: f32, w: f32, h: f32, vel: Vec2, solid: bool, fixed: bool, id: ID) {
            self.bodies.push(
                Aabb::new(
                    Vec3::new(x, y, 0.0),
                    Vec3::new(x + w, y + h, 0.0)));
            self.velocities.push(vel);
            self.metadata.push(CollisionData { solid: solid, fixed: fixed, id: id });
    }

    fn get_displacement(min_i: f32, max_i: f32, min_j: f32, max_j: f32)
        -> f32 {
            if max_i - min_j < max_j - min_i {
                max_i - min_j
            } else {
                max_j - min_i
            }
    }
}


