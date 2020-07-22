use ultraviolet::{Vec2, Vec3, geometry::Aabb};

pub struct AabbCollision<ID: Copy + Eq> {
    pub bodies: Vec<Aabb>,
    pub velocities: Vec<Vec2>,
    pub metadata: Vec<CollisionData<ID>>,
    pub contacts: Vec<(usize, usize)>,
    displacements: Vec<Option<Vec3>>,
    // in the form of [top, bottom, left, right, some corner], where true means it collides there
    pub sides_touched: Vec<[bool; 5]>,
}

#[derive(Default, Clone, Copy)]
pub struct CollisionData<ID: Copy + Eq> {
    solid: bool, // true = participates in restitution, false = no
    fixed: bool, // collision system cannot move it
    pub id: ID,
}

impl<ID: Copy + Eq> AabbCollision<ID> {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            velocities: Vec::new(),
            metadata: Vec::new(),
            contacts: Vec::new(),
            displacements: Vec::new(),
            sides_touched: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.contacts.clear();
        self.displacements.clear();
        self.sides_touched.clear();
        self.displacements.resize_with(self.bodies.len(), Default::default);
        self.sides_touched.resize_with(self.bodies.len(), Default::default);

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

            if !(i_solid && j_solid) || i_fixed && j_fixed {
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
                let i_swap = if !j_fixed {j} else {i};
                let j_swap = if !j_fixed {i} else {j};

                let Aabb { min: Vec3 { x: min_i_x, y: min_i_y, .. },
                max: Vec3 { x: max_i_x, y: max_i_y, ..} } = self.bodies[*i_swap];
                let Aabb { min: Vec3 { x: min_j_x, y: min_j_y, .. },
                max: Vec3 { x: max_j_x, y: max_j_y, ..} } = self.bodies[*j_swap];

                let half_isize_x = (max_i_x - min_i_x) / 2.0;
                let half_isize_y = (max_i_y - min_i_y) / 2.0;
                let half_jsize_x = (max_j_x - min_j_x) / 2.0;
                let half_jsize_y = (max_j_y - min_j_y) / 2.0;

                let i_center = Self::find_center(self.bodies[*i_swap]);
                let j_center = Self::find_center(self.bodies[*j_swap]);

                let overlapped_before_x = {
                    let old_x_center = i_center.x - self.velocities[*i_swap].x;
                    (old_x_center - j_center.x).abs() < half_isize_x + half_jsize_x
                };

                let overlapped_before_y = {
                    let old_y_center = i_center.y - self.velocities[*i_swap].y;
                    (old_y_center - j_center.y).abs() < half_isize_y + half_jsize_y
                };

                let mut new_sides:[bool; 5] = [false; 5];  // sides touched this iteration

                if overlapped_before_x && !overlapped_before_y && self.velocities[*i_swap].y != 0.0 {
                    if self.velocities[*i_swap].y < 0.0 {
                        new_sides[0] = true;  // top side touched
                    } else {
                        new_sides[1] = true;  // bottom side touched
                    }
                }

                if !overlapped_before_x && overlapped_before_y && self.velocities[*i_swap].x != 0.0 {
                    if self.velocities[*i_swap].x < 0.0 {
                        new_sides[2] = true;  // left side touched
                    } else {
                        new_sides[3] = true;  // right side touched
                    }
                }

                if !overlapped_before_x && !overlapped_before_y 
                && self.velocities[*i_swap].x != 0.0 && self.velocities[*i_swap].y != 0.0 {
                    new_sides[4] = true; // touched diagonally :^) not necessarily at corner
                }
            
                let displace = {
                    // overlapped vertically
                    if new_sides[0] || new_sides[1] {
                        if new_sides[0] {
                            Vec3::new(0.0, max_j_y - min_i_y, 0.0)
                        } else {
                            Vec3::new(0.0, min_j_y - max_i_y, 0.0)
                        }
                    // overlapped horizontally
                    } else if new_sides[2] || new_sides[3] {
                        if new_sides[2] {
                            Vec3::new(max_j_x - min_i_x, 0.0, 0.0)
                        } else {
                            Vec3::new(min_j_x - max_i_x, 0.0, 0.0)
                        }
                    // overlapped diagonally
                    } else if new_sides[4] {  // if new_sides[4] 
                        let mut new_x = {
                            if self.velocities[*i_swap].x < 0.0 {
                                max_j_x - min_i_x
                            } else if self.velocities[*i_swap].x > 0.0 {
                                min_j_x - max_i_x
                            } else {
                                0.0
                            }
                        };
                        let mut new_y = {
                            if self.velocities[*i_swap].y < 0.0 {
                                max_j_y - min_i_y
                            } else if self.velocities[*i_swap].y > 0.0 {
                                min_j_y - max_i_y
                            } else {
                                0.0
                            }
                        };
                        if new_x.abs() > new_y.abs() {
                            new_y =  {
                                if self.velocities[*i_swap].y > 0.0 { -1.0 * new_x.abs() } else { new_x.abs() }
                            };
                        } else if new_y.abs() > new_x.abs() {
                            new_x =  {
                                if self.velocities[*i_swap].x > 0.0 { -1.0 * new_y.abs() } else { new_y.abs() }
                            };
                        }
                        Vec3::new(new_x, new_y, 0.0)
                        
                    } else {  // if everything is false because vx and vy == 0 but still overlapping...
                        Vec3::new(0.0, 0.0, 0.0)
                    }
                };

                if new_sides[0] { self.sides_touched[*i_swap][0] = true; }
                else if new_sides[1] { self.sides_touched[*i_swap][1] = true; }
                else if new_sides[2] { self.sides_touched[*i_swap][2] = true; }
                else if new_sides[3] { self.sides_touched[*i_swap][3] = true; }
                else if new_sides[4] { self.sides_touched[*i_swap][4] = true; }
                
                if let Some(new_displace) = &mut self.displacements[*i_swap] {
                    // already touching at least one side and no corners
                    if {self.sides_touched[*i_swap][0] || self.sides_touched[*i_swap][1]
                    || self.sides_touched[*i_swap][2] || self.sides_touched[*i_swap][3]}
                    && !self.sides_touched[*i_swap][4] {
                        // touching at two different sides
                        if {self.sides_touched[*i_swap][0] || self.sides_touched[*i_swap][1]}
                        && {self.sides_touched[*i_swap][2] || self.sides_touched[*i_swap][3]} {
                            if displace.x.abs() > new_displace.x.abs() {
                                new_displace.x = displace.x;
                            } 
                            if displace.y.abs() > new_displace.y.abs() {
                                new_displace.y = displace.y;
                            }
                        // touching one side
                        } else {
                            if self.sides_touched[*i_swap][0] || self.sides_touched[*i_swap][1] {
                                if displace.y.abs() > new_displace.y.abs() {
                                    new_displace.y = displace.y;
                                }
                            } else {
                                if displace.x.abs() > new_displace.x.abs() {
                                    new_displace.x = displace.x;
                                }
                            }
                        }
                    // already touching a corner
                    } else if self.sides_touched[*i_swap][4] {
                        if self.sides_touched[*i_swap][0] || self.sides_touched[*i_swap][1] {
                            if displace.y.abs() > new_displace.y.abs() {
                                new_displace.y = displace.y;
                            }
                            new_displace.x = 0.0;
                        } else if self.sides_touched[*i_swap][2] || self.sides_touched[*i_swap][3] {
                            if displace.x.abs() > new_displace.x.abs() {
                                new_displace.x = displace.x;
                            }
                            new_displace.y = 0.0;
                        } else {  // touching corner(s) only or something
                            if displace.x.abs() > new_displace.x.abs() {
                                new_displace.x = displace.x;
                            }
                            if displace.y.abs() > new_displace.y.abs() {
                                new_displace.y = displace.y;
                            }
                        }
                    }
                } else {
                    self.displacements[*i_swap] = Some(displace);
                }
            }
        }
        
        for i in 0..self.displacements.len() {
            match self.displacements[i] {
                None => {
                    continue;
                }
                _ => {
                    self.bodies[i].min += self.displacements[i].unwrap();
                    self.bodies[i].max += self.displacements[i].unwrap();
                }
            }
        }
    }

    fn find_center(body_data: Aabb) -> Vec2 {
        Vec2::new(
            (body_data.min.x + body_data.max.x) / 2.0,
            (body_data.min.y + body_data.max.y) / 2.0
        )
    }

    fn get_displacement(min_i: f32, max_i: f32, min_j: f32, max_j: f32)
        -> f32 {
            if max_i - min_j < max_j - min_i {
                max_i - min_j
            } else {
                max_j - min_i
            }
    }

    pub fn add_collision_entity(&mut self, x: f32, y: f32, w: f32, h: f32, vel: Vec2, solid: bool, fixed: bool, id: ID) {
            self.bodies.push(
                Aabb::new(
                    Vec3::new(x, y, 0.0),
                    Vec3::new(x + w, y + h, 0.0)));
            self.velocities.push(vel);
            self.metadata.push(CollisionData { solid: solid, fixed: fixed, id: id });
            self.displacements.push(None);
    }
}
