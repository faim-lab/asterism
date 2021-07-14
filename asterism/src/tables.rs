//! Existence-based processing operates over lists, filtering/zipping/otherwise processing them, kind of like an [iterator][std::iter::Iterator]. More here: https://dataorienteddesign.com/dodmain/node4.html
//!
//! This is adjacent to/uses similar concepts as query/condition tables: https://www.dataorienteddesign.com/dodmain/node7.html#SECTION00710000000000000000 and https://www.dataorienteddesign.com/dodmain/node12.html
//!
//! The outputs in asterism are the identities + syntheses and the events. Ex. for a collision logic the identities could be the positions/sizes/metadata of each collision body, while the events would be contacts.
//!
//! A condition table composes those individual queries together.
use std::collections::HashMap;
use std::hash::Hash;

/// Builds output tables based on output of logics
pub trait OutputTable<ProcessOutput> {
    fn get_table(&self) -> Vec<ProcessOutput>;
}

/// holds logics' output tables and outputs of [processing][Compose]. Each compose processes one or two previous queries' output, then outputs them into another table. Ex: where `composes.get(&query3) == Compose::Zip(query1, query2)`, `query_output.get(&query3)` would be `query1` and `query2`'s output zipped together and copied to a new table. You could then further filter on the output of query3: `Compose::Filter(query3)`.
///
/// performance: does a lot of copying/reallocating every function call. works for now but might want to revisit in the future
pub struct ConditionTables<QueryID: Hash + Eq + Copy> {
    query_output: AnyHashMap<QueryID>,
    composes: HashMap<QueryID, Option<Compose<QueryID>>>,
}

impl<QueryID: Hash + Eq + Copy + std::fmt::Debug> ConditionTables<QueryID> {
    pub fn new() -> Self {
        Self {
            query_output: AnyHashMap::new(),
            composes: HashMap::new(),
        }
    }

    pub fn add_query<T: 'static>(&mut self, id: QueryID, compose: Option<Compose<QueryID>>) {
        let output: Vec<T> = Vec::new();
        self.query_output.insert(id, output);
        self.composes.insert(id, compose);
    }

    pub fn update_single<T: 'static>(
        &mut self,
        id: QueryID,
        output: Vec<T>,
    ) -> Result<&[T], TableError<QueryID>> {
        let query = self.composes.get(&id).ok_or(TableError::ComposeNotFound)?;
        if query.is_none() {
            let query_output = self.query_output.get_mut(&id)?;
            *query_output = output;
            Ok(query_output.as_slice())
        } else {
            Err(TableError::MismatchedQueryAction)
        }
    }

    pub fn update_filter<T: Clone + 'static>(
        &mut self,
        id: QueryID,
        predicate: impl Fn(&T) -> bool,
    ) -> Result<&[T], TableError<QueryID>> {
        let query = self.composes.get(&id).ok_or(TableError::ComposeNotFound)?;
        let query = query.as_ref().ok_or(TableError::MismatchedQueryAction)?;
        match query {
            Compose::Filter(other_id) => {
                let prev_output = self.query_output.get::<Vec<T>>(other_id)?;
                let output = prev_output
                    .iter()
                    .cloned()
                    .filter(predicate)
                    .collect::<Vec<T>>();
                let query_output = self.query_output.get_mut(&id)?;
                *query_output = output;
                Ok(query_output.as_slice())
            }
            _ => Err(TableError::MismatchedQueryAction),
        }
    }

    pub fn update_zip<A: Clone + 'static, B: Clone + 'static>(
        &mut self,
        id: QueryID,
    ) -> Result<&[(A, B)], TableError<QueryID>> {
        let query = self.composes.get(&id).ok_or(TableError::ComposeNotFound)?;
        let query = query.as_ref().ok_or(TableError::MismatchedQueryAction)?;
        match query {
            Compose::Zip(id_1, id_2) => {
                let zip_1 = self.query_output.get::<Vec<A>>(id_1)?;
                let zip_2 = self.query_output.get::<Vec<B>>(id_2)?;

                let output = zip_1
                    .iter()
                    .cloned()
                    .zip(zip_2.iter().cloned())
                    .collect::<Vec<(A, B)>>();

                let query_output = self.query_output.get_mut(&id)?;
                *query_output = output;
                Ok(query_output.as_slice())
            }
            _ => Err(TableError::MismatchedQueryAction),
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

pub struct Condition<QueryID: Copy> {
    pub compose: Compose<QueryID>,
    pub output: Vec<bool>,
}

#[derive(Debug)]
pub enum TableError<QueryID: std::fmt::Debug> {
    ComposeNotFound,
    QueryNotFound(QueryID),
    MismatchedQueryAction,
    MismatchedTypes(QueryID),
}

use std::any::TypeId;

/// Wrapper around `anycollections::AnyHashMap` that does typechecking at runtime so you don't have to worry about accidentally transmuting something you shouldn't have and causing undefined behavior. NOTE that `get()` and `get_mut()` in `anycollections::AnyHashMap` are very unsafe, but aren't marked as such.
///
/// The double lookups aren't ideal but work for now. Something like `HashMap/BTreeMap<ID, (TypeId, Box<something something UnsafeAny>)>` (????) would be better performance-wise but I can't be bothered to write that at the moment. (📌 unsafe-any: https://docs.rs/unsafe-any/0.4.2/unsafe_any/)
struct AnyHashMap<ID: Hash + Eq> {
    map: anycollections::AnyHashMap<ID>,
    /// `types_map.get(id)` *must* always be the `TypeId` of the type of map.get(id)'s output, otherwise very unsafe things will happen.
    types_map: HashMap<ID, TypeId>,
}

impl<ID> AnyHashMap<ID>
where
    ID: Hash + Eq + Copy + std::fmt::Debug,
{
    fn new() -> Self {
        Self {
            map: anycollections::AnyHashMap::new(),
            types_map: HashMap::new(),
        }
    }

    fn get<T: 'static>(&self, key: &ID) -> Result<&T, TableError<ID>> {
        if let Some(type_id) = self.types_map.get(key) {
            if *type_id == TypeId::of::<T>() {
                return self
                    .map
                    .get::<T>(key)
                    .ok_or(TableError::QueryNotFound(*key));
            }
        }
        Err(TableError::MismatchedTypes(*key))
    }

    fn get_mut<T: 'static>(&mut self, key: &ID) -> Result<&mut T, TableError<ID>> {
        if let Some(type_id) = self.types_map.get(key) {
            if *type_id == TypeId::of::<T>() {
                return self
                    .map
                    .get_mut::<T>(key)
                    .ok_or(TableError::QueryNotFound(*key));
            }
        }
        Err(TableError::MismatchedTypes(*key))
    }

    fn insert<T: 'static>(&mut self, key: ID, value: T) {
        self.types_map.insert(key, TypeId::of::<T>());
        self.map.insert(key, value);
    }
}
