use asterism::tables::*;

use crate::types::*;
use crate::{Game, Logics, State};

pub struct Predicate<Ev: PaddlesEvent> {
    pub id: QueryID,
    pub predicate: Ev,
}

impl Game {
    // need to process compose at some point to know where to execute it
    pub fn add_compose(
        &mut self,
        compose: Compose<QueryID>,
        reaction: Box<dyn Fn(&mut State, &mut Logics, &Compose<QueryID>)>,
    ) -> ConditionID {
        let mut queries = Vec::new();
        compose.extract_queries(&mut queries);

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

        let condition = self.table.add_condition(compose);

        if rsrc_ident || resources {
            self.events.stages.resources.push(condition);
        } else if collision {
            self.events.stages.collision.push(condition);
        } else if physics {
            self.events.stages.physics.push(condition);
        } else if control {
            self.events.stages.control.push(condition);
        } else {
            // the reaction should go into one of the above stages but if not, resources serves as a catch-all at the end of the update loop
            self.events.stages.resources.push(condition);
        }

        self.events.reactions.insert(condition, reaction);
        condition
    }

    pub fn add_ctrl_query(&mut self, predicate: CtrlEvent) -> QueryID {
        let id = QueryID::new(self.events.queries_max_id);
        self.events.queries_max_id += 1;
        self.events.control.push(Predicate { predicate, id });
        id
    }

    pub fn add_collision_query(&mut self, predicate: ColEvent) -> QueryID {
        let id = QueryID::new(self.events.queries_max_id);
        self.events.queries_max_id += 1;
        self.events.collision.push(Predicate { predicate, id });
        id
    }

    pub fn add_rsrc_query(&mut self, predicate: RsrcEvent) -> QueryID {
        let id = QueryID::new(self.events.queries_max_id);
        self.events.queries_max_id += 1;
        self.events.resources.push(Predicate { predicate, id });
        id
    }

    pub fn add_rsrc_ident_query(&mut self, predicate: RsrcIdent) -> QueryID {
        let id = QueryID::new(self.events.queries_max_id);
        self.events.queries_max_id += 1;
        self.events.resource_ident.push(Predicate { predicate, id });
        id
    }
}
