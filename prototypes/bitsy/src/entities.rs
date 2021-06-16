use crate::*;

impl Game {
    pub fn add_ctrl_predicate(
        &mut self,
        action: ActionID,
        key_event: ControlEventType,
        on_key_event: Box<dyn Fn(&mut State, &mut Logics, &CtrlEvent)>,
    ) {
        let key_event = CtrlEvent {
            event_type: key_event,
            action_id: action,
            set: 0,
        };
        self.events.control.push((key_event, on_key_event));
    }

    pub fn add_link(&mut self, from: (usize, IVec2), to: (usize, IVec2)) -> LinkID {
        let link_from;
        if let Some(i) = self.state.links.iter().position(|pos| from == *pos) {
            link_from = LinkID::new(i);
        } else {
            link_from = LinkID::new(self.state.link_id_max);
            self.state.link_ids.push(link_from);
            self.state.link_id_max += 1;
            self.state.links.push(from);
            self.logics.linking.graphs[0].add_node(link_from);
        }

        let link_to;
        if let Some(i) = self.state.links.iter().position(|pos| to == *pos) {
            link_to = LinkID::new(i);
        } else {
            link_to = LinkID::new(self.state.link_id_max);
            self.state.link_ids.push(link_to);
            self.state.link_id_max += 1;
            self.state.links.push(to);
            self.logics.linking.graphs[0].add_node(link_to);
        }
        self.logics.linking.graphs[0].add_edge(link_from.idx(), link_to.idx());

        self.add_collision_predicate(
            Contact::Tile(0, from.1),
            from.0,
            Box::new(
                move |state: &mut State, logics: &mut Logics, _: &ColEvent| {
                    let idx = state
                        .link_ids
                        .iter()
                        .position(|id| *id == link_from)
                        .unwrap();
                    logics
                        .linking
                        .handle_predicate(&LinkingReaction::Traverse(0, idx));

                    let idx = state.link_ids.iter().position(|id| *id == link_to).unwrap();

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
                    let idx = state.link_ids.iter().position(|id| *id == link_to).unwrap();
                    let (room, pos) = state.links[idx];
                    set_current_room(state, logics, room);
                    logics.collision.clear_entities();
                    // player position
                    logics.collision.positions.push(pos);
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

    pub fn add_link_predicate(
        &mut self,
        from: LinkID,
        to: LinkID,
        when_traversed: Box<dyn Fn(&mut State, &mut Logics, &LinkingEvent)>,
    ) {
        let from_idx = self
            .state
            .link_ids
            .iter()
            .position(|id| *id == from)
            .unwrap();
        let room = self.state.links[from_idx].0;

        let event = LinkingEvent {
            graph: 0,
            node: self.state.link_ids.iter().position(|id| *id == to).unwrap(),
            event_type: LinkingEventType::Traversed,
        };
        self.events.predicates[room]
            .linking
            .push((event, when_traversed));
    }

    pub fn add_collision_predicate(
        &mut self,
        col_event: ColEvent,
        room: usize,
        on_collide: Box<dyn Fn(&mut State, &mut Logics, &ColEvent)>,
    ) {
        self.events.predicates[room]
            .collision
            .push((col_event, on_collide));
    }

    pub fn add_rsrc_predicate(
        &mut self,
        pool: RsrcID,
        rsrc_event: ResourceEventType,
        on_rsrc_event: Box<dyn Fn(&mut State, &mut Logics, &RsrcEvent)>,
    ) {
        let rsrc_event = RsrcEvent {
            pool,
            event_type: rsrc_event,
        };
        self.events.resources.push((rsrc_event, on_rsrc_event));
    }

    pub fn set_background(&mut self, color: Color) {
        self.colors.background_color = color;
    }

    pub fn set_player(&mut self, player: Player) {
        self.colors.colors.insert(EntID::Player, player.color);
        self.logics.consume_player(player, !self.state.player);

        if !self.state.player {
            for room in self.events.predicates.iter_mut() {
                for (col_event, _) in room.collision.iter_mut() {
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
        }

        self.state.player = true;
    }

    pub fn add_character(&mut self, character: Character) -> CharacterID {
        let id = CharacterID::new(self.state.char_id_max);
        self.colors
            .colors
            .insert(EntID::Character(id), character.color);

        self.logics.consume_character(character);

        for room in self.events.predicates.iter_mut() {
            for (col_event, _) in room.collision.iter_mut() {
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

    pub fn set_num_rooms(&mut self, rooms: usize) {
        self.state.rooms.resize_with(rooms, Room::default);
        self.events.predicates.resize_with(rooms, Predicates::new);
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
        self.events.predicates.push(Predicates::new());

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
        for room in self.events.predicates.iter_mut() {
            remove.clear();
            for (idx, (col_event, _)) in room.collision.iter_mut().enumerate() {
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
                let _ = room.collision.remove(*i);
            }
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
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveEnt(
                self.state.get_col_idx(ent_i, CollisionEnt::Character),
            ));

        let mut remove = Vec::new();
        for room in self.events.predicates.iter_mut() {
            remove.clear();
            for (idx, (col_event, _)) in room.collision.iter_mut().enumerate() {
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
            for i in remove.iter().rev() {
                let _ = room.collision.remove(*i);
            }
        }
        self.state.characters.remove(ent_i);
    }

    // unsure if this is needed atm
    pub fn remove_tile_at_pos(&mut self, room: usize, pos: IVec2) {
        self.state.rooms[room].map[pos.y as usize][pos.x as usize] = None;
        // is current_room supposed to be represented through a linking logic?
        if room == self.state.current_room {
            self.logics
                .collision
                .handle_predicate(&CollisionReaction::RemoveTileAtPos(pos));
        }

        let mut remove = Vec::new();
        for room in self.events.predicates.iter_mut() {
            remove.clear();
            for (idx, (col_event, _)) in room.collision.iter_mut().enumerate() {
                if let ColEvent::Tile(_, ev_pos) = col_event {
                    if pos == *ev_pos {
                        remove.push(idx);
                    }
                }
            }
            for i in remove.iter() {
                let _ = room.collision.remove(*i);
            }
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
        for (idx, (rsrc_event, _)) in self.events.resources.iter().enumerate() {
            if rsrc == rsrc_event.pool {
                remove.push(idx);
            }
        }
        for i in remove.into_iter() {
            let _ = self.events.resources.remove(i);
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

        for (act_id, keycode, valid, _) in player.controls {
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

pub fn set_current_room(state: &mut State, logics: &mut Logics, room: usize) {
    dbg!(state.rooms.len());
    for (row, col_row) in state.rooms[state.current_room]
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

    for (row, col_row) in state.rooms[room]
        .map
        .iter()
        .zip(logics.collision.map.iter_mut())
    {
        for (tile, col_tile) in row.iter().zip(col_row.iter_mut()) {
            *col_tile = *tile;
        }
    }
    state.current_room = room;
}
