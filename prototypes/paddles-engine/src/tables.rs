// abandoned, doesn't compile

use std::cell::RefCell;
use std::rc::Rc;

use asterism::collision::{AabbColData, AabbCollision, CollisionEvent};
use asterism::control::{Action, Values};
use asterism::physics::PointPhysData;
use asterism::Logic;

use crate::types::*;

pub struct ConditionTable {
    row: Vec<bool>,
    query_arrays: Vec<Rc<RefCell<dyn QueryArray>>>,
}

impl ConditionTable {
    fn update(&mut self) {
        for query_array in self.query_arrays.iter_mut() {
            let query_array = query_array.borrow_mut().unwrap();
        }
    }
}

// query arrays, composed to build condition tables
pub trait QueryArray<L: Logic, Unit> {
    fn add_query(&mut self, query: Box<dyn Fn(&Unit) -> bool>);
    fn build_table(&mut self, logic: L);
}

pub struct CollisionQueryArray {
    queries: Vec<Box<dyn Fn(&AabbColData) -> bool>>,
    output: Vec<Vec<bool>>,
}

impl QueryArray<AabbCollision<CollisionEnt>, AabbColData> for CollisionQueryArray {
    fn add_query(&mut self, query: Box<dyn Fn(&AabbColData) -> bool>) {
        self.queries.push(query);
    }

    fn build_table(&mut self, collision: AabbCollision<CollisionEnt>) {
        self.output.resize_with(self.queries.len(), || {
            Vec::with_capacity(collision.centers.len())
        });

        let collision_data = (0..collision.centers.len())
            .map(|i| collision.get_synthesis(i))
            .collect::<Vec<_>>();

        for (row, query) in self.output.iter_mut().zip(self.queries.iter()) {
            row.resize_with(collision.centers.len(), || false);
            for (out, data) in row.iter_mut().zip(collision_data.iter()) {
                *out = query(data);
            }
        }
    }
}

pub struct ContactsQueryArray {
    queries: Vec<Box<dyn Fn(&CollisionEvent<CollisionEnt>) -> bool>>,
    output: Vec<Vec<bool>>,
}

impl QueryArray<AabbCollision<CollisionEnt>, CollisionEvent<CollisionEnt>> for ContactsQueryArray {
    fn add_query(&mut self, query: Box<dyn Fn(&CollisionEvent<CollisionEnt>) -> bool>) {
        self.queries.push(query);
    }

    fn build_table(&mut self, collision: AabbCollision<CollisionEnt>) {
        self.output.resize_with(self.queries.len(), || {
            Vec::with_capacity(collision.contacts.len())
        });

        let collision_data = &collision.contacts;

        for (row, query) in self.output.iter_mut().zip(self.queries.iter()) {
            row.resize_with(collision.contacts.len(), || false);
            for (out, data) in row.iter_mut().zip(collision_data.iter()) {
                let contact =
                    CollisionEvent(collision.metadata[data.i].id, collision.metadata[data.j].id);
                *out = query(&contact);
            }
        }
    }
}
