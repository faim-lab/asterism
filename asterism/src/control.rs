//! # Control logics
//!
//! Control logics communicate that different entities are controlled by different inputs at different times. They map button inputs, AI intentions, network socket messages, etc onto high-level game actions.
//!
//! We're currently trying to consider analog as well as digital inputs, but we haven't implemented controller support, so some of these fields don't really make sense yet.

use crate::{tables::QueryTable, Event, EventType, Logic, Reaction};

/// Information for a key/button press.
trait Input {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

/// A keyboard control logic.
///
/// A Wrapper is a helper struct that helps keep track of information that libraries may not but we do want, such as [BevyInputWrapper], [MacroquadInputWrapper], or [WinitInputWrapper].
pub struct KeyboardControl<ID, Wrapper>
where
    ID: Copy + Eq + Ord,
    Wrapper: InputWrapper,
{
    /// Input mappings from actions to keypresses. Each outer Vec is a set of inputs, ex. one player gets the first set of mappings, another gets a second set of mappings, an AI player gets the third.
    pub mapping: Vec<Vec<Action<ID, Wrapper::KeyCode>>>,
    /// The values for each keypress in the sets described above.
    pub values: Vec<Vec<Values>>,
    /// An input wrapper
    input_wrapper: Wrapper,
}

impl<ID, Wrapper> Logic for KeyboardControl<ID, Wrapper>
where
    ID: Copy + Eq + Ord,
    Wrapper: InputWrapper,
{
    type Event = ControlEvent<ID>;
    type Reaction = ControlReaction<ID, Wrapper::KeyCode>;

    /// for each control locus
    type Ident = usize;
    /// values are only included for reference; modifying values will not change the data in the logic. Not super sure about this type...
    type IdentData = Vec<Action<ID, Wrapper::KeyCode>>;

    fn handle_predicate(&mut self, reaction: &Self::Reaction) {
        match reaction {
            ControlReaction::AddKeyToSet(set, id, key, valid) => {
                self.add_key_map(*set, *key, *id, *valid)
            }
            ControlReaction::SetKeyValid(set, id) => {
                if let Some(action) = self.mapping[*set].iter_mut().find(|act| act.id == *id) {
                    action.is_valid = true;
                }
            }
            ControlReaction::SetKeyInvalid(set, id) => {
                if let Some(action) = self.mapping[*set].iter_mut().find(|act| act.id == *id) {
                    action.is_valid = false;
                }
            }
        }
    }

    fn get_synthesis(&self, ident: Self::Ident) -> Self::IdentData {
        self.mapping[ident].clone()
    }

    fn update_synthesis(&mut self, ident: Self::Ident, data: Self::IdentData) {
        self.mapping[ident] = data;
    }
}

impl<ID, Wrapper> KeyboardControl<ID, Wrapper>
where
    ID: Copy + Eq + Ord,
    Wrapper: InputWrapper,
{
    pub fn new() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new(),
            input_wrapper: Wrapper::new(),
        }
    }

    /// Checks and updates what inputs are being pressed every frame.
    pub fn update(&mut self, events: &Wrapper::InputHelper) {
        self.input_wrapper.clear();
        for (map, map_values) in self.mapping.iter().zip(self.values.iter_mut()) {
            for (action, mut values) in map.iter().zip(map_values.iter_mut()) {
                let Action {
                    key_input,
                    input_type,
                    is_valid,
                    ..
                } = action;
                let Values { value, changed_by } = &mut values;
                // if not valid, reset and skip check. could cause problems if a key were pressed before it became valid then the key became valid while still being held. this is probably semi-reasonable, actually
                if !*is_valid {
                    *value = 0.0;
                    *changed_by = 0.0;
                    continue;
                }
                match input_type {
                    InputType::Digital => {
                        // NOTE: if update_held isn't called for every key in the mappings, it can completely break some of the input wrappers.
                        //
                        // This feels easily broken... but it feels less weird than filtering out and looping through all inputs beforehand to see if they're held, _then_ calling is_held again---which is just doing the same thing twice?
                        if self.input_wrapper.update_held(&key_input.keycode, events) {
                            if self.input_wrapper.is_pressed(&key_input.keycode, events) {
                                *changed_by = 1.0;
                            } else {
                                *changed_by = 0.0;
                            }
                        } else if self.input_wrapper.is_released(&key_input.keycode, events)
                        // see comment earlier about keypresses that are invalid. logic may not be correct though
                            && *value != 0.0
                        {
                            *changed_by = -1.0;
                        } else {
                            *changed_by = 0.0;
                        }
                    }
                    InputType::Analog => unimplemented!(),
                }
                *value = (*value + *changed_by)
                    .max(key_input.min())
                    .min(key_input.max());
            }
        }
    }

    /// Returns the [Values] for the first action in the mapping with the given ID.
    pub fn get_action(&self, id: ID) -> Option<Values> {
        for (i, ..) in self.mapping.iter().enumerate() {
            if let Some(values) = self.get_action_in_set(i, id) {
                return Some(values);
            }
        }
        None
    }

    /// Returns the [Values] for the action with the given ID in the given set of mappings.
    pub fn get_action_in_set(&self, action_set: <Self as Logic>::Ident, id: ID) -> Option<Values> {
        if let Some(i) = self.mapping[action_set].iter().position(|act| act.id == id) {
            return Some(self.values[action_set][i]);
        }
        None
    }

    /// Adds a single keymap to the logic.
    pub fn add_key_map(
        &mut self,
        locus_idx: <Self as Logic>::Ident,
        keycode: Wrapper::KeyCode,
        id: ID,
        valid: bool,
    ) {
        if locus_idx >= self.mapping.len() {
            self.mapping.resize_with(locus_idx + 1, Default::default);
            self.values.resize_with(locus_idx + 1, Default::default);
        }
        self.mapping[locus_idx].push(Action::new(id, keycode, InputType::Digital, valid));
        self.values[locus_idx].push(Values::new());
    }
}

