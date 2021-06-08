//! #Animation
//!
//! Animation handles the sprites and images to be rendered as well as handles
//! the process of rendering them

use crate::entity_state::FlatEntityState;
use json::*;
use macroquad::prelude::*;
use serde;
use serde::Deserialize;
use serde_json;
use std::fs::File;

//simple animations across a spritesheet
//animations draw on a set frame cycle
pub struct SimpleAnim {
    pub sheet: Option<SpriteSheet>,
    pub frames_drawn: u8,
    frame_cycle: u8,
}

#[derive(Debug, Deserialize)]
struct Rectangle {
    x: u64,
    y: u64,
    w: u64,
    h: u64,
}

#[derive(Debug, Deserialize)]
struct Size {
    w: u64,
    h: u64,
}

#[derive(Debug, Deserialize)]
struct Cycle {
    seq_index: u64,
    state: bool,
    priority: u64,
}

#[derive(Debug, Deserialize)]
struct Sprite {
    index: u64,
    frame: Rectangle,
    rotated: bool,
    trimmed: bool,
    size: Size,
}

#[derive(Debug, Deserialize)]
struct Sequence {
    name: String,
    current: u64,
    sprites: Vec<Sprite>,
}

#[derive(Debug, Deserialize)]
struct Entity {
    name: String,
    index: u64,
    default_seq: u64,
    cycles: Vec<Cycle>,
    seqs: Vec<Sequence>,
}

//sprite sheet
pub struct SpriteSheet {
    image: Texture2D,
    entities: Vec<Entity>,
}

impl SpriteSheet {
    async fn new(image_file: &str, data_file: Vec<Entity>) -> Self {
        Self {
            image: load_texture(image_file).await,
            entities: data_file,
        }
    }

    fn create_param(&self, index: usize) -> DrawTextureParams {
        let mut texture = DrawTextureParams::default();
        texture.dest_size = Some(Vec2::new(
            self.data[index].source_size.w as f32,
            self.data[index].source_size.h as f32,
        ));
        texture.source = Some(Rect::new(
            self.data[index].frame.x as f32,
            self.data[index].frame.y as f32,
            self.data[index].frame.w as f32,
            self.data[index].frame.h as f32,
        ));

        return texture;
    }

    fn progress_seq(&mut self, entity_index: u64, seq_index: u64) -> () {
        let seq = self.entities[entity_index][seq_index];
        if seq.current < seq.sprites.len() - 1 {
            seq.current = seq.current + 1;
        } else {
            seq.current = 0;
        }
    }
}

impl SimpleAnim {
    pub fn new() -> Self {
        Self {
            sheet: None,
            frames_drawn: 0,
            frame_cycle: 0,
        }
    }

    //Takes an image file and json file description of image file.
    //Loads a sprite sheet
    pub async fn load_sprite_sheet(&mut self, image_file: &str, data_file: &str) -> () {
        let file = File::open(data_file).unwrap();
        let sprite_info: Vec<Entity> =
            serde_json::from_reader(file).expect("error while reading or parsing");
        self.sheet = Some(SpriteSheet::new(image_file, sprite_info).await);
    }

    pub fn set_frames(&mut self, cycle: u8) -> () {
        self.frame_cycle = cycle;
    }

    pub fn incr_frames(&mut self) -> () {
        if self.frames_drawn >= self.frame_cycle {
            self.frames_drawn = 0;
        } else {
            self.frames_drawn = self.frames_drawn + 1;
        }
    }

    //determines if frames need to be switched
    pub fn switch_frame(&self) -> bool {
        return self.frames_drawn == 0;
    }

    pub fn sheet_loaded(&self) -> bool {
        match &self.sheet {
            None => {
                return false;
            }
            Some(_) => {
                return true;
            }
        }
    }

    //dist, of actual sprite to be drawn from start index on sprite sheet
    fn draw_sprite(&mut self, x_pos: f32, y_pos: f32, entity_index: u64, seq_index: u64) -> () {
        draw_texture_ex(
            self.sheet.as_ref().unwrap().image,
            x_pos,
            y_pos,
            WHITE,
            self.sheet.as_ref().unwrap().create_param(
                self.sheet.as_ref().unwrap().entities[entity_index][seq_index].current,
            ),
        );
        self.sheet.progress_seq(entity_index, seq_index);
    }

    pub fn draw_entity(&self, x_pos: f32, y_pos: f32, entity_index: u64) {
        let cur_cycle = Cycle::new(); //creates blank cycle

        //goes through all the cycles for the entity
        for (cycle, i) in self.sheet.as_ref().unwrap().entities[entity_index]
            .cycles
            .iter()
            .enurmerate()
        {
            //if the cycle state is true
            if cycle.state {
                if i == 0 {
                    cur_cycle = cycle;
                } else {
                    //if priorirty is less than the previous priority
                    if cycle.priority < cur_cycle.priority {
                        cur_cycle = cycle;
                    }
                }
            }
        }
        //draw the sprite determinded
        draw_sprite(x_pos, y_pos, entity_index, cur_cycle.seq_index);
    }
}
