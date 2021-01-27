//! # Collision logics
//!
//! Collision logics offer an illusion of physical space is provided by the fact that some game
//! objects occlude the movement of others. They detect overlaps between subsets of entities and/or
//! regions of space, and automatically trigger reactions when such overlaps occur.
//!
//! Note: Collision is hard and may be broken.

use glam::Vec2 as GlamVec2;
use std::cmp::Ordering;
use std::ops::{Add, AddAssign};
use ultraviolet::Vec2 as UVVec2;

/// A trait for a set of two coordinates that represent a point in 2d space.
pub trait Vec2: Add + AddAssign + Copy {
    fn new(x: f32, y: f32) -> Self;
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn set_x(&mut self, x: f32);
    fn set_y(&mut self, y: f32);

    fn magnitude(&self) -> f32 {
        (self.x().powi(2) + self.y().powi(2)).sqrt()
    }
}

impl Vec2 for UVVec2 {
    fn new(x: f32, y: f32) -> UVVec2 {
        UVVec2::new(x, y)
    }
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }
    fn set_x(&mut self, x: f32) {
        self.x = x;
    }
    fn set_y(&mut self, y: f32) {
        self.y = y;
    }
}

impl Vec2 for GlamVec2 {
    fn new(x: f32, y: f32) -> GlamVec2 {
        GlamVec2::new(x, y)
    }
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }
    fn set_x(&mut self, x: f32) {
        self.x = x;
    }
    fn set_y(&mut self, y: f32) {
        self.y = y;
    }
}

/// Information for each contact.
pub struct Contact<V2: Vec2> {
    /// The index of the first contact in `centers`, `half_sizes`, `velocities`, `metadata`, and
    /// `displacements`.
    pub i: usize,
    /// The index of the second contact in `centers`, `half_sizes`, `velocities`, `metadata`, and
    /// `displacements`.
    pub j: usize,
    /// The projected displacement of each contact---not actual restituted displacement. If both
    /// colliding bodies are fixed, or one of them is **not** solid, defaults to a `Vec2` with
    /// a magnitude of 0.0.
    pub displacement: V2,
}

impl<V2: Vec2> PartialEq for Contact<V2> {
    /// Two `Contacts`s are equal when the indices of their contacts and their displacements are
    /// the same.
    fn eq(&self, other: &Self) -> bool {
        self.i == other.i
            && self.j == other.j
            && self.displacement.x() == other.displacement.x()
            && self.displacement.y() == other.displacement.y()
    }
}

impl<V2: Vec2> PartialOrd for Contact<V2> {
    /// A `Contact` is bigger than another when the magnitude of how much the contact should be
    /// restituted is greater than the other.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let e1 = self.get_restitution().magnitude();
        let e2 = other.get_restitution().magnitude();
        e1.partial_cmp(&e2)
    }
}

impl<V2: Vec2> Contact<V2> {
    /// Returns how much the contact should be restituted, not taking into account other possible
    /// contacts.
    fn get_restitution(&self) -> V2 {
        if self.displacement.x().abs() < self.displacement.y().abs() {
            V2::new(self.displacement.x(), 0.0)
        } else if self.displacement.y().abs() < self.displacement.x().abs() {
            V2::new(0.0, self.displacement.y())
        } else {
            V2::new(0.0, 0.0)
        }
    }
}

/// Metadata of each collision entity.
#[derive(Default, Clone, Copy)]
pub struct CollisionData<ID: Copy + Eq> {
    /// True if the entity is solid, i.e. can stop other entities.
    ///
    /// For example, a wall or player character might be solid, while a section of the ground that
    /// applies an affect on the player character when they walk over it (colliding with it) might
    /// not be.
    pub solid: bool,
    /// True if the entity is fixed, i.e. does _not_ participate in restitution.
    ///
    /// Pushable entities are _not_ fixed, while entities that shouldn't be pushable, such as walls
    /// or moving platforms, are.
    pub fixed: bool,
    pub id: ID,
}

