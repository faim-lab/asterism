use crate::*;

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
        let event = LinkingEvent {
            graph: 0,
            node: self.logics.linking.graphs[0].graph.node_idx(&to).unwrap(),
            event_type: LinkingEventType::Traversed(from.idx()),
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
        self.tables.add_query::<ColEvent>(
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

    pub fn set_background(&mut self, color: Color) {
        self.colors.background_color = color;
    }

    pub fn set_player(&mut self, player: Player) {
        self.colors.colors.insert(EntID::Player, player.color);
        self.logics.consume_player(player, !self.state.player);

        if !self.state.player {
            for (_, (col_event, _), _) in self.events.collision.iter_mut() {
                match col_event {
                    ColEvent::Ent(i, j) => {
                        *i += 1;
                        *j += 1;
                    }
                    ColEvent::Tile(i, _) => {
                        *i += 1;
                    }
                }
            }
        }

        self.state.player = true;
    }

    pub fn add_character(&mut self, character: Character) -> CharacterID {
        let id = CharacterID::new(self.state.char_id_max);
        self.colors
            .colors
            .insert(EntID::Character(id), character.color);

        self.logics.consume_character(character);

        for (_, (col_event, _), _) in self.events.collision.iter_mut() {
            match col_event {
                ColEvent::Ent(i, j) => {
                    if *i <= id.idx() {
                        *i += 1;
                    }
                    if *j <= id.idx() {
                        *j += 1;
                    }
                }
                ColEvent::Tile(i, _) => {
                    if *i <= id.idx() {
                        *i += 1;
                    }
                }
            }
        }

        self.state.char_id_max += 1;
        self.state.characters.push(id);
        id
    }

    pub fn log_rsrc(&mut self) -> RsrcID {
        let id = RsrcID::new(self.state.rsrc_id_max);
        self.state.rsrc_id_max += 1;
        self.state.resources.push(id);
        id
    }

    pub fn add_link(&mut self, from: (usize, IVec2), to: (usize, IVec2)) -> LinkID {
        let mut find = |id| match self
            .state
            .links
            .iter()
            .find(|(_, location)| **location == id)
        {
            Some((id, _)) => *id,
            None => {
                let id = LinkID::new(self.state.link_id_max);
                self.logics.linking.graphs[0].add_node(id);
                self.state.link_id_max += 1;
                self.state.links.insert(id, to);
                id
            }
        };
        let link_from = find(from);
        let link_to = find(to);

        self.logics.linking.graphs[0]
            .graph
            .add_edge(link_from.idx(), link_to.idx());

        self.add_collision_predicate(
            Contact::Tile(0, from.1),
            from.0,
            Box::new(
                move |_: &mut State, logics: &mut Logics, (_, room): &(ColEvent, usize)| {
                    let idx = logics.linking.graphs[0].graph.node_idx(&link_from).unwrap();
                    logics
                        .linking
                        .handle_predicate(&LinkingReaction::Traverse(0, idx));

                    let idx = logics.linking.graphs[0]
                        .graph
                        .node_idx(&LinkID::new(*room))
                        .unwrap();
                    logics
                        .linking
                        .handle_predicate(&LinkingReaction::Activate(0, idx));
                },
            ),
        );

        self.add_link_predicate(
            link_from,
            link_to,
            Box::new(
                move |state: &mut State, logics: &mut Logics, _: &LinkingEvent| {
                    set_current_room(state, logics, from.0, to.0);
                    logics.collision.clear_entities();
                    // player position
                    logics.collision.positions.push(to.1);
                    logics.collision.amt_moved.push(IVec2::ZERO);
                    logics.collision.metadata.push(CollisionData::new(
                        true,
                        false,
                        CollisionEnt::Player,
                    ));
                },
            ),
        );
        link_from
    }

    pub fn set_num_rooms(&mut self, rooms: usize) {
        self.state.rooms.resize_with(rooms, Room::default);
    }

    /// Loads a tilemap with maximum 10 different kinds of tiles (numbers 0-9). A space (' ') marks a place on the map without any tiles. The tile types are read from in the parameter `tiles`.
    ///
    /// # Example
    ///
    /// ```
    /// let map = r#"
    /// 0000000
    /// 0     0
    /// 0   1 0
    /// 0 1   0
    /// 0   2 0
    /// 0     0
    /// 0     0
    /// 0000000
    /// "#;
    ///
    /// game.add_room_from_str(map);
    /// ```
    pub fn add_room_from_str(&mut self, map: &str) -> Result<usize, String> {
        let room = self.state.rooms.len();
        self.state.rooms.push(Room::default());

        let map = map.trim();

        let map_length = WORLD_SIZE * WORLD_SIZE + WORLD_SIZE - 1;
        #[allow(clippy::comparison_chain)]
        if map.len() > map_length {
            return Err("map is too big".to_string());
        } else if map.len() < map_length {
            return Err("map is too small".to_string());
        }

        let mut x = 0;
        let mut y = 0;

        for ch in map.chars() {
            if ch.is_digit(10) {
                let tile_idx = ch.to_string().parse::<usize>().unwrap();
                if tile_idx > self.state.tile_type_count {
                    return Err(format!("tile {} not found", tile_idx));
                }
                self.add_tile_at_pos(TileID::new(tile_idx), room, IVec2::new(x as i32, y as i32));
                x += 1;
            } else if ch == ' ' {
                x += 1;
            } else if ch == '\n' {
                y += 1;
                x = 0;
            } else {
                return Err(format!("unrecognized character: '{}'", ch));
            }
        }

        Ok(self.state.rooms.len() - 1)
    }

    pub fn log_tile_info(&mut self, tile: Tile) -> TileID {
        let id = TileID::new(self.state.tile_type_count);
        self.state.tile_type_count += 1;
        self.colors.colors.insert(EntID::Tile(id), tile.color);

        self.logics.collision.tile_solid.insert(id, tile.solid);

        id
    }

    pub fn add_tile_at_pos(&mut self, tile: TileID, room: usize, pos: IVec2) {
        self.state.rooms[room].map[pos.y as usize][pos.x as usize] = Some(tile);
    }

    pub fn remove_player(&mut self) {
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveEnt(0));

        let mut remove = Vec::new();
        for (idx, (_, (col_event, _), _)) in self.events.collision.iter_mut().enumerate() {
            match col_event {
                ColEvent::Ent(i, j) => {
                    if *i == 0 {
                        remove.push(idx);
                    }
                    if *j == 0 {
                        remove.push(idx);
                    }
                    if *i > 0 {
                        *i -= 1;
                    }
                    if *j > 0 {
                        *j -= 1;
                    }
                }
                ColEvent::Tile(i, _) => {
                    if *i == 0 {
                        remove.push(idx);
                    }
                    if *i > 0 {
                        *i -= 1;
                    }
                }
            }
        }

        for i in remove.iter().rev() {
            let _ = self.events.collision.remove(*i);
        }
        self.state.player = false;
    }

    pub fn remove_character(&mut self, character: CharacterID) {
        let ent_i = self
            .state
            .characters
            .iter()
            .position(|cid| *cid == character)
            .unwrap();
        // this assumes
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveEnt(
                self.state.get_col_idx(ent_i, CollisionEnt::Character),
            ));

        let node = self.logics.linking.graphs[0].get_current_node();
        let current_room = self.state.links.get(&node).unwrap().0;

        let mut remove = Vec::new();
        for (idx, (_, (col_event, room), _)) in self.events.collision.iter_mut().enumerate() {
            if *room == current_room {
                match col_event {
                    ColEvent::Ent(i, j) => {
                        if *i != 0 && *i - 1 == ent_i {
                            remove.push(idx);
                        }
                        if *j != 0 && *j - 1 == ent_i {
                            remove.push(idx);
                        }
                        if *i > ent_i {
                            *i -= 1;
                        }
                        if *j > ent_i {
                            *j -= 1;
                        }
                    }
                    ColEvent::Tile(i, _) => {
                        if *i != 0 && *i - 1 == ent_i {
                            remove.push(idx);
                        }
                        if *i > ent_i {
                            *i -= 1;
                        }
                    }
                }
            }
        }
        for i in remove.iter().rev() {
            let _ = self.events.collision.remove(*i);
        }
        self.state.characters.remove(ent_i);
    }

    // unsure if this is needed atm
    pub fn remove_tile_at_pos(&mut self, room: usize, pos: IVec2) {
        self.state.rooms[room].map[pos.y as usize][pos.x as usize] = None;
        let node = self.logics.linking.graphs[0].get_current_node();
        let current_room = self.state.links.get(&node).unwrap().0;

        if room == current_room {
            self.logics
                .collision
                .handle_predicate(&CollisionReaction::RemoveTileAtPos(pos));
        }

        let mut remove = Vec::new();
        for (idx, (_, (col_event, event_room), _)) in self.events.collision.iter_mut().enumerate() {
            if *event_room == room {
                if let ColEvent::Tile(_, ev_pos) = col_event {
                    if pos == *ev_pos {
                        remove.push(idx);
                    }
                }
            }
        }
        for i in remove.iter() {
            let _ = self.events.collision.remove(*i);
        }
    }

    pub fn remove_rsrc(&mut self, rsrc: RsrcID) {
        let ent_i = self
            .state
            .resources
            .iter()
            .position(|rid| *rid == rsrc)
            .unwrap();
        self.logics.resources.items.remove(&rsrc);

        let mut remove = Vec::new();
        for (idx, (_, rsrc_event, _)) in self.events.resource_event.iter().enumerate() {
            if rsrc == rsrc_event.pool {
                remove.push(idx);
            }
        }
        for i in remove.into_iter() {
            let _ = self.events.resource_event.remove(i);
        }
        self.state.resources.remove(ent_i);
    }
}

