use crate::*;

impl Game {
    pub fn add_ctrl_predicate(
        &mut self,
        player: PlayerID,
        action: ActionID,
        key_event: ControlEventType,
        on_key_event: Box<dyn Fn(&mut State, &mut Logics, &CtrlEvent)>,
    ) {
        let key_event = CtrlEvent {
            event_type: key_event,
            action_id: action,
            set: player.idx(),
        };
        self.events.control.push((key_event, on_key_event));
    }

    pub fn add_collision_predicate(
        &mut self,
        col_event: ColEvent,
        on_collide: Box<dyn Fn(&mut State, &mut Logics, &ColEvent)>,
    ) {
        self.events.collision.push((col_event, on_collide));
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

    pub fn set_player(&mut self, player: Player) -> PlayerID {
        let id = PlayerID::new(0);
        self.colors.colors.insert(EntID::Player(id), player.color);
        self.logics.consume_player(player);
        self.state.player = Some(id);
        id
    }

    pub fn add_character(&mut self, character: Character) -> CharacterID {
        let id = CharacterID::new(self.state.char_id_max);
        self.colors
            .colors
            .insert(EntID::Character(id), character.color);

        self.logics.consume_character(character);

        self.state.char_id_max += 1;
        self.state.characters.push(id);
        id
    }

    pub fn add_rsrc(&mut self, resource: Resource) -> RsrcID {
        let id = RsrcID::new(self.state.rsrc_id_max);
        self.logics.consume_rsrc(id, resource);
        self.state.rsrc_id_max += 1;
        self.state.resources.push(id);
        id
    }

    pub fn log_tile_info(&mut self, tile: Tile) -> TileID {
        let id = TileID::new(self.state.tile_type_count);
        self.state.tile_type_count += 1;
        self.colors.colors.insert(EntID::Tile(id), tile.color);

        self.logics.collision.tile_solid.insert(id, tile.solid);

        id
    }

    pub fn add_tile_at_pos(&mut self, room: usize, tile: TileID, pos: IVec2) {
        self.state.rooms[room].map[pos.y as usize][pos.x as usize] = Some(tile);
    }

    pub fn set_current_room(&mut self, room: usize) {
        self.state.current_room = room;
        self.logics
            .collision
            .clear_and_resize_map(WORLD_SIZE, WORLD_SIZE);
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
        for (idx, (col_event, _)) in self.events.collision.iter_mut().enumerate() {
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
        for i in remove.into_iter().rev() {
            let _ = self.events.collision.remove(i);
        }
        self.state.characters.remove(ent_i);
    }

    pub fn remove_tile_at_pos(&mut self, room: usize, pos: IVec2) {
        self.state.rooms[room].map[pos.y as usize][pos.x as usize] = None;
        if self.state.current_room == room {
            self.logics
                .collision
                .handle_predicate(&CollisionReaction::RemoveTileAtPos(pos));
        }

        let mut remove = Vec::new();
        for (idx, (col_event, _)) in self.events.collision.iter_mut().enumerate() {
            if let ColEvent::Tile(_, ev_pos) = col_event {
                if pos == *ev_pos {
                    remove.push(idx);
                }
            }
        }
        for i in remove.into_iter() {
            let _ = self.events.collision.remove(i);
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
    pub fn consume_player(&mut self, player: Player) {
        self.collision.positions.insert(0, player.pos);
        self.collision.amt_moved.insert(0, IVec2::ZERO);

        self.collision
            .metadata
            .insert(0, CollisionData::new(true, false, CollisionEnt::Player));

        for (act_id, keycode, valid, _) in player.controls {
            self.control.add_key_map(0, keycode, act_id, valid);
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
