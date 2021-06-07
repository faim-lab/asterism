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
//! - Write tilemap collision
//! - Write syntheses functions
//! - See what errors still persist from there
//! - Add linking logics?
//! - Game mode logics?

#![allow(unused)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::new_without_default)]

use std::collections::BTreeMap;

use asterism::{
    collision::AabbCollision,
    control::{KeyboardControl, MacroquadInputWrapper},
    resources::QueuedResources,
};
use macroquad::prelude::*;

// reexports
pub use asterism::control::{Action, ControlEventType, ControlReaction, Values};
pub use asterism::resources::{ResourceEventType, ResourceReaction, Transaction};
pub use asterism::Logic;
pub use types::{ColEvent, CtrlEvent, RsrcEvent};

const TILE_SIZE: usize = 32;
const WORLD_SIZE: usize = 8;
pub const GAME_SIZE: usize = TILE_SIZE * WORLD_SIZE;

mod collision;
use collision::*;
mod types;
use types::*;
mod syntheses;
use syntheses::*;

pub struct Game {
    pub state: State,
    pub logics: Logics,
    events: Events,
    colors: Colors,
}

struct Colors {
    background_color: Color,
    colors: BTreeMap<EntID, Color>,
}

pub(crate) struct Room {
    pub map: [[Option<TileID>; WORLD_SIZE]; WORLD_SIZE],
}

pub struct State {
    current_room: usize,
    rooms: Vec<Room>,
    player: PlayerID,
    characters: Vec<CharacterID>,
    add_queue: Vec<Ent>,
    remove_queue: Vec<EntID>,
}

impl State {
    fn new(player: PlayerID) -> Self {
        Self {
            current_room: 0,
            rooms: Vec::new(),
            player,
            characters: Vec::new(),
            add_queue: Vec::new(),
            remove_queue: Vec::new(),
        }
    }

    fn get_col_idx(&mut self, i: usize, ent: CollisionEnt) -> usize {
        match ent {
            CollisionEnt::Player => 0,
            CollisionEnt::Character => i + 1,
        }
    }
}

pub struct Logics {
    control: KeyboardControl<ActionID, MacroquadInputWrapper>,
    collision: TileMapCollision<TileID, CollisionEnt>,
    resources: QueuedResources<RsrcID, u16>,
}

type PredicateFn<Event> = Vec<(Event, Box<dyn Fn(&mut State, &mut Logics, &Event)>)>;

pub struct Events {
    collision: PredicateFn<ColEvent>,
    control: PredicateFn<CtrlEvent>,
    resources: PredicateFn<RsrcEvent>,

    player_synth: PlayerSynth,
    tile_synth: TileSynth,
    character_synth: CharacterSynth,
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

pub async fn run(mut game: Game) {
    use std::time::*;
    let mut available_time = 0.0;
    let mut since = Instant::now();
    const DT: f32 = 1.0 / 60.0;

    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        draw(&game);
        available_time += since.elapsed().as_secs_f32();
        since = Instant::now();

        // framerate
        while available_time >= DT {
            available_time -= DT;

            let add_queue = std::mem::take(&mut game.state.add_queue);
            for _ent in add_queue {}

            control(&mut game);
            collision(&mut game);
            resources(&mut game);

            let remove_queue = std::mem::take(&mut game.state.remove_queue);
            for _ent in remove_queue {}
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

fn collision(game: &mut Game) {
    game.player_col_synthesis();
    game.tile_synthesis();
    game.character_synthesis();

    game.logics.collision.update();

    for (predicate, reaction) in game.events.collision.iter() {
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

fn draw(game: &Game) {
    clear_background(game.colors.background_color);
    let map = game.state.rooms[game.state.current_room].map;
    for (y, row) in map.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            if let Some(tile) = tile {
                let color = game
                    .colors
                    .colors
                    .get(&EntID::Tile(*tile))
                    .expect("tile color undefined");
            }
        }
    }

    let color = game
        .colors
        .colors
        .get(&EntID::Player(game.state.player))
        .expect("no player color defined");
}
