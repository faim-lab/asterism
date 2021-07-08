use crate::syntheses::*;
use crate::types::*;
use crate::{Logics, State};
use asterism::linking::LinkingEvent;

type PredicateFn<Event> = Vec<(Event, Box<dyn Fn(&mut State, &mut Logics, &Event)>)>;

pub(crate) struct Events {
    pub control: PredicateFn<CtrlEvent>,
    pub collision: PredicateFn<(ColEvent, usize)>, // usize is the current room number
    pub linking: PredicateFn<LinkingEvent>,
    pub resource_event: PredicateFn<RsrcEvent>,
    pub resource_ident: PredicateFn<(RsrcID, (u16, u16, u16))>,

    pub player_synth: PlayerSynth,
    pub tile_synth: TileSynth,
    pub character_synth: CharacterSynth,
}

pub struct PlayerSynth {
    pub ctrl: Option<Synthesis<Player>>,
    pub col: Option<Synthesis<Player>>,
    pub rsrc: Option<Synthesis<Player>>,
}

pub struct TileSynth {
    pub col: Option<Synthesis<Tile>>,
}

pub struct CharacterSynth {
    pub col: Option<Synthesis<Character>>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            control: Vec::new(),
            collision: Vec::new(),
            linking: Vec::new(),
            resource_event: Vec::new(),
            resource_ident: Vec::new(),

            player_synth: PlayerSynth {
                ctrl: None,
                col: None,
                rsrc: None,
            },
            tile_synth: TileSynth { col: None },
            character_synth: CharacterSynth { col: None },
        }
    }
}
