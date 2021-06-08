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
    pub sheet: SpriteSheet,
    pub frames_drawn: u8,
    frame_cycle: u8,
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

#[derive(Debug, Deserialize)]
struct Cycle {
    seq_index: usize,
    state: bool,
    priority: u64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct Sprite {
    index: usize,
    frame: Rectangle,
    rotated: bool,
    trimmed: bool,
    size: Size,
}

#[derive(Debug, Deserialize)]
struct Sequence {
    seq_name: String,
    cycle_index: usize,
    current: usize,
    sprites: Vec<Sprite>,
}

#[derive(Debug, Deserialize)]
struct Entity {
    name: String,
    index: usize,
    default_seq: usize,
    cycles: Vec<Cycle>,
    seqs: Vec<Sequence>,
}

//sprite sheet
pub struct SpriteSheet {
    image: Texture2D,
    entities: Vec<Entity>,
}

impl Cycle {
    fn new() -> Self {
        Self {
            seq_index: 0,
            state: false,
            priority: 255,
        }
    }
}

impl SpriteSheet {
    async fn new(image_file: &str, data_file: Vec<Entity>) -> Self {
        Self {
            image: load_texture(image_file).await,
            entities: data_file,
        }
    }

    fn create_param(&self, sprite: Sprite) -> DrawTextureParams {
        let mut texture = DrawTextureParams::default();
        texture.dest_size = Some(Vec2::new(sprite.size.w as f32, sprite.size.h as f32));
        texture.source = Some(Rect::new(
            sprite.frame.x as f32,
            sprite.frame.y as f32,
            sprite.frame.w as f32,
            sprite.frame.h as f32,
        ));

        return texture;
    }

    fn progress_seq(&mut self, entity_index: usize, seq_index: usize) -> () {
        if self.entities[entity_index].seqs[seq_index].current
            < self.entities[entity_index].seqs[seq_index].sprites.len() - 1
        {
            self.entities[entity_index].seqs[seq_index].current =
                self.entities[entity_index].seqs[seq_index].current + 1;
        } else {
            self.entities[entity_index].seqs[seq_index].current = 0;
        }
    }
}

impl SimpleAnim {
    pub async fn new(image_file: &str, data_file: &str) -> Self {
        Self {
            sheet: SpriteSheet::new(
                image_file,
                serde_json::from_reader(File::open(data_file).unwrap())
                    .expect("error while reading or parsing"),
            )
            .await,
            frames_drawn: 0,
            frame_cycle: 0,
        }
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

    //dist, of actual sprite to be drawn from start index on sprite sheet
    fn draw_sprite(
        &mut self,
        x_pos: f32,
        y_pos: f32,
        is_cycle: bool,
        entity_index: usize,
        seq_index: usize,
    ) -> () {
        draw_texture_ex(
            self.sheet.image,
            x_pos,
            y_pos,
            WHITE,
            self.sheet.create_param(
                self.sheet.entities[entity_index].seqs[seq_index].sprites
                    [self.sheet.entities[entity_index].seqs[seq_index].current],
            ),
        );

        if is_cycle && self.switch_frame() {
            self.sheet.progress_seq(entity_index, seq_index);
        }
    }

    pub fn draw_entity(&mut self, x_pos: f32, y_pos: f32, entity_index: usize) {
        let mut cur_cycle = &Cycle::new(); //creates blank cycle
        let mut active_cycle = false; //is there a cycle that has been triggered

        //goes through all the cycles for the entity
        for (i, cycle) in self.sheet.entities[entity_index].cycles.iter().enumerate() {
            //if the cycle state is true
            if cycle.state {
                active_cycle = true;
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
        if active_cycle {
            self.draw_sprite(x_pos, y_pos, true, entity_index, cur_cycle.seq_index);
        }
        //no active cycles so draw default seq
        else {
            self.draw_sprite(
                x_pos,
                y_pos,
                false,
                entity_index,
                self.sheet.entities[entity_index].default_seq,
            );
        }
    }
}
