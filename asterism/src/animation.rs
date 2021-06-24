//! #Animation
//!
//! Animation handles the sprites and images to be rendered as well as handles
//! the process of rendering them

use json::*;
use macroquad::prelude::*;
use serde;
use serde::Deserialize;
use serde_json;
use std::fs;

//simple animations across a spritesheet
pub struct SimpleAnim {
    pub sheet: SpriteSheet,
    pub frames_drawn: u64,
    pub objects: Vec<AnimObject>
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

#[derive(Clone, Copy, Debug, Deserialize)]
struct Sprite {
    index: usize,
    frame: Rectangle,
    trimmed: bool,
    size: Size,
}

#[derive(Clone, Debug, Deserialize)]
struct Sequence {
    seq_name: String,
    frame_rate: u64,
    is_cycle: bool,
    is_active: bool,
    priority: u64,
    current: usize,
    sprites: Vec<Sprite>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Entity {
    name: String,
    sheet_index: usize,
    default_seq: usize,
    seqs: Vec<Sequence>,
}

pub struct AnimObject
{
    entity: Entity,
    rotation: f32,
    flip_x: bool,
    flip_y: bool,
    pivot: Option<Vec2>,
    pub pos: Vec2,
    pub is_visible: bool,
}

//sprite sheet
pub struct SpriteSheet {
    image: Texture2D,
    entities: Vec<Entity>,
}

impl Sequence
{
    fn cur_sprite (&self) -> Sprite
    {
	return self.sprites[self.current].clone();
    }
    fn progress_seq (&mut self, frames_drawn:u64) -> ()
    {
	if frames_drawn % self.frame_rate == 0
	{
	    //incriments current index
	    self.current = self.current + 1;

	    //if current is now out of bounds
	    if self.current >= self.sprites.len()
	    {
		//reset to begining sprite
		self.current = 0;

		if !self.is_cycle//if not a cycle
		{
		   self.is_active = false; //seq no longer active
		}
	    }
	}
    }
}
impl SpriteSheet {
    fn new(image_file: Texture2D, data_file: Vec<Entity>) -> Self {
        Self {
            image: image_file,
            entities: data_file,
        }
    }

    pub fn get_entity(&self, entity_index: usize) -> Entity
    {
	return self.entities[entity_index].clone();
    }
}

impl AnimObject
{
    pub fn new(entity: Entity, pos: Vec2) -> Self {
	Self {
	    entity: entity,
	    rotation: 0.0,
	    flip_x: false,
	    flip_y: false,
	    pivot: None,
	    pos: pos,
	    is_visible: true,
	}
    }

    pub fn visible_true(&mut self) -> ()
    {
	self.is_visible = true;
    }

     pub fn visible_false(&mut self) -> ()
    {
	self.is_visible = false;
    }
    
    pub fn flip_x_true (&mut self) ->()
    {
	self.flip_x = true;
    }

     pub fn flip_x_false (&mut self) ->()
    {
	self.flip_x = false;
    }

    pub fn flip_y_true (&mut self) ->()
    {
	self.flip_y = true;
    }

     pub fn flip_y_false (&mut self) ->()
    {
	self.flip_y = false;
    }
    pub fn set_rotation (&mut self, new_rotation: f32) ->()
    {
	self.rotation = new_rotation;
    }
    pub fn set_pivot (&mut self, new_pivot: Option<Vec2>) ->()
    {
	self.pivot = new_pivot;
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
	 texture.rotation = self.rotation;
	 texture.flip_x = self.flip_x;
	 texture.flip_y = self.flip_y;
	 texture.pivot = self.pivot;

        return texture;
     }

    fn draw(&mut self, image: Texture2D, frames_drawn: u64) -> ()
    {
	let mut cur_index = self.entity.default_seq;
	let mut cur_priority = self.entity.seqs[cur_index].priority;

	for (i, seq) in self.entity.seqs.iter().enumerate()
	{
	    if seq.is_active && seq.priority < cur_priority
	    {
		cur_priority = seq.priority;
		cur_index = i;
	    }
	}

	draw_texture_ex(image,
			self.pos.x,
			self.pos.y,
			WHITE,
			self.create_param(self.entity.seqs[cur_index].cur_sprite()));

	self.entity.seqs[cur_index].progress_seq(frames_drawn);

			
    }

    
}
impl SimpleAnim {
    pub async fn new(image_file: &str, data_file: &str) -> Self {
        Self {
            sheet: SpriteSheet::new(
                macroquad::texture::load_texture(image_file)
                    .await
                    .expect("error reading"),
                serde_json::from_str(&fs::read_to_string(data_file).unwrap())
                    .expect("error while reading or parsing"),
            ),
            frames_drawn: 0,
	    objects: Vec::new(),
        }
    }


    //incriments frames drawn
    pub fn incr_frames(&mut self) -> () {

	if self.frames_drawn < u64::MAX
	{
	    self.frames_drawn = self.frames_drawn +1; 
	}
	else
	{
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
	for seq in self.objects[obj_index].entity.seqs.iter_mut()
	    {
		if seq.seq_name.eq(seq_name)
		{
		    seq.is_active = true;
		}
	    }
    }
    //turns seq state to false
    pub fn deactivate_seq(&mut self, obj_index: usize, seq_name: &str) {
       
	    for seq in self.objects[obj_index].entity.seqs.iter_mut()
	    {
		if seq.seq_name.eq(seq_name)
		{
		    seq.is_active = false;
		}
	    }
	
    }

    pub fn draw(&mut self) -> ()
    {
	for obj in self.objects.iter_mut()
	{
	    if obj.is_visible{
		obj.draw(self.sheet.image, self.frames_drawn);
	    }
	}
	 self.incr_frames();
    }
        
}