/// A keyboard input.
#[derive(Clone, Copy)]
pub struct KeyInput<KeyCode: Copy> {
    /// The keycode that the input is tracking.
    keycode: KeyCode,
}

impl<KeyCode: Copy> Input for KeyInput<KeyCode> {
    /// Minimum value for a keypress is 0.0.
    fn min(&self) -> f32 {
        0.0
    }
    /// Maximum value for a keypress is 1.0.
    fn max(&self) -> f32 {
        1.0
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum InputType {
    Analog,
    Digital,
}

/// Information about the player's input related to one action.
#[derive(Copy, Clone, Debug)]
pub struct Values {
    /// How much the value of the input was changed last frame.
    pub changed_by: f32,
    /// What the value of the input is now.
    pub value: f32,
}

impl Values {
    pub fn new() -> Self {
        Self {
            changed_by: 0.0,
            value: 0.0,
        }
    }
}

/// Information for an action and the input it's attached to.
#[derive(Clone, Copy)]
pub struct Action<ID, KeyCode: Copy> {
    pub id: ID,
    /// The input's keycode and min/max.
    pub key_input: KeyInput<KeyCode>,
    /// If the input is valid that frame, i.e. should be able to be pressed.
    pub is_valid: bool,
    /// If the input is digital or analog.
    pub input_type: InputType,
}

impl<ID, KeyCode: Copy> Action<ID, KeyCode> {
    pub fn new(id: ID, keycode: KeyCode, input_type: InputType, is_valid: bool) -> Self {
        Self {
            id,
            key_input: KeyInput { keycode },
            is_valid,
            input_type,
        }
    }

    pub fn get_keycode(&self) -> &KeyCode {
        &self.key_input.keycode
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ControlReaction<ID: Copy + Eq, KeyCode: Copy + Eq> {
    AddKeyToSet(usize, ID, KeyCode, bool),
    SetKeyValid(usize, ID),
    SetKeyInvalid(usize, ID),
}

impl<ID: Copy + Eq, KeyCode: Copy + Eq> Reaction for ControlReaction<ID, KeyCode> {}

#[derive(PartialEq, Eq, Ord, PartialOrd, Debug)]
pub struct ControlEvent<ID> {
    pub event_type: ControlEventType,
    pub set: usize,
    pub action_id: ID,
}

impl<ID> Event for ControlEvent<ID> {
    type EventType = ControlEventType;
    fn get_type(&self) -> &Self::EventType {
        &self.event_type
    }
}

#[derive(PartialEq, Eq, Ord, PartialOrd, Debug)]
pub enum ControlEventType {
    KeyPressed,
    KeyReleased,
    KeyHeld,
    KeyUnheld,
}

impl EventType for ControlEventType {}

type QueryOver<ID, Wrapper> = (
    <KeyboardControl<ID, Wrapper> as Logic>::Ident,
    <KeyboardControl<ID, Wrapper> as Logic>::IdentData,
);
impl<ID: Copy + Eq + Ord, Wrapper: InputWrapper> QueryTable<QueryOver<ID, Wrapper>>
    for KeyboardControl<ID, Wrapper>
{
    fn check_predicate(&self, predicate: impl Fn(&QueryOver<ID, Wrapper>) -> bool) -> Vec<bool> {
        (0..self.mapping.len())
            .map(|i| {
                let query_over = (i, self.get_synthesis(i));
                predicate(&query_over)
            })
            .collect()
    }
}

type QueryEvent<ID, Wrapper> = <KeyboardControl<ID, Wrapper> as Logic>::Event;
impl<ID: Copy + Eq + Ord, Wrapper: InputWrapper> QueryTable<QueryEvent<ID, Wrapper>>
    for KeyboardControl<ID, Wrapper>
{
    // I feel like this should get rid of check_predicate
    fn check_predicate(&self, predicate: impl Fn(&QueryEvent<ID, Wrapper>) -> bool) -> Vec<bool> {
        self.mapping
            .iter()
            .flatten()
            .zip(self.values.iter().flatten())
            .enumerate()
            .map(|(i, (action, values))| {
                let event_type = if values.changed_by > 0.0 {
                    ControlEventType::KeyPressed
                } else if values.changed_by < 0.0 {
                    ControlEventType::KeyReleased
                } else if values.value != 0.0 {
                    ControlEventType::KeyHeld
                } else {
                    ControlEventType::KeyUnheld
                };
                let event = ControlEvent {
                    set: i,
                    action_id: action.id,
                    event_type,
                };
                predicate(&event)
            })
            .collect()
    }
}

/// A wrapper to help keep track of input information that preexisting input handlers may not offer, but that we need.
pub trait InputWrapper {
    type KeyCode: Copy + Eq;
    type InputHelper;
    fn new() -> Self;

    /// clears input information for this frame
    fn clear(&mut self);

    /// if the key is held or not. If keeping track of current inputs, also logs what keys are being pressed this frame.
    fn update_held(&mut self, key: &Self::KeyCode, events: &Self::InputHelper) -> bool;

    /// if the key has just been pressed or not
    fn is_pressed(&self, key: &Self::KeyCode, events: &Self::InputHelper) -> bool;

    /// if the key has just been released or not
    fn is_released(&self, key: &Self::KeyCode, events: &Self::InputHelper) -> bool;
}

use macroquad::prelude::{is_key_down, is_key_pressed, is_key_released, KeyCode as MqKeyCode};
/// Macroquad's input handler already correctly handles the information we need, so this is just a wrapper for their functions
pub struct MacroquadInputWrapper {}

impl InputWrapper for MacroquadInputWrapper {
    type KeyCode = MqKeyCode;
    type InputHelper = ();
    fn new() -> Self {
        Self {}
    }

    fn clear(&mut self) {}

    fn update_held(&mut self, key: &MqKeyCode, _events: &()) -> bool {
        is_key_down(*key)
    }

    fn is_pressed(&self, key: &MqKeyCode, _events: &()) -> bool {
        is_key_pressed(*key)
    }

    fn is_released(&self, key: &MqKeyCode, _events: &()) -> bool {
        is_key_released(*key)
    }
}

#[cfg(feature = "winit-render")]
use std::collections::BTreeSet;
#[cfg(feature = "winit-render")]
use winit::event::VirtualKeyCode;
#[cfg(feature = "winit-render")]
use winit_input_helper::WinitInputHelper;

/// WinitInputHelper doesn't handle key repeat properly because of key repeat, so track the keys pressed last and this frame.
#[cfg(feature = "winit-render")]
pub struct WinitInputWrapper {
    this_frame_keys: BTreeSet<VirtualKeyCode>,
    last_frame_keys: BTreeSet<VirtualKeyCode>,
}

#[cfg(feature = "winit-render")]
impl InputWrapper for WinitInputWrapper {
    type KeyCode = VirtualKeyCode;
    type InputHelper = WinitInputHelper;

    fn new() -> Self {
        Self {
            this_frame_keys: BTreeSet::new(),
            last_frame_keys: BTreeSet::new(),
        }
    }

    fn clear(&mut self) {
        self.last_frame_keys = std::mem::take(&mut self.this_frame_keys);
    }

    fn update_held(&mut self, key: &VirtualKeyCode, events: &WinitInputHelper) -> bool {
        if events.key_held(*key) {
            self.this_frame_keys.insert(*key);
            return true;
        }
        false
    }

    fn is_pressed(&self, key: &VirtualKeyCode, _events: &WinitInputHelper) -> bool {
        self.this_frame_keys.contains(key) && !self.last_frame_keys.contains(key)
    }

    fn is_released(&self, key: &VirtualKeyCode, events: &WinitInputHelper) -> bool {
        events.key_released(*key)
    }
}

#[cfg(feature = "bevy-engine")]
use bevy_input::{keyboard::KeyCode as BevyKeyCode, Input as BevyInput};

#[cfg(feature = "bevy-engine")]
/// Bevy's input handler already correctly handles the information we need, so this is just a wrapper for their functions
pub struct BevyInputWrapper;

#[cfg(feature = "bevy-engine")]
impl InputWrapper for BevyInputWrapper {
    type KeyCode = BevyKeyCode;
    type InputHelper = BevyInput<BevyKeyCode>;

    fn new() -> Self {
        Self
    }

    fn clear(&mut self) {}

    fn update_held(&mut self, key: &BevyKeyCode, events: &BevyInput<BevyKeyCode>) -> bool {
        events.pressed(*key)
    }

    fn is_pressed(&self, key: &BevyKeyCode, events: &BevyInput<BevyKeyCode>) -> bool {
        events.just_pressed(*key)
    }

    fn is_released(&self, key: &BevyKeyCode, events: &BevyInput<BevyKeyCode>) -> bool {
        events.just_released(*key)
    }
}
