use asterism::control::{Action, Values};
use asterism::Logic;
use macroquad::math::IVec2;

use crate::collision::*;
use crate::types::*;
use crate::Game;

pub type Synthesis<Ident> = Box<dyn Fn(Ident) -> Ident>;

impl Game {
    pub fn set_player_col_synthesis(&mut self, synthesis: Synthesis<Player>) {
        self.events.player_synth.col = Some(synthesis);
    }
    pub fn set_player_ctrl_synthesis(&mut self, synthesis: Synthesis<Player>) {
        self.events.player_synth.ctrl = Some(synthesis);
    }
    pub fn set_player_rsrc_synthesis(&mut self, synthesis: Synthesis<Player>) {
        self.events.player_synth.rsrc = Some(synthesis);
    }

    pub fn set_character_col_synthesis(&mut self, synthesis: Synthesis<Character>) {
        self.events.character_synth.col = Some(synthesis);
    }

    pub fn set_tile_synthesis(&mut self, synthesis: Synthesis<Tile>) {
        self.events.tile_synth.col = Some(synthesis);
    }

    pub(crate) fn player_col_synthesis(&mut self) {
        if let Some(player_id) = self.state.player {
            if let Some(synthesis) = self.events.player_synth.col.as_ref() {
                let i = player_id.idx();
                let mut col = self.logics.collision.get_synthesis(ColIdent::EntIdx(i));

                let rsrc = self
                    .state
                    .resources
                    .iter()
                    .map(|rsrc_id| self.logics.resources.get_synthesis(*rsrc_id));

                let player = self.build_player(&col, &self.logics.control.get_synthesis(i), rsrc);

                let player = synthesis(player);

                if let TileMapColData::Ent { pos, amt_moved, .. } = &mut col {
                    *pos = player.pos;
                    *amt_moved = player.amt_moved;
                }

                self.logics
                    .collision
                    .update_synthesis(ColIdent::EntIdx(i), col);
                self.colors
                    .colors
                    .insert(EntID::Player(player_id), player.color);
            }
        }
    }

    pub(crate) fn player_ctrl_synthesis(&mut self) {
        if let Some(player_id) = self.state.player {
            if let Some(synthesis) = self.events.player_synth.ctrl.as_ref() {
                let i = player_id.idx();
                let col_idx = self.state.get_col_idx(i, CollisionEnt::Player);
                let mut ctrl = self.logics.control.get_synthesis(i);

                let rsrc = self
                    .state
                    .resources
                    .iter()
                    .map(|rsrc_id| self.logics.resources.get_synthesis(*rsrc_id));

                let player = self.build_player(
                    &self
                        .logics
                        .collision
                        .get_synthesis(ColIdent::EntIdx(col_idx)),
                    &ctrl,
                    rsrc,
                );

                let player = synthesis(player);

                for (((_, _, valid, vals), actions), values) in player
                    .controls
                    .iter()
                    .zip(ctrl.0.iter_mut())
                    .zip(ctrl.1.iter_mut())
                {
                    actions.is_valid = *valid;
                    *values = *vals;
                }
                self.logics.control.update_synthesis(i, ctrl);
            }
        }
    }

    pub(crate) fn player_rsrc_synthesis(&mut self) {
        if let Some(player_id) = self.state.player {
            if let Some(synthesis) = self.events.player_synth.ctrl.as_ref() {
                let i = player_id.idx();
                let col_idx = self.state.get_col_idx(i, CollisionEnt::Player);
                let rsrc = self
                    .state
                    .resources
                    .iter()
                    .map(|rsrc_id| self.logics.resources.get_synthesis(*rsrc_id));

                let player = self.build_player(
                    &self
                        .logics
                        .collision
                        .get_synthesis(ColIdent::EntIdx(col_idx)),
                    &self.logics.control.get_synthesis(i),
                    rsrc,
                );
                let player = synthesis(player);

                // not sure about this logic, the RsrcID::new() part feels wrong...
                for (i, rsrc) in player.inventory.into_iter().enumerate() {
                    let id = RsrcID::new(i);
                    if self.logics.resources.items.contains_key(&id) {
                        self.logics
                            .resources
                            .update_synthesis(id, (rsrc.val, rsrc.min, rsrc.max));
                    } else {
                        let id = self.log_rsrc();
                        self.logics
                            .resources
                            .items
                            .insert(id, (rsrc.val, rsrc.min, rsrc.max));
                    }
                }
            }
        }
    }

    pub(crate) fn tile_synthesis(&mut self) {
        if let Some(synthesis) = self.events.tile_synth.col.as_ref() {
            for (y, row) in self.state.get_current_room().map.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    if let Some(tile_id) = tile {
                        let pos = IVec2::new(x as i32, y as i32);
                        let mut col = self.logics.collision.get_synthesis(ColIdent::Position(pos));

                        let mut tile = Tile::new();
                        if let TileMapColData::Position { solid, .. } = col {
                            tile.solid = solid;
                        }
                        tile.color = *self
                            .colors
                            .colors
                            .get(&EntID::Tile(*tile_id))
                            .unwrap_or_else(|| panic!("tile {} color not set", tile_id.idx()));

                        let tile = synthesis(tile);
                        if let TileMapColData::Position { solid, .. } = &mut col {
                            *solid = tile.solid;
                        }
                        self.logics
                            .collision
                            .update_synthesis(ColIdent::Position(pos), col);
                    }
                }
            }
        }
    }

    pub(crate) fn character_synthesis(&mut self) {
        if let Some(synthesis) = self.events.character_synth.col.as_ref() {
            for (i, char_id) in self.state.characters.iter().enumerate() {
                let col_idx = self.state.get_col_idx(i, CollisionEnt::Character);
                let mut col = self
                    .logics
                    .collision
                    .get_synthesis(ColIdent::EntIdx(col_idx));

                let mut character = Character::new();
                if let TileMapColData::Ent { pos, .. } = col {
                    character.pos = pos;
                }
                character.color = *self
                    .colors
                    .colors
                    .get(&EntID::Character(*char_id))
                    .unwrap_or_else(|| panic!("character {} color not set", char_id.idx()));

                let character = synthesis(character);

                if let TileMapColData::Ent { pos, .. } = &mut col {
                    *pos = character.pos;
                }

                self.logics
                    .collision
                    .update_synthesis(ColIdent::EntIdx(col_idx), col);
            }
        }
    }

    fn build_player(
        &self,
        col: &TileMapColData<TileID, CollisionEnt>,
        ctrl: &(
            Vec<Action<ActionID, macroquad::input::KeyCode>>,
            Vec<Values>,
        ),
        rsrc: impl Iterator<Item = (u16, u16, u16)>,
    ) -> Player {
        let mut player = Player::new();
        if let TileMapColData::Ent { pos, amt_moved, .. } = col {
            player.pos = *pos;
            player.amt_moved = *amt_moved;
        }

        for (player, (actions, values)) in player
            .controls
            .iter_mut()
            .zip(ctrl.0.iter().zip(ctrl.1.iter()))
        {
            *player = (
                actions.id,
                *actions.get_keycode(),
                actions.is_valid,
                *values,
            );
        }

        // every resource in this logic belongs to the player
        //
        // in a more complex inventory system, keeping track of what RsrcIDs belong to who in the game state? here it's kind of pointless
        for vals in rsrc {
            let mut rsrc = Resource::new();
            rsrc.val = vals.0;
            rsrc.min = vals.1;
            rsrc.max = vals.2;
            player.inventory.push(rsrc);
        }

        player
    }
}
