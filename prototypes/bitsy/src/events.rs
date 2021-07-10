use crate::syntheses::*;
use crate::types::*;
use crate::{Logics, State};
use asterism::linking::LinkingEvent;

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

            max_query_count: 0,
        }
    }

    pub fn add_query(&mut self) -> UserQueryID {
        let id = UserQueryID::new(self.max_query_count);
        self.max_query_count += 1;
        id
    }
}
