use ultraviolet::{Vec2, Vec3, geometry::Aabb};

// - see chat w/ julie re: structures?
//   - or maybe make a fn that checks which side the contacts happening on..???
// - this is just my personal suffer zone

#[derive(Copy, Clone)]
pub struct Sides {
    pub top: bool,
    pub bottom: bool,
    pub right: bool,
    pub left: bool,
    pub corner: bool,
}

impl Sides {
    pub fn new() -> Self {
        Self {
            right: false,
            left: false,
            top: false,
            bottom: false,
            corner: false,
        }
    }
}

impl Default for Sides {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AabbCollision<ID: Copy + Eq> {
    pub bodies: Vec<Aabb>,
    pub velocities: Vec<Vec2>,
    pub metadata: Vec<CollisionData<ID>>,
    pub contacts: Vec<(usize, usize, Sides)>,
    pub displacements: Vec<Option<Vec3>>,
    pub sides_touched: Vec<Sides>,
}

#[derive(Default, Clone, Copy)]
pub struct CollisionData<ID: Copy + Eq> {
    solid: bool,
    fixed: bool,
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
        self.step_update();

        self.contacts.clear();
        for (vel, displacement) in self.velocities.iter_mut().zip(self.displacements.iter()) {
            if let Some(displace) = displacement {
                if displace.x != 0.0 {
                    vel.x = 0.0;
                }

                if displace.y != 0.0 {
                    vel.y = 0.0;
                }
            } else {
                vel.x = 0.0;
                vel.y = 0.0;
            }
        }
        self.displacements.clear();
        self.displacements.resize_with(self.bodies.len(), Default::default);
        self.step_update();
    }

