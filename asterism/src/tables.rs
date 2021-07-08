//! Query/condition tables: [](https://www.dataorienteddesign.com/dodmain/node7.html#SECTION00710000000000000000) and [](https://www.dataorienteddesign.com/dodmain/node12.html); it's in C but the ideas are similar.
//!
//! A query table asks a question of a table. Our tables in asterism are the identities/syntheses, and the events; ex. for a collision logic the identities could be the positions/sizes/metadata of each collision body, while the events would be contacts.
//!
//! A condition table composes those individual queries together.
use anycollections::AnyHashMap;
use std::collections::HashMap;
use std::hash::Hash;

/// Builds a query table over each "unit" of a logic.
///
/// kind of weird, I don't like the reallocations but I don't think it's worse than what I'm doing in the engines with building syntheses every frame.
pub trait QueryTable<QueryOver> {
    type ProcessOutput;
    fn check_predicate(&self, predicate: impl Fn(&QueryOver) -> bool) -> Vec<Self::ProcessOutput>;
}

pub struct ConditionTables<QueryID: Hash + Eq + Copy> {
    query_output: AnyHashMap<QueryID>,
    composes: HashMap<QueryID, Option<Compose<QueryID>>>,
}

impl<QueryID: Hash + Eq + Copy> ConditionTables<QueryID> {
    pub fn new() -> Self {
        Self {
            query_output: AnyHashMap::new(),
            composes: HashMap::new(),
        }
    }

    pub fn add_query<T: 'static>(&mut self, id: QueryID, compose: Option<Compose<QueryID>>) {
        let output: Vec<T> = Vec::new();
        self.query_output.insert(id, Box::new(output));
        self.composes.insert(id, compose);
    }

    pub fn update_single<T: 'static>(&mut self, id: QueryID, output: Vec<T>) -> Result<&[T], ()> {
        let query = self.composes.get(&id).ok_or(())?;
        if query.is_none() {
            let query_output = self.query_output.get_mut(&id).unwrap();
            *query_output = output;
            Ok(query_output.as_slice())
        } else {
            Err(())
        }
    }

    pub fn update_filter<T: Clone + 'static>(
        &mut self,
        id: QueryID,
        predicate: impl Fn(&T) -> bool,
    ) -> Result<&[T], ()> {
        // please don't ask questions about this line of code
        let query = self.composes.get(&id).ok_or(())?.as_ref().ok_or(())?;
        match query {
            Compose::Filter(other_id) => {
                let prev_output = self.query_output.get::<Vec<T>>(&other_id).ok_or(())?;
                let output = prev_output
                    .iter()
                    .cloned()
                    .filter(predicate)
                    .collect::<Vec<T>>();
                let query_output = self.query_output.get_mut(&id).ok_or(())?;
                *query_output = output;
                Ok(query_output.as_slice())
            }
            _ => Err(()),
        }
    }

    pub fn update_zip<A: Clone + 'static, B: Clone + 'static>(
        &mut self,
        id: QueryID,
    ) -> Result<&[(A, B)], ()> {
        let query = self.composes.get(&id).ok_or(())?.as_ref().ok_or(())?;
        match query {
            Compose::Zip(id_1, id_2) => {
                let zip_1 = self.query_output.get::<Vec<A>>(&id_1).ok_or(())?;
                let zip_2 = self.query_output.get::<Vec<B>>(&id_2).ok_or(())?;

                let output = zip_1
                    .iter()
                    .cloned()
                    .zip(zip_2.iter().cloned())
                    .collect::<Vec<(A, B)>>();

                let query_output = self.query_output.get_mut(&id).ok_or(())?;
                *query_output = output;
                Ok(query_output.as_slice())
            }
            _ => Err(()),
        }
    }
}

/// Possible ways to compose queries
#[non_exhaustive]
#[derive(Clone)]
pub enum Compose<QueryID: Copy> {
    Filter(QueryID),
    Zip(QueryID, QueryID),
}

/*
/// Possible ways to deal with the output of queries
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum Process {
    IfAny,
    IfNone,
    IfLength(usize, Compare),
}
*/

pub struct Condition<QueryID: Copy> {
    pub compose: Compose<QueryID>,
    pub output: Vec<bool>,
}
