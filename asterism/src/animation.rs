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

//sprite sheet
pub struct SpriteSheet {
    image: Texture2D,
    data: Vec<Sprite>,
    start_sprite: Vec<usize>,
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
    pub async fn load_sprite_sheet(&mut self, image_file: &str, data_file: &str) -> () {
        let file = File::open(data_file).unwrap();
        let sprite_info: Vec<Sprite> =
            serde_json::from_reader(file).expect("error while reading or parsing");
        self.sheet = Some(SpriteSheet::new(image_file, sprite_info).await);
    }

    //work on naming
    //assigns a base row for different sprites
    pub fn assign_rows(&mut self, assignments: Vec<usize>) -> () {
        match &mut self.sheet {
            None => {}
            Some(s_sheet) => {
                for i in assignments.iter() {
                    s_sheet.assign_sprite(*i);
                }
            }
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
    pub fn draw_sprite(&self, x_pos: f32, y_pos: f32, start_index: usize, dist: usize) -> () {
        draw_texture_ex(
            self.sheet.as_ref().unwrap().image,
            x_pos,
            y_pos,
            WHITE,
            self.sheet
                .as_ref()
                .unwrap()
                .create_param(self.sheet.as_ref().unwrap().start_sprite[start_index] + dist),
        );
    }
}
