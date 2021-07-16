use asterism::tables::*;

use crate::types::*;
use crate::{Game, Logics, State};

pub struct Predicate<Ev: PaddlesEvent> {
    pub id: UserQueryID,
    pub predicate: Ev,
    pub reaction: Box<dyn Fn(&mut State, &mut Logics, &Ev::AsterEvent)>,
}

type PredicateFn<Ev> = Box<dyn Fn(&mut State, &mut Logics, &<Ev as PaddlesEvent>::AsterEvent)>;

impl Game {
    pub fn add_ctrl_query(
        &mut self,
        predicate: CtrlEvent,
        reaction: PredicateFn<CtrlEvent>,
    ) -> UserQueryID {
        let id = self.add_query();
        self.events.control.push(Predicate {
            predicate,
            id,
            reaction,
        });
        self.tables.add_query::<CtrlEvent>(
            QueryType::User(id),
            Some(Compose::Filter(QueryType::CtrlEvent)),
        );
        id
    }

    pub fn add_collision_query(
        &mut self,
        predicate: ColEvent,
        reaction: PredicateFn<ColEvent>,
    ) -> UserQueryID {
        let id = self.add_query();
        self.events.collision.push(Predicate {
            predicate,
            id,
            reaction,
        });
        self.tables.add_query::<AColEvent>(
            QueryType::User(id),
            Some(Compose::Filter(QueryType::ColEvent)),
        );
        id
    }

    pub fn add_rsrc_query(
        &mut self,
        predicate: RsrcEvent,
        reaction: PredicateFn<RsrcEvent>,
    ) -> UserQueryID {
        let id = self.add_query();
        self.events.resources.push(Predicate {
            predicate,
            id,
            reaction,
        });
        self.tables.add_query::<ARsrcEvent>(
            QueryType::User(id),
            Some(Compose::Filter(QueryType::RsrcEvent)),
        );
        id
    }

    pub fn add_rsrc_ident_query(
        &mut self,
        predicate: RsrcIdent,
        reaction: PredicateFn<RsrcIdent>,
    ) -> UserQueryID {
        let id = self.add_query();
        self.events.resource_ident.push(Predicate {
            predicate,
            id,
            reaction,
        });
        self.tables.add_query::<ARsrcIdent>(
            QueryType::User(id),
            Some(Compose::Filter(QueryType::RsrcIdent)),
        );
        id
    }
}
