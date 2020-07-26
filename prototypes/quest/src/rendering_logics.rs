use super::vector::Vector;
use super::make_square::render_thing;
use super::Vertex;

pub enum TextureActions {
		RevertToBase(u32),
		Highlight(u32),
}

#[derive(Debug)]
struct TextureThing {
		current_texture: u32,
		
		base_texture: u32,
		highlight_texture: Option<u32>,
}

impl TextureThing {
		fn new(base_texture: u32, highlight_texture: Option<u32>) -> TextureThing {
				TextureThing {
						current_texture: base_texture,
						base_texture,
						highlight_texture,
				}
		}
		
		pub fn get_texture(&self) -> u32 {
				self.current_texture
		}

		pub fn highlight(&mut self) {
				match self.highlight_texture {
						Some(texture_number) => self.current_texture = texture_number,
						None => panic!("Yikes, attempted to highlight an unhighlightable object")
				}
		}

		pub fn return_to_base_texture(&mut self) {
				self.current_texture = self.base_texture;
		}
		
}

// render--------------------------------------
#[derive(Debug)]
pub struct RenderableComponent {
		position: Option<Vector>,
		facing: Vector,
		size: f32,
		texture: TextureThing,
/*
would like to have it store the result of its previous render to speed up program, only re-render those assets that need it
		*/
		//		z_index: u8, <-may be needed to preserve front/back stuff if list is
		// getting shuffled to render...?
}

impl RenderableComponent {
		pub fn new(position: Option<Vector>,
					 facing: Vector,
					 size: f32,
					 texture: (u32, Option<u32>),) -> RenderableComponent {
				RenderableComponent {
						position,
						facing,
						size,
						texture: TextureThing::new(texture.0, texture.1),
				}
		}
		fn hide(&mut self) {
				self.size = 0_f32;
		}
		fn update_coordinates(&mut self, new_coords: Vector) {
				self.position = if (new_coords.x.abs() - (self.size / 2_f32)) < 1_f32
						|| (new_coords.y.abs() - (self.size / 2_f32)) < 1_f32 {
								Some(new_coords)
						} else {
								None
						}
		}
		// rendering everything as squares not going to work if empty -> black
		fn render(&self) -> Option<(Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u16>)> {
				match &self.position {
						Some(i) => Some(render_thing(&i, &self.facing, self.size, self.texture.get_texture())),
						None => None,
				}
		}
}

#[derive(Debug)]
pub struct RenderableComponentVec {
		parts: Vec<RenderableComponent>,
}

impl RenderableComponentVec {
		pub fn new() -> RenderableComponentVec {
				RenderableComponentVec {
						parts: Vec::new(),
				}
		}
		pub fn add(&mut self, new_component: RenderableComponent) {
				self.parts.push(new_component);
		}
		pub fn hide(&mut self, thing_to_be_hidden_id: u32) {
				self.parts[thing_to_be_hidden_id as usize].hide();
		}
		pub fn update_all_coords(&mut self, new_coords: Vec<Vector>) {
				for i in 0..self.parts.len() {
						self.parts[i].update_coordinates(new_coords[i])
				}
		}
		pub fn update_textures(&mut self, texture_actions: Vec<TextureActions>) {
				for action in texture_actions {
						match action {
								TextureActions::Highlight(index) => self.parts[index as usize].texture.highlight(),
								TextureActions::RevertToBase(index) => self.parts[index as usize].texture.return_to_base_texture(),
						}
				}
		}
		pub fn render_all(&self) -> (Vec<Vertex>, Vec<u16>) {
				let mut indices_vec: Vec<u16> = Vec::new();
				let mut vertices_vec: Vec<Vertex> = Vec::new();
				let mut indices_counter: u16 = 0;
				for thing in self.parts.iter() {
						let potential_placeholder: Option<(Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u16>)> = thing.render();
						match potential_placeholder {
								Some(i) => {
										let placeholder = i;

										for j in placeholder.2.iter() {
												indices_vec.push(j + indices_counter * 4);
										}
										indices_counter += 1;

										let combined_vertices: Vec<([f32; 3], [f32; 2])> = placeholder.0.into_iter()
												.zip(placeholder.1.into_iter())
												.collect();
										for (pos, coords) in combined_vertices.iter() {
												vertices_vec.push(
														Vertex { position: *pos, tex_coords: *coords }
												);
										}
								},
								None => (),
						}
				}
		
				(vertices_vec, indices_vec)
		}
		pub fn get_selection_info(&self) -> Vec<Option<u32>> {
				let mut selection_info_vec: Vec<Option<u32>> = Vec::new();
				for part in &self.parts {
						selection_info_vec.push(part.texture.highlight_texture)
				}
				selection_info_vec
		}
		pub fn get_nearest_to_coords(&self, potential_target_coords: Option<Vector>/*, indices_list: Vec<u32>*/) -> Option<u32> {
				match potential_target_coords {
						Some(coords) => {
								let indices_list: Vec<u32> = vec![38, 46];
								let target_coords: Vector = coords;
								let mut nearest: Option<(u32, f32)> = None;
								for index in indices_list {
										match self.parts[index as usize].position {
												Some(coords) => {
														match nearest {
																Some((_, prev_closest)) => {
																		let new_distance: f32 = coords.distance_from(&target_coords);
																		if new_distance < prev_closest {
																				nearest = Some((
																						index,
																						new_distance
																				));
																		}
																},
																None => {
																		nearest = Some((
																				index,
																				coords.distance_from(&target_coords)
																		));
																},
														}
												},
												None => (),
										}
								}
								match nearest {
										Some((index, _)) => Some(index),
										None => None,
								}
						},
						None => None,
				}
		}
}