/// A collision logic for axis-aligned bounding boxes.
pub struct AabbCollision<ID: Copy + Eq, V2: Vec2> {
    /// A vector of the centers of the bounding box.
    pub centers: Vec<V2>,
    /// A vector of half the width and half the height of the bounding box.
    pub half_sizes: Vec<V2>,
    /// A vector of the velocity of the entities.
    pub velocities: Vec<V2>,
    /// A vector of entity metadata.
    pub metadata: Vec<CollisionData<ID>>,
    /// A vector of all entities that are touching.
    ///
    /// Indices do _not_ run parallel with those in the above vectors.
    pub contacts: Vec<Contact<V2>>,
    /// A vector of how much each entity is displaced overall.
    ///
    /// Indices _do_ run parallel with other vectors in the struct other than `contacts`.
    pub displacements: Vec<V2>,
}

pub struct ContactCollisionID<ID: Copy + Eq>(ID, ID);

impl<ID: Copy + Eq> PartialEq for ContactCollisionID<ID> {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
    }
}

impl<ID: Copy + Eq, V2: Vec2> AabbCollision<ID, V2> {
    pub fn new() -> Self {
        Self {
            centers: Vec::new(),
            half_sizes: Vec::new(),
            velocities: Vec::new(),
            metadata: Vec::new(),
            contacts: Vec::new(),
            displacements: Vec::new(),
        }
    }

    /// Checks collisions every frame and handles restitution (works most of the time).
    ///
    /// Uses the following algorithm:
    ///
    /// 1. Find all contacts, add to a big vec
    /// 2. Sort the big list by decreasing magnitude of displacement
    /// 3. Have a vec<Vec2> of length # collision bodies; these are how much each body has been
    ///    displaced so far during restitution.
    /// 4. Process contacts vec in order: adding the displacement so far for each
    ///    involved entity to the contact displacement, displace the correct entity the correct
    ///    "remaining" amount (which might be 0) and add that to the vec of (3).
    ///
    /// Explanation lightly modified from a message by Prof Osborn.
    pub fn update(&mut self) {
        self.contacts.clear();
        self.displacements.clear();
        self.displacements
            .resize(self.centers.len(), V2::new(0.0, 0.0));

        // check contacts
        for i in 0..self.centers.len() {
            for j in i + 1..self.centers.len() {
                // if i is fixed and other is unfixed, swap places
                let temp_i = i;
                let i = if self.metadata[i].fixed && !self.metadata[j].fixed {
                    j
                } else {
                    i
                };
                let j = if self.metadata[temp_i].fixed && !self.metadata[j].fixed {
                    temp_i
                } else {
                    j
                };

                if self.intersects(i, j) {
                    let displacement = if self.metadata[i].solid && self.metadata[j].solid {
                        let displ = self.find_displacement(i, j);
                        let speed_ratio = self.get_speed_ratio(i, j);
                        V2::new(displ.x() * speed_ratio.x(), displ.y() * speed_ratio.y())
                    } else {
                        V2::new(0.0, 0.0)
                    };
                    let contact = Contact { i, j, displacement };
                    self.contacts.push(contact);
                }
            }
        }

        self.contacts.sort_by(|e1, e2| e2.partial_cmp(e1).unwrap());

        let remove = &mut Vec::<usize>::new();

        // sorted by magnitude of displ.
        // if both unfixed, need to displace both
        for (idx, contact) in self.contacts.iter().enumerate() {
            let Contact {
                i,
                j,
                displacement: displ_i,
                ..
            } = *contact;

            let already_moved_i = self.displacements[i];
            let remaining_displ_i = V2::new(
                displ_i.x() - already_moved_i.x(),
                displ_i.y() - already_moved_i.y(),
            );
            let flipped_i = {
                if {
                    already_moved_i.x().abs() > displ_i.x().abs()
                        || already_moved_i.y().abs() > displ_i.y().abs()
                } {
                    true
                } else {
                    false
                }
            };
            if flipped_i {
                remove.push(idx);
            }

            if !(remaining_displ_i.x() == 0.0 || remaining_displ_i.y() == 0.0 || flipped_i) {
                if displ_i.x().abs() < displ_i.y().abs() {
                    self.displacements[i].set_x(remaining_displ_i.x());
                } else if displ_i.y().abs() < displ_i.x().abs() {
                    self.displacements[i].set_y(remaining_displ_i.y());
                }
            }

            if !self.metadata[j].fixed {
                let already_moved_j = self.displacements[j];
                let displ_j = {
                    let displ_full = self.find_displacement(j, i);
                    let speed_ratio = self.get_speed_ratio(j, i);
                    V2::new(
                        displ_full.x() * speed_ratio.x(),
                        displ_full.y() * speed_ratio.y(),
                    )
                };
                let remaining_displ_j = V2::new(
                    displ_j.x() - already_moved_j.x(),
                    displ_j.y() - already_moved_j.y(),
                );
                let flipped_j = {
                    if {
                        already_moved_j.x().abs() > displ_j.x().abs()
                            || already_moved_j.y().abs() > displ_j.y().abs()
                    } {
                        true
                    } else {
                        false
                    }
                };
                if flipped_j && (remove.len() > 0 && remove[remove.len() - 1] != idx) {
                    remove.push(idx);
                }

                if !(remaining_displ_j.x() == 0.0 || remaining_displ_j.y() == 0.0 || flipped_j) {
                    if displ_j.x().abs() < displ_j.y().abs() {
                        self.displacements[j].set_x(remaining_displ_j.x());
                    } else if displ_j.y().abs() < displ_j.x().abs() {
                        self.displacements[j].set_y(remaining_displ_j.y());
                    }
                }
            }
        }

        for i in remove.iter().rev() {
            self.contacts.remove(*i);
        }

        for (i, displacement) in self.displacements.iter().enumerate() {
            self.centers[i] += *displacement;
        }
    }

