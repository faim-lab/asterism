use std::collections::BTreeMap;

pub struct QueuedResources<ID: Copy + Ord> {
    pub items: BTreeMap<ID, f32>,
    pub transactions: Vec<Vec<(ID, Transaction)>>,
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

    pub fn update(&mut self) {
        self.completed.clear();
        'exchange: for exchange in self.transactions.iter() {
            let mut snapshot: BTreeMap<ID, f32> = BTreeMap::new();
            for (item_type, ..) in exchange {
                snapshot.insert(*item_type, *self.items.get(&item_type).unwrap());
            }
            let mut item_types = vec![];
            for (item_type, change) in exchange.iter() {
                if !self.is_possible(item_type, change) {
                    item_types.push(*item_type);
                    self.completed.push((false, item_types.clone()));
                    for (item_type, val) in snapshot.iter() {
                        *self.items.get_mut(&item_type).unwrap() = *val;
                        continue 'exchange;
                    }
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

    pub fn get_value_by_itemtype(&self, item_type: &ID) -> f32 {
        *self.items.get(item_type).unwrap()
    }
}

#[derive(Clone, Copy)]
pub enum Transaction {
    Change(i8),
}