    fn step_update(&mut self) {
        for (i, body) in self.bodies.iter().enumerate() {
            for (j, body2) in self.bodies.iter().enumerate() {
                if body.intersects(body2) && i != j {
                    let sides = self.update_sides_touched(i, j);

                    if sides.top { self.sides_touched[i].top = true; }
                    if sides.bottom { self.sides_touched[i].bottom = true; }
                    if sides.left { self.sides_touched[i].left = true; }
                    if sides.right { self.sides_touched[i].right = true; }
                    if sides.corner { self.sides_touched[i].corner = true; }
                    self.contacts.push((i, j, sides));
                }
            }
        }

        for (i, j, sides) in self.contacts.iter() {
            let CollisionData { solid: i_solid, fixed: i_fixed, .. } =
                self.metadata[*i];
            let CollisionData { solid: j_solid, fixed: j_fixed, .. } =
                self.metadata[*j];

            if !(i_solid && j_solid) || i_fixed {
                continue;
            }

            let Vec2 { x: vel_i_x, y: vel_i_y } = self.velocities[*i];
            let Vec2 { x: vel_j_x, y: vel_j_y } = self.velocities[*j];
            let mut speed_ratio = Vec2::new(1.0, 1.0);
            let speed_sum = Vec2::new(vel_i_x.abs() + vel_j_x.abs(), vel_i_y.abs() + vel_j_y.abs());
            if !j_fixed {
                if speed_sum.x == 0.0 && speed_sum.y == 0.0 {
                    speed_ratio.x = 0.5;
                    speed_ratio.y = 0.5;
                } else if speed_sum.x == 0.0 {
                    speed_ratio.x = 0.5;
                    speed_ratio.y = vel_i_y.abs() / speed_sum.y;
                } else if speed_sum.y == 0.0 {
                    speed_ratio.y = 0.5;
                    speed_ratio.x = vel_i_x.abs() / speed_sum.x;
                } else {
                    speed_ratio.x = vel_i_x.abs() / speed_sum.x;
                    speed_ratio.y = vel_i_y.abs() / speed_sum.y;
                }
            }

            let Aabb {
                min: Vec3 { x: min_i_x, y: min_i_y, .. },
                max: Vec3 { x: max_i_x, y: max_i_y, ..}
            } = self.bodies[*i];
            let Aabb {
                min: Vec3 { x: min_j_x, y: min_j_y, .. },
                max: Vec3 { x: max_j_x, y: max_j_y, ..}
            } = self.bodies[*j];

            let mut displace = {
                if sides.top || sides.bottom {
                    if sides.top {
                        Vec3::new(0.0, max_j_y - min_i_y, 0.0)
                    } else {
                        Vec3::new(0.0, min_j_y - max_i_y, 0.0)
                    }
                } else if sides.left || sides.right {
                    if sides.left {
                        Vec3::new(max_j_x - min_i_x, 0.0, 0.0)
                    } else {
                        Vec3::new(min_j_x - max_i_x, 0.0, 0.0)
                    }
                } else if sides.corner {
                    let mut new_x = {
                        if vel_i_x - vel_j_x < 0.0 {
                            max_j_x - min_i_x
                        } else if vel_i_x - vel_j_x > 0.0 {
                            min_j_x - max_i_x
                        } else {
                            0.0
                        }
                    };
                    let mut new_y = {
                        if vel_i_y - vel_j_y < 0.0 {
                            max_j_y - min_i_y
                        } else if vel_i_y - vel_j_y > 0.0 {
                            min_j_y - max_i_y
                        } else {
                            0.0
                        }
                    };
                    if new_x.abs() > new_y.abs() {
                        new_y =  {
                            if vel_i_y - vel_j_y > 0.0 { -1.0 * new_x.abs() } else { new_x.abs() }
                        };
                    } else if new_y.abs() > new_x.abs() {
                        new_x =  {
                            if vel_i_x - vel_j_x > 0.0 { -1.0 * new_y.abs() } else { new_y.abs() }
                        };
                    }
                    Vec3::new(new_x, new_y, 0.0)

                } else {  // if everything is false because vx and vy == 0 but still overlapping...
                    Vec3::new(0.0, 0.0, 0.0)
                }
            };
            displace.x *= speed_ratio.x;
            displace.y *= speed_ratio.y;

            let all_sides = &self.sides_touched[*i];
            if let Some(new_displace) = &mut self.displacements[*i] {
                // already touching at least one side and no corners
                if {all_sides.bottom || all_sides.top || all_sides.left || sides.right}
                && !all_sides.corner {
                    // touching at two different sides
                    if (all_sides.top || all_sides.bottom) && (all_sides.left || all_sides.right) {
                        if displace.x.abs() > new_displace.x.abs() {
                            new_displace.x = displace.x;
                        } 
                        if displace.y.abs() > new_displace.y.abs() {
                            new_displace.y = displace.y;
                        }
                        // touching one side
                    } else {
                        if all_sides.top || all_sides.bottom {
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
                } else if all_sides.corner {
                    if all_sides.top || all_sides.bottom {
                        if displace.y.abs() > new_displace.y.abs() {
                            new_displace.y = displace.y;
                        }
                        new_displace.x = 0.0;
                    } else if all_sides.left || all_sides.right {
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
                self.displacements[*i] = Some(displace);
            }
        }

        for (i, displacement) in self.displacements.iter().enumerate() {
            match displacement {
                Some(new_displace) => {
                    self.bodies[i].min += *new_displace;
                    self.bodies[i].max += *new_displace;
                }
                None => {}
            }
        }
    }

    fn update_sides_touched(&self, i: usize, j: usize) -> Sides {
        let rel_vel_x = self.velocities[i].x - self.velocities[j].x;
        let rel_vel_y = self.velocities[i].y - self.velocities[j].y;

        let Aabb {
            min: Vec3 { x: min_i_x, y: min_i_y, .. },
            max: Vec3 { x: max_i_x, y: max_i_y, ..}
        } = self.bodies[i];
        let Aabb {
            min: Vec3 { x: min_j_x, y: min_j_y, .. },
            max: Vec3 { x: max_j_x, y: max_j_y, ..}
        } = self.bodies[j];

        let half_isize_x = (max_i_x - min_i_x) / 2.0;
        let half_isize_y = (max_i_y - min_i_y) / 2.0;
        let half_jsize_x = (max_j_x - min_j_x) / 2.0;
        let half_jsize_y = (max_j_y - min_j_y) / 2.0;

        let i_center = Vec2::new(
            (max_i_x + min_i_x) / 2.0,
            (max_i_y + min_i_y) / 2.0);
        let j_center = Vec2::new(
            (max_j_x + min_j_x) / 2.0,
            (max_j_y + min_j_y) / 2.0);

        let overlapped_before_x = {
            let old_ix_center = i_center.x - self.velocities[i].x;
            let old_jx_center = j_center.x - self.velocities[j].x;
            (old_ix_center - old_jx_center).abs() < half_isize_x + half_jsize_x
        };

        let overlapped_before_y = {
            let old_iy_center = i_center.y - self.velocities[i].y;
            let old_jy_center = j_center.y - self.velocities[j].y;
            (old_iy_center - old_jy_center).abs() < half_isize_y + half_jsize_y
        };

        let mut sides = Sides::new();

        if !overlapped_before_y && overlapped_before_x && rel_vel_y != 0.0 {
            if rel_vel_y < 0.0 {
                sides.top = true;
            } else {
                sides.bottom = true;
            }
        }

        if !overlapped_before_x && overlapped_before_y && rel_vel_x != 0.0 {
            if rel_vel_x < 0.0 {
                sides.left = true;
            } else {
                sides.right = true;
            }
        }

        if !overlapped_before_x && !overlapped_before_y && rel_vel_x != 0.0 && rel_vel_y != 0.0 {
            sides.corner = true;
        }
        sides
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
