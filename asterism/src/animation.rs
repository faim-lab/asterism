//! #Animation
//!
//! Animation handles the sprites and images to be rendered as well as handles
//! the process of rendering them

use futures::executor::block_on;
use macroquad::prelude::*;
use serde;
use serde::Deserialize;
use serde_json;
use std::fs;

/// simple animations across a spritesheet. Stationary background that is represented with simple rectangles---not sprites
pub struct SimpleAnim {
    /// spritesheet being used to draw from, holds all sprites
    pub sheet: SpriteSheet,
    /// number of times drawn is called
    pub frames_drawn: u64,
    /// animation objects being drawn
    pub objects: Vec<AnimObject>,
    /// stationary background elements being drawn
    pub b_elements: Vec<BackElement>,
    /// the color of the background
    pub background_color: Color,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct Rectangle {
    x: u64,
    y: u64,
    w: u64,
    h: u64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct Size {
    w: u64,
    h: u64,
}

/// represents the information needed to access a specifc sprite (image) from a larger sprite sheet
#[derive(Clone, Copy, Debug, Deserialize)]
struct Sprite {
    /// order in sequence
    index: usize,
    /// the specific location  it takes in the spritesheet
    /// is_active: has this sequence been activated i.e. needs to be played, by events in the game
    frame: Rectangle,
    trimmed: bool,
    /// the size of the image
    size: Size,
}

/// a series of sprites that can be played together in order to create an animation
#[derive(Clone, Debug, Deserialize)]
struct Sequence {
    /// the name of the sequence
    seq_name: String,
    /// determines when the sequence switches to a new sprite
    frame_rate: u64,
    /// does these sequence cycle (start over) or does it play once and end
    is_cycle: bool,
    /// has this sequence been activated i.e. needs to be played, by events in the game
    is_active: bool,
    /// if there are multiple active sequences, what priority does this sequence have over the others. Lower priority takes precidence
    priority: u64,
    /// index of current sprite to be displayed
    current: usize,
    /// whether or not this sequence pauses/puts a hold on certain game events (for example changes in position) until it finishes (WIP)
    pause: bool,
    /// a list of sprites that make up this sequence
    sprites: Vec<Sprite>,
}

/// A collection of sequences that can be used to represent one figure/game entity, ex. all sequences (and therefore all sprites) depicting  Mario would be an entity
#[derive(Clone, Debug, Deserialize)]
pub struct Entity {
    /// the name of the entity being represented
    name: String,
    /// order of entity in sheet
    sheet_index: usize,
    /// the sequence that plays when no other sequences are active
    default_seq: usize,
    /// list of sequences
    seqs: Vec<Sequence>,
}

/// a specific instance of an entity, the link between animation and other logics/the game itself
pub struct AnimObject {
    /// the entity (all animation/sprite data) asosciated with this object
    entity: Entity,
    /// the rotation of this object
    rotation: f32,
    /// if the object is flipped on the x axis
    flip_x: bool,
    /// if the object is flipped on the y axis
    flip_y: bool,
    /// the pivot point of the object
    pivot: Option<Vec2>,
    /// the position of the object in the game
    pub pos: Vec2,
    /// whether or not this object is drawn
    pub is_visible: bool,
    /// whether or not this object is waiting for some sequence to finish before resuming regular behvaior (WIP)
    pub paused: bool,
}

/// a background element, is stationary throughout the game but still needs to be drawn does not have any asosciated sprites but instead is a simple rectangle
pub struct BackElement {
    /// the x and y position of the upper left hand corner
    pos: Vec2,
    /// the width and height of the element
    bounds: Vec2,
    /// the color of the element
    color: Color,
}

/// stores the image file for all sprites/visuals used in animations. stores the information gained from a json/datafile to access and use specific sprites from the image file
pub struct SpriteSheet {
    /// the image file loaded as a texture
    image: Texture2D,
    /// the list of entities that can be found in the image
    entities: Vec<Entity>,
}

impl Sequence {
    /// returns a clone of the current sprite
    fn cur_sprite(&self) -> Sprite {
        self.sprites[self.current]
    }

    /// moves the sequence forward, loops back to zero (the start) if needed and then makes the sequence inactive if not a cycle
    ///
    /// returns the state of the sequence (active/inactive)
    fn progress_seq(&mut self, frames_drawn: u64) -> bool {
        if frames_drawn % self.frame_rate == 0 {
            // increments current index
            self.current += 1;

            // if current is now out of bounds
            if self.current >= self.sprites.len() {
                //reset to begining sprite
                self.current = 0;

                if !self.is_cycle
                //if not a cycle
                {
                    self.is_active = false; //seq no longer active
                }
            }
        }
        self.is_active
    }
}

impl BackElement {
    pub fn new(x: f32, y: f32, w: f32, h: f32, color: Color) -> Self {
        Self {
            pos: Vec2::new(x, y),
            bounds: Vec2::new(w, h),
            color,
        }
    }

    /// draws background element
    fn draw(&self) {
        draw_rectangle(
            self.pos.x,
            self.pos.y,
            self.bounds.x,
            self.bounds.y,
            self.color,
        );
    }
}
impl SpriteSheet {
    fn new(image_file: Texture2D, data_file: Vec<Entity>) -> Self {
        Self {
            image: image_file,
            entities: data_file,
        }
    }

    /// returns a clone of a given entity
    pub fn get_entity(&self, entity_index: usize) -> Entity {
        self.entities[entity_index].clone()
    }
}

impl AnimObject {
    pub fn new(entity: Entity, pos: Vec2) -> Self {
        Self {
            entity,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
            pos,
            is_visible: true,
            paused: false,
        }
    }

    pub fn visible_true(&mut self) {
        self.is_visible = true;
    }

    pub fn visible_false(&mut self) {
        self.is_visible = false;
    }

    pub fn flip_x_true(&mut self) {
        self.flip_x = true;
    }

    pub fn flip_x_false(&mut self) {
        self.flip_x = false;
    }

    pub fn flip_y_true(&mut self) {
        self.flip_y = true;
    }

    pub fn flip_y_false(&mut self) {
        self.flip_y = false;
    }
    pub fn set_rotation(&mut self, new_rotation: f32) {
        self.rotation = new_rotation;
    }
    pub fn set_pivot(&mut self, new_pivot: Option<Vec2>) {
        self.pivot = new_pivot;
    }
    pub fn pause(&mut self) {
        self.paused = true;
    }
    pub fn unpause(&mut self) {
        self.paused = false;
    }

    /// creates a [DrawTextureParams][https://docs.rs/macroquad/0.3.4/macroquad/texture/struct.DrawTextureParams.html] for a given sprite
    fn create_param(&self, sprite: Sprite) -> DrawTextureParams {
        DrawTextureParams {
            dest_size: Some(Vec2::new(sprite.size.w as f32, sprite.size.h as f32)),
            source: Some(Rect::new(
                sprite.frame.x as f32,
                sprite.frame.y as f32,
                sprite.frame.w as f32,
                sprite.frame.h as f32,
            )),
            rotation: self.rotation,
            flip_x: self.flip_x,
            flip_y: self.flip_y,
            pivot: self.pivot,
        }
    }

    /// draws the object. image: the image file where the object's entity is located, frames_drawn: the number of times draw has been called for Animation (i.e. how many frames have been drawn total, outside of this object)
    fn draw(&mut self, image: Texture2D, frames_drawn: u64) {
        //start at default seq for object
        let mut cur_index = self.entity.default_seq;
        //set priority to default
        let mut cur_priority = self.entity.seqs[cur_index].priority;

        //loop through all seqs in the entity
        for (i, seq) in self.entity.seqs.iter().enumerate() {
            //if the seq is active and
            //takes precidence over the current seq
            if seq.is_active && seq.priority < cur_priority {
                //set as new current
                cur_priority = seq.priority;
                cur_index = i;
            }
        }

        //draw sprite based on seq
        draw_texture_ex(
            image,
            self.pos.x,
            self.pos.y,
            WHITE,
            self.create_param(self.entity.seqs[cur_index].cur_sprite()),
        );

        //is not just drawing default seq, default current
        if self.entity.seqs[cur_index].is_active {
            //if still active after progressing
            if self.entity.seqs[cur_index].progress_seq(frames_drawn) {
                //keep paused in accordance with active seq
                self.paused = self.entity.seqs[cur_index].pause;
            }
            //if the seq that just finished pauses the object
            else if self.entity.seqs[cur_index].pause {
                //unpause
                self.paused = false
            }
        }
    }
}
impl SimpleAnim {
    /// takes a image file and data file and loads then into a spritesheet
    pub fn new(image_file: &str, data_file: &str) -> Self {
        Self {
            sheet: SpriteSheet::new(
                block_on(macroquad::texture::load_texture(image_file)).expect("error reading"),
                serde_json::from_str(&fs::read_to_string(data_file).unwrap())
                    .expect("error while reading or parsing"),
            ),
            frames_drawn: 0,
            objects: Vec::new(),
            b_elements: Vec::new(),
            background_color: WHITE,
        }
    }

    /// increments frames drawn
    fn incr_frames(&mut self) {
        // makes sure no overflow
        if self.frames_drawn < u64::MAX {
            self.frames_drawn += 1;
        } else {
            self.frames_drawn = 0;
        }
    }

    //turns a seq state to true
    pub fn seq_true(&mut self, obj_index: usize, seq_index: usize) {
        self.objects[obj_index].entity.seqs[seq_index].is_active = true;
    }

    //turns seq state to false
    pub fn seq_false(&mut self, obj_index: usize, seq_index: usize) {
        self.objects[obj_index].entity.seqs[seq_index].is_active = false;
    }

    //turns a seq state to true
    pub fn activate_seq(&mut self, obj_index: usize, seq_name: &str) {
        for seq in self.objects[obj_index].entity.seqs.iter_mut() {
            if seq.seq_name.eq(seq_name) {
                seq.is_active = true;
            }
        }
    }
    //turns seq state to false
    pub fn deactivate_seq(&mut self, obj_index: usize, seq_name: &str) {
        for seq in self.objects[obj_index].entity.seqs.iter_mut() {
            if seq.seq_name.eq(seq_name) {
                seq.is_active = false;
            }
        }
    }

    pub fn set_background_color(&mut self, new_color: Color) {
        self.background_color = new_color;
    }

    // draws a current frame, i.e. background + all visible objects
    pub fn draw(&mut self) {
        clear_background(self.background_color);

        //draws background elements
        for element in self.b_elements.iter_mut() {
            element.draw();
        }

        //draws all visible objects
        for obj in self.objects.iter_mut() {
            if obj.is_visible {
                obj.draw(self.sheet.image, self.frames_drawn);
            }
        }
        self.incr_frames();
    }
}