    pub fn find_displacement(&self, i: usize, j: usize) -> V2 {
        let (ci, cj) = (self.centers[i], self.centers[j]);
        let (hsi, hsj) = (self.half_sizes[i], self.half_sizes[j]);
        let displ_abs = V2::new(
            hsi.x() + hsj.x() - (ci.x() - cj.x()).abs(),
            hsi.y() + hsj.y() - (ci.y() - cj.y()).abs(),
        );
        V2::new(
            if ci.x() - cj.x() < 0.0 { -1.0 } else { 1.0 } * displ_abs.x(),
            if ci.y() - cj.y() < 0.0 { -1.0 } else { 1.0 } * displ_abs.y(),
        )
    }

    /// Calculates the speed ratio of the two entities, i.e. the amount of restitution an
    /// entity should be responsible for.
    ///
    /// Assumes that the entity at index `i` is unfixed. When the entity at index `j` is fixed,
    /// entity `i` will be responsible for all of the restitution. Otherwise, it is responsible
    /// for an amount of restitution proportional to the entities' velocity.
    ///
    /// I think this is mostly ripped from this tutorial:
    /// https://gamedevelopment.tutsplus.com/series/basic-2d-platformer-physics--cms-998
    fn get_speed_ratio(&self, i: usize, j: usize) -> V2 {
        if !self.metadata[j].fixed {
            let (vxi, vyi) = (self.velocities[i].x().abs(), self.velocities[i].y().abs());
            let (vxj, vyj) = (self.velocities[j].x().abs(), self.velocities[j].y().abs());

            let speed_sum = V2::new(vxi + vxj, vyi + vyj);
            let mut speed_ratio = if speed_sum.x() == 0.0 && speed_sum.y() == 0.0 {
                V2::new(0.5, 0.5)
            } else if speed_sum.x() == 0.0 {
                V2::new(0.5, vyi / speed_sum.y())
            } else if speed_sum.y() == 0.0 {
                V2::new(vxi / speed_sum.x(), 0.5)
            } else {
                V2::new(vxi / speed_sum.x(), vyi / speed_sum.y())
            };

            if speed_ratio.x() == 0.0 {
                speed_ratio.set_x(1.0);
            }
            if speed_ratio.y() == 0.0 {
                speed_ratio.set_y(1.0);
            }

            speed_ratio
        } else {
            V2::new(1.0, 1.0)
        }
    }