impl Logics {
    pub fn consume_player(&mut self, player: Player, new: bool) {
        if new {
            self.collision.positions.insert(0, player.pos);
            self.collision.amt_moved.insert(0, player.amt_moved);
            self.collision
                .metadata
                .insert(0, CollisionData::new(true, false, CollisionEnt::Player));
        } else {
            self.collision.positions[0] = player.pos;
            self.collision.amt_moved[0] = player.amt_moved;
            self.collision.metadata[0] = CollisionData::new(true, false, CollisionEnt::Player);
        }

        if !self.control.mapping.is_empty() {
            self.control.mapping[0].clear();
            self.control.values[0].clear();
        }

        for (act_id, keycode, valid) in player.controls {
            self.control.add_key_map(0, keycode, act_id, valid);
        }

        self.resources.items.clear();
        for (id, rsrc) in player.inventory.into_iter() {
            self.consume_rsrc(id, rsrc);
        }
    }

    pub fn consume_character(&mut self, character: Character) {
        self.collision.positions.push(character.pos);
        self.collision.amt_moved.push(IVec2::ZERO);
        self.collision
            .metadata
            .push(CollisionData::new(true, true, CollisionEnt::Character));
    }

    pub fn consume_rsrc(&mut self, id: RsrcID, rsrc: Resource) {
        self.resources
            .items
            .insert(id, (rsrc.val, rsrc.min, rsrc.max));
    }
}

pub fn set_current_room(state: &mut State, logics: &mut Logics, from_room: usize, to_room: usize) {
    for (row, col_row) in state.rooms[from_room]
        .map
        .iter_mut()
        .zip(logics.collision.map.iter())
    {
        for (tile, col_tile) in row.iter_mut().zip(col_row.iter()) {
            *tile = *col_tile;
        }
    }

    logics
        .collision
        .clear_and_resize_map(WORLD_SIZE, WORLD_SIZE);

    for (row, col_row) in state.rooms[to_room]
        .map
        .iter()
        .zip(logics.collision.map.iter_mut())
    {
        for (tile, col_tile) in row.iter().zip(col_row.iter_mut()) {
            *col_tile = *tile;
        }
    }
}
