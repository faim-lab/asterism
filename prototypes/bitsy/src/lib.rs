//! # What a Bitsy is, to me
//!
//! - characters moving around a tilemap
//! - tiles
//! - sprites that you interact with (dialogue?)
//! - items/inventory??
//! - multiple rooms, moving from one to another
//!
//! drawing: I'm not doing the pixel art thing. The player, interactable characters, and tiles all have different colors
//!
//! TODO:
//! - [x] Write tilemap collision
//! - [x] Write syntheses functions
//! - [x] Write bitsy test game
//! - [x] See what errors still persist from there
//! - [x] Add resource logics/inventory (I think????)
//! - [x] Adding/removing entities
//! - [ ] Add linking logics
//!     - [x] graph/state machine struct
//!     - [ ] composing multiple queries
//!         - [ ] query tables???
//! - [ ] Game mode logics/dialogue?

#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::new_without_default)]

use std::collections::BTreeMap;

use asterism::{
    control::{KeyboardControl, MacroquadInputWrapper},
    linking::GraphedLinking,
    resources::QueuedResources,
};
use macroquad::prelude::*;

// reexports
pub use asterism::control::{Action, ControlEventType, ControlReaction, Values};
pub use asterism::linking::{LinkingEvent, LinkingEventType, LinkingReaction};
pub use asterism::resources::{ResourceEventType, ResourceReaction, Transaction};
pub use asterism::Logic;
pub use collision::*;
pub use entities::set_current_room;
pub use types::*;

const TILE_SIZE: usize = 32;
pub const WORLD_SIZE: usize = 8;
pub const GAME_SIZE: usize = TILE_SIZE * WORLD_SIZE;

mod collision;
mod syntheses;
mod types;
use syntheses::*;
mod entities;
mod tables;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "extreme dungeon crawler".to_owned(),
        window_width: GAME_SIZE as i32,
        window_height: GAME_SIZE as i32,
        fullscreen: false,
        ..Default::default()
    }
}

pub struct Game {
    pub state: State,
    pub logics: Logics,
    pub events: Events,
    pub colors: Colors,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: State::new(),
            logics: Logics::new(),
            events: Events::new(),
            colors: Colors {
                background_color: DARKBLUE,
                colors: BTreeMap::new(),
            },
        }
    }
}

pub struct Colors {
    pub background_color: Color,
    pub colors: BTreeMap<EntID, Color>,
}

#[derive(Default)]
pub struct Room {
    pub map: [[Option<TileID>; WORLD_SIZE]; WORLD_SIZE],
}

pub struct State {
    pub rooms: Vec<Room>,
    pub player: bool,
    pub resources: Vec<RsrcID>,
    rsrc_id_max: usize,
    pub characters: Vec<CharacterID>,
    char_id_max: usize,
    pub links: BTreeMap<LinkID, (usize, IVec2)>,
    link_id_max: usize,
    tile_type_count: usize,
    add_queue: Vec<Ent>,
    remove_queue: Vec<EntID>,
}

impl State {
    fn new() -> Self {
        Self {
            rooms: Vec::new(),
            player: false,
            characters: Vec::new(),
            char_id_max: 0,
            resources: Vec::new(),
            rsrc_id_max: 0,
            links: BTreeMap::new(),
            link_id_max: 0,
            tile_type_count: 0,
            add_queue: Vec::new(),
            remove_queue: Vec::new(),
        }
    }

    pub fn get_col_idx(&self, i: usize, ent: CollisionEnt) -> usize {
        match ent {
            CollisionEnt::Player => 0,
            CollisionEnt::Character => i + 1,
        }
    }

    pub fn queue_remove(&mut self, ent: EntID) {
        self.remove_queue.push(ent);
    }
    pub fn queue_add(&mut self, ent: Ent) {
        self.add_queue.push(ent);
    }
}

pub struct Logics {
    pub control: KeyboardControl<ActionID, MacroquadInputWrapper>,
    pub collision: TileMapCollision<TileID, CollisionEnt>,
    pub resources: QueuedResources<RsrcID, u16>,
    pub linking: GraphedLinking<LinkID>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: KeyboardControl::new(),
            collision: TileMapCollision::new(WORLD_SIZE, WORLD_SIZE),
            resources: QueuedResources::new(),
            linking: {
                let mut linking = GraphedLinking::new();
                linking.add_graph(0, []);
                linking
            },
        }
    }
}

type PredicateFn<Event> = Vec<(Event, Box<dyn Fn(&mut State, &mut Logics, &Event)>)>;

pub struct Events {
    // honestly this is more like a game mode logic than a linking logic, but
    pub predicates: Vec<Predicates>,
    pub resources: PredicateFn<RsrcEvent>,
    pub control: PredicateFn<CtrlEvent>,

    player_synth: PlayerSynth,
    tile_synth: TileSynth,
    character_synth: CharacterSynth,
}

pub struct Predicates {
    pub collision: PredicateFn<ColEvent>,
    pub linking: PredicateFn<LinkingEvent>,
}

impl Predicates {
    fn new() -> Self {
        Self {
            collision: Vec::new(),
            linking: Vec::new(),
        }
    }
}

struct PlayerSynth {
    ctrl: Option<Synthesis<Player>>,
    col: Option<Synthesis<Player>>,
    rsrc: Option<Synthesis<Player>>,
}

struct TileSynth {
    col: Option<Synthesis<Tile>>,
}

struct CharacterSynth {
    col: Option<Synthesis<Character>>,
}

