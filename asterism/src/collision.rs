use ultraviolet::Vec2 as UVVec2;
use glam::Vec2 as GlamVec2;
use std::ops::{Add, AddAssign, Mul};
use std::cmp::Ordering;

/*

1. Find all contacts, add to a big vec
2. Sort the big list by decreasing magnitude of displacement
3. Have a vec<Vec2> of length # collision bodies; these are how much each body has been displaced so far during restitution, all initialized to (0, 0)
4. Process contacts vec in order: appropriately adding the displacement so far for each involved entity to the contact displacement, displace the correct entity the correct "remaining" amount (which might be 0) and add that to the vec of (3). This could change "penetrating" contacts to "touching" contacts, so we should probably modify the contacts vec as we go too to update with the actual displacement performed.

*/

pub trait Vec2 {
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

pub struct Contact<V2: Vec2> {
    pub i: usize,
    pub j: usize,
    pub displacement: V2
}

impl<V2: Vec2> PartialEq for Contact<V2> {
    fn eq(&self, other: &Self) -> bool {
        self.i == other.i
            && self.j == other.j
            && self.displacement.x() == other.displacement.x()
            && self.displacement.y() == other.displacement.y()
    }
}

// impl<V2: Vec2> Eq for Contact<V2> {}

impl<V2: Vec2> PartialOrd for Contact<V2> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.displacement.magnitude()
            .partial_cmp(&other.displacement.magnitude())
    }
}

/* impl<V2: Vec2> Ord for Contact<V2> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
            // uh i sure hope this doesnt panic lol
    }
} */

pub struct AabbCollision<ID, V2> where
ID: Copy + Eq,
V2: Vec2 + Add + AddAssign + Mul<Output = V2> + Copy {
        // this trait bound makes me want to cry
    pub centers: Vec<V2>,
    pub half_sizes: Vec<V2>,
        // Vec2 {x: width * 0.5, y: height * 0.5}
    pub velocities: Vec<V2>,
    pub metadata: Vec<CollisionData<ID>>,
    pub contacts: Vec<Contact<V2>>,
    pub displacements: Vec<V2>
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
            displacements: Vec::new()
        }
    }

    pub fn update(&mut self) {
        self.contacts.clear();
        self.displacements.clear();
        self.displacements.resize(self.centers.len(), V2::new(0.0, 0.0));

        // check contacts
        for (i, (ci, hsi)) in self.centers.iter()
            .zip(self.half_sizes.iter()).enumerate() {
            for (j, (cj, hsj)) in self.centers.iter()
                .zip(self.half_sizes.iter()).enumerate() {
                if i != j
                    && (ci.x() - cj.x()).abs() <= hsi.x() + hsj.x()
                    && (ci.y() - cj.y()).abs() <= hsi.y() + hsj.y()
                {
                    let displacement = if
                        self.metadata[i].solid && self.metadata[j].solid
                        && !self.metadata[i].fixed
                    {
                        self.find_displacement(i, j) * self.get_speed_ratio(i, j)
                    } else {
                        V2::new(0.0, 0.0)
                    };
                    let contact = Contact {
                        i, j, displacement
                    };
                    self.contacts.push(contact);
                    self.contacts.sort_by(|e1, e2| e2.partial_cmp(e1).unwrap());
                }
            }
        }

        // do some sort of....... iteration thru contacts
        // sorted by magnitude of displ.
        for contact in self.contacts.iter() {
            let Contact {
                i, j: _, displacement: displ
            } = *contact;

            let already_moved = &mut self.displacements[i];

            let remaining_displ = V2::new(
                displ.x() - already_moved.x(),
                displ.y() - already_moved.y());

            if (remaining_displ.x() == 0.0
                || remaining_displ.y() == 0.0)
                && displ.magnitude() != 0.0 {
                continue;
            }

            if displ.x().abs() < displ.y().abs() {
                already_moved.set_x(remaining_displ.x());
            } else if displ.y().abs() < displ.x().abs() {
                already_moved.set_y(remaining_displ.y());
            }
        }

        for (i, displacement) in self.displacements.iter().enumerate() {
            self.centers[i] += *displacement;
        }
    }

    fn find_displacement(&self, i: usize, j: usize) -> V2 {
        let (ci, cj) = (self.centers[i], self.centers[j]);
        let (hsi, hsj) = (self.half_sizes[i], self.half_sizes[j]);
        let displ_abs = V2::new(
            hsi.x() + hsj.x() - (ci.x() - cj.x()).abs(),
            hsi.y() + hsj.y() - (ci.y() - cj.y()).abs()
        );
        let displ_signs = V2::new(
            if ci.x() - cj.x() < 0.0 { -1.0 }
            else { 1.0 },
            if ci.y() - cj.y() < 0.0 { -1.0 }
            else { 1.0 }
        );
        displ_abs * displ_signs
    }

    fn get_speed_ratio(&self, i: usize, j: usize) -> V2 {
        let (vxi, vyi) =
            (self.velocities[i].x(), self.velocities[i].y());
        let (vxj, vyj) =
            (self.velocities[j].x(), self.velocities[j].y());

        let speed_sum = V2::new(vxi.abs() + vxj.abs(), vyi.abs() + vyj.abs());
        if !self.metadata[j].fixed {
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
    }

    pub fn add_collision_entity(&mut self, center: V2, half_size: V2, vel: V2, solid: bool, fixed: bool, id: ID) {
        self.centers.push(center);
        self.half_sizes.push(half_size);
        self.velocities.push(vel);
        self.metadata.push(CollisionData { solid: solid, fixed: fixed, id: id });
    }

}

