//! # Resource Logics
//!
//! Resource logics communicate that generic or specific resources can be created, destroyed, converted, or transferred between abstract or concrete locations. They create, destroy, and exchange (usually) discrete quantities of generic or specific resources in or between abstract or concrete locations on demand or over time, and trigger other actions when these transactions take place.

use crate::{tables::OutputTable, Event, EventType, Logic, Reaction};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::ops::{Add, AddAssign};

/// A resource logic that queues transactions, then applies them all at once when updating.
pub struct QueuedResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Add<Output = Value> + AddAssign + Ord + Copy,
{
    /// The items involved, and their values.
    pub items: BTreeMap<ID, (Value, Value, Value)>, // value, min, max
    /// Each transaction is a list of items involved in the transaction and the amount they're being changed.
    pub transactions: Vec<(ID, Transaction<Value>)>,
    /// A Vec of all transactions and if they were able to be completed or not. If yes, supply a Vec of the IDs of successful transactions; if no, supply the ID of the pool that caused the error and a reason (see [ResourceError]).
    pub completed: Vec<Result<ID, (ID, ResourceError)>>,
}

impl<ID, Value> Logic for QueuedResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Add<Output = Value> + AddAssign + Ord + Copy,
{
    type Event = ResourceEvent<ID>;
    type Reaction = ResourceReaction<ID, Value>;

    type Ident = ID;
    type IdentData = (Value, Value, Value);

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        self.transactions.push(*reaction);
    }

    fn get_ident_data(&self, ident: Self::Ident) -> Self::IdentData {
        *self
            .items
            .get(&ident)
            .unwrap_or_else(|| panic!("requested pool {:?} doesn't exist in resource logic", ident))
    }

    fn update_ident_data(&mut self, ident: Self::Ident, data: Self::IdentData) {
        let vals = self
            .items
            .get_mut(&ident)
            .unwrap_or_else(|| panic!("pool {:?} not found", ident));
        *vals = data;
    }
}

impl<ID, Value> QueuedResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Add<Output = Value> + AddAssign + Ord + Copy,
{
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            transactions: Vec::new(),
            completed: Vec::new(),
        }
    }

    /// Updates the values of resources based on the queued transactions. If a transaction cannot be completed (if the value goes below its min or max), a snapshot of the resources before the transaction occurred is restored, and the transaction is marked as incomplete, and we continue to process the remaining transactions.
    pub fn update(&mut self) {
        self.completed.clear();

        for exchange in self.transactions.iter() {
            let (item_type, change) = exchange;

            if let Err(err) = self.is_possible(item_type, change) {
                self.completed.push(Err(err));
                continue;
            }

            let (val, min, max) = self.items.get_mut(item_type).unwrap();
            match change {
                Transaction::Change(amt) => {
                    *val += *amt;
                }
                Transaction::Set(amt) => {
                    *val = *amt;
                }
                Transaction::SetMax(new_max) => {
                    *max = *new_max;
                }
                Transaction::SetMin(new_min) => {
                    *min = *new_min;
                }
            }
            self.completed.push(Ok(*item_type));
        }
        self.transactions.clear();
    }

    /// Checks if the transaction is possible or not
    fn is_possible(
        &self,
        item_type: &ID,
        transaction: &Transaction<Value>,
    ) -> Result<(), (ID, ResourceError)> {
        if let Some((value, min, max)) = self.items.get(item_type) {
            match transaction {
                Transaction::Change(amt) => {
                    if *value + *amt > *max {
                        Err((*item_type, ResourceError::TooBig))
                    } else if *value + *amt < *min {
                        Err((*item_type, ResourceError::TooSmall))
                    } else {
                        Ok(())
                    }
                }
                _ => Ok(()),
            }
        } else {
            Err((*item_type, ResourceError::PoolNotFound))
        }
    }

    /// Gets the value of the item based on its ID.
    pub fn get_value_by_itemtype(&self, item_type: &ID) -> Option<Value> {
        self.items.get(item_type).map(|(val, ..)| *val)
    }
}

/// A transaction holding the amount the value should change by.
#[derive(Clone, Copy)]
pub enum Transaction<Value>
where
    Value: Add + AddAssign,
{
    Change(Value),
    Set(Value),
    SetMax(Value),
    SetMin(Value),
}

/// Errors possible when trying to complete a transaction.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ResourceError {
    PoolNotFound,
    TooBig,
    TooSmall,
}

pub type ResourceReaction<ID, Value> = (ID, Transaction<Value>);

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ResourceEvent<ID> {
    pub pool: ID,
    pub event_type: ResourceEventType,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ResourceEventType {
    PoolUpdated,
    TransactionUnsuccessful(ResourceError),
}

impl EventType for ResourceEventType {}

impl<ID: Ord, Value: Add + AddAssign> Reaction for ResourceReaction<ID, Value> {}

impl<ID: Ord> Event for ResourceEvent<ID> {
    type EventType = ResourceEventType;
    fn get_type(&self) -> &Self::EventType {
        &self.event_type
    }
}

type QueryIdent<ID, Value> = (
    <QueuedResources<ID, Value> as Logic>::Ident,
    <QueuedResources<ID, Value> as Logic>::IdentData,
);

impl<ID, Value> OutputTable<QueryIdent<ID, Value>> for QueuedResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Add<Output = Value> + AddAssign + Ord + Copy,
{
    fn get_table(&self) -> Vec<QueryIdent<ID, Value>> {
        self.items
            .keys()
            .map(|id| (*id, self.get_ident_data(*id)))
            .collect()
    }
}

impl<ID, Value> OutputTable<ResourceEvent<ID>> for QueuedResources<ID, Value>
where
    ID: Copy + Ord + Debug,
    Value: Add<Output = Value> + AddAssign + Ord + Copy,
{
    fn get_table(&self) -> Vec<ResourceEvent<ID>> {
        self.completed
            .iter()
            .map(|completed| match completed {
                Ok(id) => ResourceEvent {
                    pool: *id,
                    event_type: ResourceEventType::PoolUpdated,
                },
                Err((id, err)) => ResourceEvent {
                    pool: *id,
                    event_type: ResourceEventType::TransactionUnsuccessful(*err),
                },
            })
            .collect()
    }
}
