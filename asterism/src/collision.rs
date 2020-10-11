use ultraviolet::Vec2 as UVVec2;
use glam::Vec2 as GlamVec2;
use std::ops::{Add, AddAssign, Mul};

pub trait Vec2 {
    fn new(x: f32, y: f32) -> Self;
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn set_x(&mut self, x: f32);
    fn set_y(&mut self, y: f32);
}

impl Vec2 for UVVec2 {
    fn new(x: f32, y: f32) -> UVVec2 { UVVec2::new(x, y) }
    fn x(&self) -> f32 { self.x }
    fn y(&self) -> f32 { self.y }
    fn set_x(&mut self, x: f32) { self.x = x; }
    fn set_y(&mut self, y: f32) { self.y = y; }
}

impl Vec2 for GlamVec2 {
    fn new(x: f32, y: f32) -> GlamVec2 { GlamVec2::new(x, y) }
    fn x(&self) -> f32 { GlamVec2::x(*self) }
    fn y(&self) -> f32 { GlamVec2::x(*self) }
    fn set_x(&mut self, x: f32) { GlamVec2::set_x(&mut *self, x) }
    fn set_y(&mut self, y: f32) { GlamVec2::set_y(&mut *self, y) }
}

pub struct AabbCollision<ID, V2> where
    ID: Copy + Eq,
    V2: Vec2 + Add + AddAssign + Mul<Output = V2> + Copy {
            // this type parameter makes me want to cry
    pub centers: Vec<V2>,
    pub half_sizes: Vec<V2>,
        // Vec2 {x: width * 0.5, y: height * 0.5}
    pub velocities: Vec<V2>,
    pub metadata: Vec<CollisionData<ID>>,
    pub contacts: Vec<(usize, usize, [bool; 5])>,
    pub displacements: Vec<Option<V2>>,
    pub sides_touched: Vec<[bool; 5]>,
}

#[derive(Default, Clone, Copy)]
pub struct CollisionData<ID: Copy + Eq> {
    solid: bool,
    fixed: bool,
    pub id: ID,
}

