use crate::types::*;
use crate::{Game, Logics, State};
use asterism::{
    control::ControlEventType, linking::LinkingEvent, linking::LinkingEventType,
    resources::ResourceEventType, tables::Compose,
};

type PredicateFn<Event> = (
    UserQueryID,
    Event,
    Box<dyn Fn(&mut State, &mut Logics, &Event)>,
);

pub(crate) struct Events {
    max_query_count: usize,

    pub control: Vec<PredicateFn<CtrlEvent>>,
    pub collision: Vec<PredicateFn<(ColEvent, usize)>>, // usize is the current room number
    pub linking: Vec<PredicateFn<LinkingEvent>>,
    pub resource_event: Vec<PredicateFn<RsrcEvent>>,
    #[allow(clippy::type_complexity)]
    pub resource_ident: Vec<PredicateFn<(RsrcID, (u16, u16, u16))>>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            control: Vec::new(),
            collision: Vec::new(),
            linking: Vec::new(),
            resource_event: Vec::new(),
            resource_ident: Vec::new(),

            max_query_count: 0,
        }
    }

    pub fn add_query(&mut self) -> UserQueryID {
        let id = UserQueryID::new(self.max_query_count);
        self.max_query_count += 1;
        id
    }
}

impl Game {
    pub fn add_ctrl_predicate(
        &mut self,
        action: ActionID,
        key_event: ControlEventType,
        on_key_event: Box<dyn Fn(&mut State, &mut Logics, &CtrlEvent)>,
    ) {
        let query_id = self.events.add_query();
        self.tables.add_query::<CtrlEvent>(
            QueryType::User(query_id),
            Some(Compose::Filter(QueryType::ControlEvent)),
        );
        let key_event = CtrlEvent {
            event_type: key_event,
            action_id: action,
            set: 0,
        };
        self.events
            .control
            .push((query_id, key_event, on_key_event));
    }

    pub fn add_link_predicate(
        &mut self,
        from: LinkID,
        to: LinkID,
        when_traversed: Box<dyn Fn(&mut State, &mut Logics, &LinkingEvent)>,
    ) {
        let query_id = self.events.add_query();
        self.tables.add_query::<LinkingEvent>(
            QueryType::User(query_id),
            Some(Compose::Filter(QueryType::LinkingEvent)),
        );
        let to = self.logics.linking.graphs[0].graph.node_idx(&to).unwrap();
        let from = self.logics.linking.graphs[0].graph.node_idx(&from).unwrap();
        let event = LinkingEvent {
            graph: 0,
            node: to,
            event_type: LinkingEventType::Traversed(from),
        };

        self.events.linking.push((query_id, event, when_traversed));
    }

    #[allow(clippy::type_complexity)]
    pub fn add_collision_predicate(
        &mut self,
        col_event: ColEvent,
        room: usize,
        on_collide: Box<dyn Fn(&mut State, &mut Logics, &(ColEvent, usize))>,
    ) {
        let query_id = self.events.add_query();
        self.tables.add_query::<(ColEvent, (usize, LinkID))>(
            QueryType::User(query_id),
            Some(Compose::Filter(QueryType::ContactRoom)),
        );
        self.events
            .collision
            .push((query_id, (col_event, room), on_collide));
    }

    pub fn add_rsrc_predicate(
        &mut self,
        pool: RsrcID,
        rsrc_event: ResourceEventType,
        on_rsrc_event: Box<dyn Fn(&mut State, &mut Logics, &RsrcEvent)>,
    ) {
        let query_id = self.events.add_query();
        self.tables.add_query::<RsrcEvent>(
            QueryType::User(query_id),
            Some(Compose::Filter(QueryType::ResourceEvent)),
        );
        let rsrc_event = RsrcEvent {
            pool,
            event_type: rsrc_event,
        };
        self.events
            .resource_event
            .push((query_id, rsrc_event, on_rsrc_event));
    }
}