    /// Adds a collision entity to the logic, taking two Vec2s with the center and half the
    /// dimensions of the AABB. `solid` represents if the entity can stop other entities, and
    /// `fixed` represents if it can participate in restitution, i.e. be moved by the collision
    /// logic or not. See [CollisionData] for further explanation.
    pub fn add_collision_entity(
        &mut self,
        center: V2,
        half_size: V2,
        vel: V2,
        solid: bool,
        fixed: bool,
        id: ID,
    ) {
        self.centers.push(center);
        self.half_sizes.push(half_size);
        self.velocities.push(vel);
        self.metadata.push(CollisionData { solid, fixed, id });
    }

    /// Adds a collision entity to the logic, taking the x and y positions, width, and height of the AABB as
    /// well as its velocity and some metadata. See
    /// [add_collision_entity][AabbCollision::add_collision_entity] for details on what the other
    /// fields represent.
    pub fn add_entity_as_xywh(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        vel: V2,
        solid: bool,
        fixed: bool,
        id: ID,
    ) {
        self.add_collision_entity(
            V2::new(x + w / 2.0, y + h / 2.0),
            V2::new(w / 2.0, h / 2.0),
            vel,
            solid,
            fixed,
            id,
        );
    }

    /// Returns unit vector of normal of displacement for the `i` entity in the contact. I.e.,
    /// if a contact is moved in a positive x direction after restitution _because of_ the
    /// other entity involved in collision, `sides_touched` will return `V2::new(1.0, 0.0)`.
    pub fn sides_touched(&self, contact: &Contact<V2>) -> V2 {
        let restitution = contact.get_restitution();
        let x = match restitution.x().partial_cmp(&0.0).unwrap() {
            Ordering::Equal => 0.0,
            Ordering::Less => -1.0,
            Ordering::Greater => 1.0,
        };
        let y = match restitution.y().partial_cmp(&0.0).unwrap() {
            Ordering::Equal => 0.0,
            Ordering::Less => -1.0,
            Ordering::Greater => 1.0,
        };
        let mut unit = V2::new(x, y);

        if unit.magnitude() == 0.0 {
            // if unit.magnitude() == 0, that means either the x or y value of displ
            // = 0.0
            let displ = contact.displacement;
            let center_displ = V2::new(
                self.centers[contact.i].x() - self.centers[contact.j].x(),
                self.centers[contact.i].y() - self.centers[contact.j].y(),
            );
            if displ.x() == 0.0 {
                match center_displ.x().partial_cmp(&0.0).unwrap() {
                    Ordering::Equal => {}
                    Ordering::Less => unit.set_x(-1.0),
                    Ordering::Greater => unit.set_x(1.0),
                }
            }
            if displ.y() == 0.0 {
                match center_displ.y().partial_cmp(&0.0).unwrap() {
                    Ordering::Equal => {}
                    Ordering::Less => unit.set_y(-1.0),
                    Ordering::Greater => unit.set_y(1.0),
                }
            }
        }
        unit
    }

    /// Gets the center for the entity given its CollisionID, if it exists.
    pub fn get_center_for_entity(&self, id: ID) -> Option<V2> {
        if let Some(i) = self.metadata.iter().position(|metadata| metadata.id == id) {
            Some(self.centers[i])
        } else {
            None
        }
    }

    /// Gets the (x, y) position for the entity given its CollisionID, if it exists: (center -
    /// half_size). Matches with [add_entity_as_xywh].
    pub fn get_xy_pos_for_entity(&self, id: ID) -> Option<V2> {
        if let Some(i) = self.metadata.iter().position(|metadata| metadata.id == id) {
            let center = self.centers[i];
            let half_size = self.half_sizes[i];
            Some(V2::new(
                center.x() - half_size.x(),
                center.y() - half_size.y(),
            ))
        } else {
            None
        }
    }

    pub fn get_ids_for_contact(&self, contact: &Contact<V2>) -> ContactCollisionID<ID> {
        let Contact { i, j, .. } = *contact;
        ContactCollisionID(self.metadata[i].id, self.metadata[j].id)
    }

    fn intersects(&self, i: usize, j: usize) -> bool {
        (self.centers[i].x() - self.centers[j].x()).abs()
            <= self.half_sizes[i].x() + self.half_sizes[j].x()
            && (self.centers[i].y() - self.centers[j].y()).abs()
                <= self.half_sizes[i].y() + self.half_sizes[j].y()
    }
}
