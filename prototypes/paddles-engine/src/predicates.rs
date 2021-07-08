use asterism::tables::*;

use crate::types::*;
use crate::{Game, Logics, State};

pub struct Predicate<Ev: PaddlesEvent> {
    pub id: QueryID,
    pub predicate: Ev,
}

impl Game {
    // need to process compose at some point to know where to execute it
    pub fn add_compose<T: 'static>(
        &mut self,
        compose: Compose<QueryID>,
        reaction: Box<dyn Fn(&mut State, &mut Logics, &dyn std::any::Any)>,
    ) -> QueryID {
        let id = QueryID::new(self.events.queries_max_id);
        self.events.queries_max_id += 1;
        let queries = Vec::with_capacity(2);
        match compose {
            Compose::Filter(filter_id) => queries.push(filter_id),
            Compose::Zip(zip_1, zip_2) => {
                queries.push(zip_1);
                queries.push(zip_2);
            }
        }

        // update logics in this order
        let mut control = false;
        let mut physics = false;
        let mut collision = false;
        let mut resources = false;
        let mut rsrc_ident = false;

        for query in queries.iter() {
            control = self.events.control.iter().any(|ctrl| ctrl.id == *query);
            physics = self.events.physics.iter().any(|phys| phys.id == *query);
            collision = self.events.collision.iter().any(|col| col.id == *query);
            resources = self.events.resources.iter().any(|rsrc| rsrc.id == *query);
            rsrc_ident = self
                .events
                .resource_ident
                .iter()
                .any(|rsrc| rsrc.id == *query);
        }

        if rsrc_ident || resources {
            self.events.stages.resources.push(id);
        } else if collision {
            self.events.stages.collision.push(id);
        } else if physics {
            self.events.stages.physics.push(id);
        } else if control {
            self.events.stages.control.push(id);
        } else {
            // the reaction should go into one of the above stages but if not, resources serves as a catch-all at the end of the update loop
            self.events.stages.resources.push(id);
        }

        self.events.reactions.insert(id, reaction);
        self.table.add_query::<T>(id, Some(compose));

        id
    }

    pub fn add_ctrl_query(&mut self, predicate: CtrlEvent) -> QueryID {
        let id = QueryID::new(self.events.queries_max_id);
        self.events.queries_max_id += 1;
        self.events.control.push(Predicate { predicate, id });
        self.table.add_query::<CtrlEvent>(id, None);
        id
    }

    pub fn add_collision_query(&mut self, predicate: ColEvent) -> QueryID {
        let id = QueryID::new(self.events.queries_max_id);
        self.events.queries_max_id += 1;
        self.events.collision.push(Predicate { predicate, id });
        self.table.add_query::<(usize, usize)>(id, None);
        id
    }

    pub fn add_rsrc_query(&mut self, predicate: RsrcEvent) -> QueryID {
        let id = QueryID::new(self.events.queries_max_id);
        self.events.queries_max_id += 1;
        self.events.resources.push(Predicate { predicate, id });
        self.table.add_query::<ARsrcEvent>(id, None);
        id
    }

    pub fn add_rsrc_ident_query(&mut self, predicate: RsrcIdent) -> QueryID {
        let id = QueryID::new(self.events.queries_max_id);
        self.events.queries_max_id += 1;
        self.events.resource_ident.push(Predicate { predicate, id });
        self.table.add_query::<RsrcPool>(id, None);
        id
    }
}