impl Events {
    fn new() -> Self {
        Self {
            predicates: Vec::new(),
            control: Vec::new(),
            resources: Vec::new(),
            player_synth: PlayerSynth {
                ctrl: None,
                col: Some(Box::new(|mut player: Player| {
                    let mut vert_moved = false;
                    if player.controls[0].3.changed_by > 0.0 {
                        player.pos.y = (player.pos.y - 1).max(0);
                        player.amt_moved.y = -1;
                        vert_moved = true;
                    }
                    if player.controls[1].3.changed_by > 0.0 {
                        player.pos.y = (player.pos.y + 1).min(WORLD_SIZE as i32 - 1);
                        player.amt_moved.y = 1;
                        vert_moved = true;
                    }
                    if !vert_moved {
                        player.amt_moved.y = 0;
                    }
                    let mut horiz_moved = false;
                    if player.controls[2].3.changed_by > 0.0 {
                        player.pos.x = (player.pos.x - 1).max(0);
                        player.amt_moved.x = -1;
                        horiz_moved = true;
                    }
                    if player.controls[3].3.changed_by > 0.0 {
                        player.pos.x = (player.pos.x + 1).min(WORLD_SIZE as i32 - 1);
                        player.amt_moved.x = 1;
                        horiz_moved = true;
                    }
                    if !horiz_moved {
                        player.amt_moved.x = 0;
                    }
                    player
                })),
                rsrc: None,
            },
            tile_synth: TileSynth { col: None },
            character_synth: CharacterSynth { col: None },
        }
    }
}

pub async fn run(mut game: Game) {
    game.logics
        .collision
        .clear_and_resize_map(WORLD_SIZE, WORLD_SIZE);

    let current_node = game.logics.linking.graphs[0].current_node;
    let node = game.logics.linking.graphs[0].nodes[current_node];
    let current_room = game.state.links.get(&node).unwrap().0;

    for (row, col_row) in game.state.rooms[current_room]
        .map
        .iter()
        .zip(game.logics.collision.map.iter_mut())
    {
        for (tile, col_tile) in row.iter().zip(col_row.iter_mut()) {
            *col_tile = *tile;
        }
    }

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        draw(&game);

        let add_queue = std::mem::take(&mut game.state.add_queue);
        for _ent in add_queue {}

        control(&mut game);
        collision(&mut game, current_room);
        tables::test(&mut game.logics);
        resources(&mut game);
        linking(&mut game, current_room);

        let remove_queue = std::mem::take(&mut game.state.remove_queue);
        for ent in remove_queue {
            match ent {
                EntID::Player => {
                    game.remove_player();
                }
                EntID::Tile(id) => {
                    let mut remove = Vec::new();
                    for (room_idx, room) in game.state.rooms.iter().enumerate() {
                        for (y, row) in room.map.iter().enumerate() {
                            for (x, tile) in row.iter().enumerate() {
                                if let Some(tile) = tile {
                                    if *tile == id {
                                        remove.push((room_idx, IVec2::new(x as i32, y as i32)));
                                    }
                                }
                            }
                        }
                    }
                    for (i, pos) in remove {
                        game.remove_tile_at_pos(i, pos);
                    }
                }
                EntID::Character(id) => {
                    game.remove_character(id);
                }
            }
        }

        next_frame().await;
    }
}

fn control(game: &mut Game) {
    game.player_ctrl_synthesis();

    game.logics.control.update(&());
    for (predicate, reaction) in game.events.control.iter() {
        if game.logics.control.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }
}

fn collision(game: &mut Game, current_room: usize) {
    game.player_col_synthesis();
    game.tile_synthesis();
    game.character_synthesis();

    game.logics.collision.update();
    let predicates = &game.events.predicates[current_room];

    for (predicate, reaction) in predicates.collision.iter() {
        if game.logics.collision.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }
}

fn resources(game: &mut Game) {
    game.player_rsrc_synthesis();

    game.logics.resources.update();

    for (predicate, reaction) in game.events.resources.iter() {
        if game.logics.resources.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }
}

fn linking(game: &mut Game, current_room: usize) {
    game.logics.linking.update();

    let predicates = &game.events.predicates[current_room];

    for (predicate, reaction) in predicates.linking.iter() {
        if game.logics.linking.check_predicate(predicate) {
            reaction(&mut game.state, &mut game.logics, predicate);
        }
    }
}

fn draw(game: &Game) {
    clear_background(game.colors.background_color);
    for (y, row) in game.logics.collision.map.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            if let Some(tile) = tile {
                let color = game
                    .colors
                    .colors
                    .get(&EntID::Tile(*tile))
                    .unwrap_or_else(|| panic!("tile {} color undefined", tile.idx()));
                draw_rectangle(
                    x as f32 * TILE_SIZE as f32,
                    y as f32 * TILE_SIZE as f32,
                    TILE_SIZE as f32,
                    TILE_SIZE as f32,
                    *color,
                );
            }
        }
    }

    for (i, pos) in game.logics.collision.positions.iter().skip(1).enumerate() {
        let character = game.state.characters[i];
        let color = game
            .colors
            .colors
            .get(&EntID::Character(character))
            .unwrap_or_else(|| panic!("character {} color defined", character.idx()));
        draw_rectangle(
            pos.x as f32 * TILE_SIZE as f32,
            pos.y as f32 * TILE_SIZE as f32,
            TILE_SIZE as f32,
            TILE_SIZE as f32,
            *color,
        );
    }

    if game.state.player {
        let color = game
            .colors
            .colors
            .get(&EntID::Player)
            .expect("player color not set");
        let pos = game.logics.collision.get_synthesis(ColIdent::EntIdx(
            game.state.get_col_idx(0, CollisionEnt::Player),
        ));
        if let TileMapColData::Ent { pos, .. } = pos {
            draw_rectangle(
                pos.x as f32 * TILE_SIZE as f32,
                pos.y as f32 * TILE_SIZE as f32,
                TILE_SIZE as f32,
                TILE_SIZE as f32,
                *color,
            );
        }
    }
}
