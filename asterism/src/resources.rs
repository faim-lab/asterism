//! # Resource Logics
//!
//! Resource logics communicate that generic or specific resources can be created, destroyed,
//! converted, or transferred between abstract or concrete locations. They create, destroy, and
//! exchange (usually) discrete quantities of generic or specific resources in or between abstract
//! or concrete locations on demand or over time, and trigger other actions when these transactions
//! take place.

use std::collections::BTreeMap;

/// A resource logic that queues transactions, then applies them all at once when updating.
pub struct QueuedResources<ID: Copy + Ord> {
    /// The items involved, and their values.
    pub items: BTreeMap<ID, f32>,
    /// Each transaction is a list of items involved in the transaction and the amount
    /// they're being changed.
    pub transactions: Vec<Vec<(ID, Transaction)>>,
    /// If the transaction was able to be completed or not. `completed[i].1` is the list
    /// of changes that could be successfully completed for the transaction.
    pub completed: Vec<(bool, Vec<ID>)>,
}

impl<ID: Copy + Ord> QueuedResources<ID> {
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            transactions: Vec::new(),
            completed: Vec::new(),
        }
    }

    /// Updates the values of resources based on the queued transactions. If a
    /// transaction cannot be completed (if the value goes below zero), a snapshot of the
    /// resources before the transaction occurred is restored, and the transaction is marked as
    /// incomplete, and we continue to process the remaining transactions.
    pub fn update(&mut self) {
        self.completed.clear();
        'exchange: for exchange in self.transactions.iter() {
            let mut snapshot: BTreeMap<ID, f32> = BTreeMap::new();
            for (item_type, ..) in exchange {
                snapshot.insert(*item_type, *self.items.get(&item_type).unwrap());
            }

            let mut item_types = Vec::new();
            for (item_type, change) in exchange.iter() {
                if !self.is_possible(item_type, change) {
                    self.completed.push((false, item_types));
                    for (item_type, val) in snapshot.iter() {
                        *self.items.get_mut(&item_type).unwrap() = *val;
                    }
                    continue 'exchange;
                }
                match change {
                    Transaction::Change(amt) => {
                        *self.items.get_mut(&item_type).unwrap() += *amt as f32;
                        item_types.push(*item_type);
                    }
                }
            }
            self.completed.push((true, item_types));
        }
        self.transactions.clear();
    }

    /// Checks if the transaction is possible or not, assuming that the value of the resource cannot be less
    /// than zero.
    fn is_possible(&self, item_type: &ID, transaction: &Transaction) -> bool {
        if !self.items.contains_key(item_type) {
            false
        } else {
            let value = self.items.get(item_type);
            match transaction {
                Transaction::Change(amt) => {
                    if value.unwrap() + *amt as f32 >= 0.0 {
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }

    /// Gets the value of the item based on its ID.
    pub fn get_value_by_itemtype(&self, item_type: &ID) -> f32 {
        *self.items.get(item_type).unwrap()
    }
}

/// A transaction holding the amount the value should change by.
#[derive(Clone, Copy)]
pub enum Transaction {
    Change(f32),
}
