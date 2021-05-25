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
    pub sheet: Some(SpriteSheet),
    pub frames_drawn: u8,
    frame_cycle: u8,
}

//sprite sheet
struct SpriteSheet {
    image: Texture2D,
    data: Vec<Sprite>,
    start_sprite: Vec<usize>,
}

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
struct Sprite {
    name: String,
    frame: Rectangle,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: Rectangle,
    source_size: Size,
}

impl SpriteSheet {
    async fn new(image_file: &str, data_file: Vec<Sprite>) -> Self {
        Self {
            image: load_texture(image_file).await,
            data: data_file,
            start_sprite: Vec::new(),
        }
    }

    //the base/starting sprite for the sprite
    fn assign_sprite(&mut self, assignment: usize) -> () {
        self.start_sprite.push(assignment);
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
    pub async fn load_sprite_sheet(image_file: &str, data_file: File) -> () {
        let sprite_info: Vec<Sprite> =
            serde_json::from_reader(file).expect("error while reading or parsing");
        self.sheet = Some(SpriteSheet::new(image_file, sprite_info)).await;
    }

    //work on naming
    //assigns a base row for different sprites
    pub fn assign_rows(assignments: Vec<usize>) -> () {
        for i in assignments.iter() {
            self.sheet.assign_sprite(i);
        }
    }

    pub fn set_frames(cycle: u8) -> () {
        self.frame_cycle = cycle;
    }

    pub fn incr_frames() -> () {
        if self.frames_drawn >= self.frame_cycle {
            self.frames_drawn = 0;
        } else {
            self.frames_drawn = self.frames_drawn + 1;
        }
    }

    //determines if frames need to be switched
    pub fn switch_frame() -> bool {
        return self.frames_drawn == 0;
    }
}