impl<ID, V2> AabbCollision<ID, V2> where
    ID: Copy + Eq,
    V2: Vec2 + Add + AddAssign + Mul<Output = V2> + Copy {
    pub fn new() -> Self {
        Self {
            centers: Vec::new(),
            half_sizes: Vec::new(),
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
        self.displacements.resize_with(self.centers.len(), Default::default);
        self.sides_touched.resize_with(self.centers.len(), Default::default);
        self.step_update();

        self.contacts.clear();
        for (vel, displacement) in self.velocities.iter_mut().zip(self.displacements.iter()) {
            if let Some(displace) = displacement {
                if displace.x() != 0.0 {
                    vel.set_x(0.0);
                }
                if displace.y() != 0.0 {
                    vel.set_y(0.0);
                }
            } else {
                vel.set_x(0.0);
                vel.set_y(0.0);
            }
        }
        self.displacements.clear();
        self.displacements.resize_with(self.centers.len(), Default::default);
        self.step_update();
    }

    fn step_update(&mut self) {
        for (i, (ci, hsi)) in self.centers.iter()
            .zip(self.half_sizes.iter()).enumerate() {
            for (j, (cj, hsj)) in self.centers.iter()
                .zip(self.half_sizes.iter()).enumerate() {
                if i != j && {
                    (ci.x() - cj.x()).abs() <= hsi.x() + hsj.x() &&
                    (ci.y() - cj.y()).abs() <= hsi.y() + hsj.y()
                } {
                    let sides = self.update_sides_touched(i, j);
                    for (k, is_touched) in sides.iter().enumerate() {
                        if *is_touched {
                            self.sides_touched[i][k] = true;
                        }
                    }
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

            let (vxi, vyi) =
                (self.velocities[*i].x(), self.velocities[*i].y());
            let (vxj, vyj) =
                (self.velocities[*j].x(), self.velocities[*j].y());

            let speed_sum = V2::new(
                vxi.abs() + vxj.abs(),
                vyi.abs() + vyj.abs());
            let speed_ratio = {
                if !j_fixed {
                    if speed_sum.x() == 0.0 && speed_sum.y() == 0.0 {
                        V2::new(0.5, 0.5)
                    } else if speed_sum.x() == 0.0 {
                        V2::new(0.5, vyi.abs() / speed_sum.y())
                    } else if speed_sum.y() == 0.0 {
                        V2::new(0.5, vxi.abs() / speed_sum.x())
                    } else {
                        V2::new(vxi.abs() / speed_sum.x(),
                            vyi.abs() / speed_sum.y())
                    }
                } else {
                    V2::new(1.0, 1.0)
                }
            };

            let mut displace = self.find_displacement(*i, *j);
            displace = displace * speed_ratio;

            let all_sides = &self.sides_touched[*i];
            if let Some(new_displace) = &mut self.displacements[*i] {
                // already touching at least one side and no corners
                if { all_sides[0] || all_sides[1] || all_sides[2] || sides[3]} && !all_sides[4] {
                    // touching at two different sides
                    if (all_sides[0] || all_sides[1]) && (all_sides[2] || all_sides[3]) {
                        if displace.x().abs() > new_displace.x().abs() {
                            new_displace.set_x(displace.x());
                        } 
                        if displace.y().abs() > new_displace.y().abs() {
                            new_displace.set_y(displace.y());
                        }
                    // touching one side
                    } else {
                        if all_sides[0] || all_sides[1] {
                            if displace.y().abs() > new_displace.y().abs() {
                                new_displace.set_y(displace.y());
                            }
                        } else {
                            if displace.x().abs() > new_displace.x().abs() {
                                new_displace.set_x(displace.x());
                            }
                        }
                    }
                // already touching a corner
                } else if all_sides[4] {
                    if all_sides[0] || all_sides[1] {
                        if displace.y().abs() > new_displace.y().abs() {
                            new_displace.set_y(displace.y());
                        }
                        new_displace.set_x(0.0);
                    } else if all_sides[2] || all_sides[3] {
                        if displace.x().abs() > new_displace.x().abs() {
                            new_displace.set_x(displace.x());
                        }
                        new_displace.set_y(0.0);
                    } else {  // touching corner(s) only or something
                        if displace.x().abs() > new_displace.x().abs() {
                            new_displace.set_x(displace.x());
                        }
                        if displace.y().abs() > new_displace.y().abs() {
                            new_displace.set_y(displace.y());
                        }
                    }
                }
            } else {
                self.displacements[*i] = Some(displace);
            }
        }

        for (i, displacement) in self.displacements.iter().enumerate() {
            match displacement {
                Some(new_displace) => self.centers[i] += *new_displace,
                None => {}
            }
        }
    }

    fn update_sides_touched(&self, i: usize, j: usize) -> [bool; 5] {
        let ci = self.centers[i];
        let cj = self.centers[j];
        let hsi = self.half_sizes[i];
        let hsj = self.half_sizes[j];

        let overlapped_before_x = {
            let old_ci = ci.x() - self.velocities[i].x();
            let old_cj = cj.x() - self.velocities[j].x();
            (old_ci - old_cj).abs() < hsi.x() + hsj.x()
        };

        let overlapped_before_y = {
            let old_ci = ci.y() - self.velocities[i].y();
            let old_cj = cj.y() - self.velocities[j].y();
            (old_ci - old_cj).abs() < hsi.y() + hsj.y()
        };

        let rel_vel_x = self.velocities[i].x() - self.velocities[j].x();
        let rel_vel_y = self.velocities[i].y() - self.velocities[j].y();
        let mut sides = [false; 5];


        if !overlapped_before_y && overlapped_before_x && rel_vel_y != 0.0 {
            if rel_vel_y < 0.0 {
                sides[0] = true;
            } else {
                sides[1] = true;
            }
        }

        if !overlapped_before_x && overlapped_before_y && rel_vel_x != 0.0 {
            if rel_vel_x < 0.0 {
                sides[2] = true;
            } else {
                sides[3] = true;
            }
        }

        if !overlapped_before_x && !overlapped_before_y && rel_vel_x != 0.0 && rel_vel_y != 0.0 {
            sides[4] = true;
        }
        sides
    }

    fn find_displacement(&self, i: usize, j: usize) -> V2 {
        let sides = self.sides_touched[i];
        let (ci, cj) = (self.centers[i], self.centers[j]);
        let (hsi, hsj) = (self.half_sizes[i], self.half_sizes[j]);
        let dx = hsi.x() + hsj.x() - (ci.x() - cj.x()).abs();
        let dy = hsi.y() + hsj.y() - (ci.y() - cj.y()).abs();

        let (vxi, vyi) =
            (self.velocities[i].x(), self.velocities[i].y());
        let (vxj, vyj) =
            (self.velocities[j].x(), self.velocities[j].y());

        if sides[2] || sides[3] {
            V2::new(0.0, dy)
        } else if sides[0] || sides[1] {
            V2::new(dx, 0.0)
        } else if sides[4] {
            let mut new_x = if vxi == vyi { dx } else { 0.0 };
            let mut new_y = if vyi == vyj { dy } else { 0.0 };
            if new_x > new_y {
                new_y = if vyi > vyj { -new_x } else { new_x };
            } else if new_y > new_x {
                new_x = if vxi < vxj { -new_y } else { new_y };
            }
            V2::new(new_x, new_y)

        } else {  // if everything is false because vx and vy == 0 but still overlapping... ??????????? wtf?????????? what is this????
            V2::new(0.0, 0.0)
        }
    }

    pub fn add_collision_entity(&mut self, center: V2, half_size: V2, vel: V2, solid: bool, fixed: bool, id: ID) {
        self.centers.push(center);
        self.half_sizes.push(half_size);
        self.velocities.push(vel);
        self.metadata.push(CollisionData { solid: solid, fixed: fixed, id: id });
        self.displacements.push(None);
    }
}
