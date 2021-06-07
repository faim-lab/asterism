use asterism::Logic;

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
        if let Some(synthesis) = self.events.player_synth.col.as_ref() {
            let i = self.state.player.idx();
            let col_idx = self.state.get_col_idx(i, CollisionEnt::Player);
            let mut col = self
                .logics
                .collision
                .get_synthesis(ColIdent::EntIdx(col_idx));
            let ctrl = self.logics.control.get_synthesis(i);

            if let TileMapColData::Ent { pos, .. } = col {
                let mut player = Player::new();
                player.pos = pos;

                for (actions, values) in ctrl.0.iter().zip(ctrl.1.iter()) {
                    let ctrl = (
                        actions.id,
                        *actions.get_keycode(),
                        actions.is_valid,
                        *values,
                    );
                    player.controls.push(ctrl);
                }

                for (item, vals) in self.logics.resources.items.iter() {
                    player.inventory.insert(*item, *vals);
                }

                player.color = *self
                    .colors
                    .colors
                    .get(&EntID::Player(self.state.player))
                    .expect("player color not set");
                let player = synthesis(player);

                pos = player.pos;
                self.logics
                    .collision
                    .update_synthesis(ColIdent::EntIdx(col_idx), col);
                self.colors
                    .colors
                    .insert(EntID::Player(self.state.player), player.color);
            }
        }
    }

    pub(crate) fn player_ctrl_synthesis(&mut self) {
        if let Some(synthesis) = self.events.player_synth.ctrl.as_ref() {
            let i = self.state.player.idx();
            let col_idx = self.state.get_col_idx(i, CollisionEnt::Player);
            let col = self.logics.collision.get_synthesis(col_idx);
            let mut ctrl = self.logics.control.get_synthesis(i);

            let mut player = Player::new();
            player.pos = col.center - col.half_size;
            for (actions, values) in ctrl.0.iter().zip(ctrl.1.iter()) {
                let ctrl = (
                    actions.id,
                    *actions.get_keycode(),
                    actions.is_valid,
                    *values,
                );
                player.controls.push(ctrl);
            }
            player.color = *self
                .colors
                .colors
                .get(&EntID::Player(self.state.player))
                .expect("player color not set");
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

    pub(crate) fn player_rsrc_synthesis(&mut self) {
        if let Some(synthesis) = self.events.player_synth.ctrl.as_ref() {
            let i = self.state.player.idx();
            let col_idx = self.state.get_col_idx(i, CollisionEnt::Player);
            let col = self.logics.collision.get_synthesis(col_idx);
            let mut ctrl = self.logics.control.get_synthesis(i);

            let mut player = Player::new();
            player.pos = col.center - col.half_size;
            for (actions, values) in ctrl.0.iter().zip(ctrl.1.iter()) {
                let ctrl = (
                    actions.id,
                    *actions.get_keycode(),
                    actions.is_valid,
                    *values,
                );
                player.controls.push(ctrl);
            }

            player.color = *self
                .colors
                .colors
                .get(&EntID::Player(self.state.player))
                .expect("player color not set");
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

    pub(crate) fn tile_synthesis(&mut self) {
        if let Some(synthesis) = self.events.tile_synth.col.as_ref() {
            let map = self.state.rooms[self.state.current_room].map;
            for (i, _) in map.iter().enumerate() {
                let col_idx = self.state.get_col_idx(i, CollisionEnt::Tile);
                let mut col = self.logics.collision.get_synthesis(col_idx);

                let mut tile = Tile::new();
                tile.pos = col.center - col.half_size;
                tile.solid = col.solid;
                // tile.color = *self
                //     .colors
                //     .colors
                //     .get(&EntID::Tile(tile_id))
                //     .expect("player color not set");

                let tile = synthesis(tile);
                col.center = tile.pos + col.half_size;
                self.logics.collision.update_synthesis(col_idx, col);
            }
        }
    }

    pub(crate) fn character_synthesis(&mut self) {
        if let Some(synthesis) = self.events.character_synth.col.as_ref() {
            for (i, char_id) in self.state.characters.iter().enumerate() {
                let col_idx = self.state.get_col_idx(i, CollisionEnt::Character);
                let mut col = self.logics.collision.get_synthesis(col_idx);

                let mut character = Character::new();
                // get position from physics
                character.pos = col.center;
                character.color = *self
                    .colors
                    .colors
                    .get(&EntID::Character(*char_id))
                    .expect("player color not set");

                let character = synthesis(character);

                col.center = character.pos + col.half_size;

                self.logics.collision.update_synthesis(col_idx, col);
            }
        }
    }
}
