use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};

mod texture;
mod vector;
use vector::Vector;
mod make_square;

mod rendering_logics;
use rendering_logics::{RenderableComponent, RenderableComponentVec};

mod story;
use story::{StoryPhase, StoryManager};

mod example;

pub struct Game {
		story: StoryManager,
		
		view_area: ViewArea,
		
		renderable_components: RenderableComponentVec,
		position_components: PositionComponentVec,
		selection_components: SelectionComponentVec,
		
		user_inputs: UserInputs,
}
impl Game {
		pub fn new() -> Game {
				let story: StoryManager = StoryManager::new();
				let position_component_vec = PositionComponentVec::new();
				let renderable_component_vec = RenderableComponentVec::new();
				let selection_component_vec = SelectionComponentVec::new(renderable_component_vec.get_selection_info());
				let user_inputs = UserInputs::new();
				let mut game: Game = Game {
						story: story,
						view_area: ViewArea::new(),
						position_components: position_component_vec,
						renderable_components: renderable_component_vec,
						selection_components: selection_component_vec,
						user_inputs: user_inputs,
				};
				game.add_things(example::make_example_1());
				game
				// println!("{:?}", game.renderable_components);
		}
		pub fn add_things(&mut self, new_things_list: Vec<((f32, f32), (Vector, f32, (u32, Option<u32>)))>) {
				for thing in new_things_list.into_iter() {
						let pos_x: f32 = (thing.0).0;
						let pos_y: f32 = (thing.0).1;

						let facing: Vector = (thing.1).0;
						let size: f32 = (thing.1).1;
						let texture: (u32, Option<u32>) = (thing.1).2;
						
						self.position_components.add(
								PositionComponent::new(
										pos_x, pos_y
								)
						);

						self.renderable_components.add(
								RenderableComponent::new(
										None, facing, size, texture
								)
						);

						self.selection_components.add(Some(false));
				}
				self.renderable_components
						.update_all_coords(self.position_components.get_render_positions(&self.view_area));
		}
		/*fn add_test_thing(&mut self) {
				self.position_components.add(
						PositionComponent::new(1_f32, 1_f32)
				);
				self.renderable_components.add(
						RenderableComponent::new(None, Vector::new(1_f32, 1_f32), 0.2, 0_u32)
				);
				self.renderable_components.update_all_coords(&self.view_area, &self.position_components);
		}*/
		fn render(&self) -> (Vec<Vertex>, Vec<u16>) {				
				self.renderable_components.render_all()
		}
		fn update(&mut self) {
				match self.story.get_current_phase() {
						StoryPhase::Menu => {
								if self.user_inputs.click {
										self.story.advance_phase();
										self.renderable_components.hide(65);
								}
						},
						StoryPhase::PlayerMove => {
								self.view_area.update_using_input(&self.user_inputs);
								
								self.renderable_components
										.update_all_coords(self.position_components.get_render_positions(&self.view_area));
								self.renderable_components
										.update_textures(self.selection_components.highlight_under_mouse(self.renderable_components.get_nearest_to_coords(self.user_inputs.mouse_coords)));
						},
				}
				self.user_inputs.click = false;
		}
}

// view area

struct ViewArea {
		// location of the center of the view
		position: Vector,
}

impl ViewArea {
		// only here until i implement zooming
		const LENGTH: f32 = 100_f32;
		const WIDTH: f32 = 100_f32;
		const SPEED: f32 = 0.1;
		fn new() -> ViewArea {
				ViewArea::default()
		}
		fn pos_cords_to_render_cords(&self, thing_position: &PositionComponent) -> (f32, f32) {
				let potential_render_x: f32 = (thing_position.x - self.position.x) / (ViewArea::LENGTH / 2_f32);
				let potential_render_y: f32 = -(thing_position.y - self.position.y) / (ViewArea::WIDTH / 2_f32);
				(potential_render_x, potential_render_y)
		}
		fn update_using_input(&mut self, user_inputs: &UserInputs) {
				let mut movement_vec = Vector::new(0_f32, 0_f32);
				if user_inputs.va_up {
						movement_vec.y += -1_f32;
				}
				if user_inputs.va_down {
						movement_vec.y += 1_f32;
				}
				if user_inputs.va_left {
						movement_vec.x += -1_f32;
				}
				if user_inputs.va_right {
						movement_vec.x += 1_f32;
				}

				if movement_vec.length() != 0_f32 {
						movement_vec = movement_vec.normalize().scale_by(ViewArea::SPEED);
				}

				self.position = movement_vec.add(&self.position);
				
				// level needs bounds
				self.position.x = self.position.x.max(-50_f32 + (ViewArea::LENGTH / 2_f32));
				self.position.x = self.position.x.min(150_f32 - (ViewArea::LENGTH / 2_f32));

				self.position.y = self.position.y.max(-50_f32 + (ViewArea::WIDTH / 2_f32));
				self.position.y = self.position.y.min(150_f32 - (ViewArea::WIDTH / 2_f32));
		}
}

impl Default for ViewArea {
		fn default() -> ViewArea {
				ViewArea {
						position: Vector {
								x: 50_f32,
								y: 50_f32,
						}
				}
		}
}

// position
#[derive(Debug)]
struct PositionComponent {
		x: f32,
		y: f32,
}

impl PositionComponent {
		fn new(x: f32, y: f32) -> PositionComponent {
				PositionComponent {
						x,
						y,
				}
		}
/*		fn move_left(&mut self) {
				self.x += 0.01;
		}*/
}

#[derive(Debug)]
struct PositionComponentVec {
		parts: Vec<PositionComponent>,
}

impl PositionComponentVec {
/*		fn slide_to_the_left(&mut self) {
				/*for i in 0..self.parts.len() {
						self.parts[i].move_left();
				}*/
				self.parts[64].move_left();
		}*/
		fn new() -> PositionComponentVec {
				PositionComponentVec {
						parts: Vec::new(),
				}
		}
		fn add(&mut self, position_component: PositionComponent) {
				self.parts.push(position_component);
		}
		fn get_render_positions(&self, view_area: &ViewArea) -> Vec<Vector> {
				let mut render_positions: Vec<Vector> = Vec::new();
				for i in 0..self.parts.len() {
						let (render_x, render_y): (f32, f32) = view_area
								.pos_cords_to_render_cords(&self.parts[i]);
						render_positions.push(Vector::new(render_x, render_y));
				}
				//println!("{:?}", render_positions);
				render_positions
		}
}

// input---------------------------------------

struct UserInputs {
		pub va_up: bool,
		pub va_down: bool,
	  pub va_left: bool,
		pub va_right: bool,
		pub mouse_coords: Option<Vector>,
		pub click: bool,
}

impl UserInputs {
		fn new() -> UserInputs {
				UserInputs {
						va_up: false,
						va_down: false,
						va_left: false,
						va_right: false,
						mouse_coords: None,
						click: false,
				}
		}
}

use rendering_logics::TextureActions;

pub struct SelectionComponentVec {
		parts: Vec<SelectionComponent>,
		previously_highlighted: Option<usize>,
}

impl SelectionComponentVec {
		fn new(highlight_textures: Vec<Option<u32>>) -> SelectionComponentVec {
				let mut parts_constructor: Vec<SelectionComponent> = Vec::new();
				for texture in highlight_textures {
						match texture {
								Some(_) => parts_constructor.push(
										SelectionComponent {
												is_highlighted: Some(false)
										}
								),
								None => parts_constructor.push(
										SelectionComponent {
												is_highlighted: None
										}
								),
						}
				}

				SelectionComponentVec {
						parts: parts_constructor,
						previously_highlighted: None,
				}
		}
		fn add(&mut self, initial_state: Option<bool>) {
				self.parts.push(SelectionComponent{
						is_highlighted: initial_state,
				});
		}
		fn highlight_under_mouse(&mut self, nearest_mouse_index: Option<u32>) -> Vec<TextureActions> {
				let mut texture_changes: Vec<TextureActions> = Vec::new();
				
				match self.previously_highlighted {
						Some(index) => {
								if self.parts[index].unhighlight() {
										texture_changes.push(TextureActions::RevertToBase(self.previously_highlighted.unwrap() as u32));
								}
						},
						None => (),
				};

				match nearest_mouse_index {
						Some(index) => {
								if self.parts[index as usize].highlight() {
										texture_changes.push(TextureActions::Highlight(index));
										self.previously_highlighted = Some(index as usize);
								}
						},
						None => (),
				};
				
				texture_changes
		}
}

pub struct SelectionComponent {
		is_highlighted: Option<bool>,
}

impl SelectionComponent {
		fn highlight(&mut self) -> bool {
				match self.is_highlighted {
						Some(state) => {
								if state {
										false
								} else {
										self.is_highlighted = Some(true);
										true
								}
						},
						None => false,
				}
		}

		fn unhighlight(&mut self) -> bool {
				match self.is_highlighted {
						Some(state) => {
								if state {
										self.is_highlighted = Some(false);
										true
								} else {
										false
								}
						},
						None => false,
				}
		}
}



//-----------------------------------------------------------------------------------------


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
            ]
        }
    }
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

		game: Game,
		
		#[allow(dead_code)]
		diffuse_texture: texture::Texture,
		diffuse_bind_group: wgpu::BindGroup,

    size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY, // Vulkan + Metal + DX12 + Browser WebGPU
        ).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: Default::default(),
        }).await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

				let diffuse_bytes = include_bytes!("texture.png");
				let (diffuse_texture, cmd_buffer) = texture::Texture::from_bytes(
						&device,
						diffuse_bytes,
						"texture.png"
				).unwrap();

				queue.submit(&[cmd_buffer]);

				let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
						bindings: &[
								wgpu::BindGroupLayoutEntry {
										binding: 0,
										visibility: wgpu::ShaderStage::FRAGMENT,
										ty: wgpu::BindingType::SampledTexture {
												multisampled: false,
												dimension: wgpu::TextureViewDimension::D2,
												component_type: wgpu::TextureComponentType::Uint,
										},
								},
								wgpu::BindGroupLayoutEntry {
										binding: 1,
										visibility: wgpu::ShaderStage::FRAGMENT,
										ty: wgpu::BindingType::Sampler {
												comparison: false,
										},
								},
						],
						label: Some("texture_bind_group_layout")
				});

				let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

				
				
        let vs_src = include_str!("shader.vert");
        let fs_src = include_str!("shader.frag");
        let mut compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = compiler.compile_into_spirv(vs_src, shaderc::ShaderKind::Vertex, "shader.vert", "main", None).unwrap();
        let fs_spirv = compiler.compile_into_spirv(fs_src, shaderc::ShaderKind::Fragment, "shader.frag", "main", None).unwrap();
        let vs_data = wgpu::read_spirv(std::io::Cursor::new(vs_spirv.as_binary_u8())).unwrap();
        let fs_data = wgpu::read_spirv(std::io::Cursor::new(fs_spirv.as_binary_u8())).unwrap();
        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&texture_bind_group_layout],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
           /* color_states: &[
                wgpu::ColorStateDescriptor {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                },
        ],*/
						color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,                 
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,                 
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[
                    Vertex::desc(),
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

				/*let (vertices, indices) = (&[
						Vertex { position: [-0.7, 0.7, 0.0], tex_coords: [0.0, 0.0] }, // A
						Vertex { position: [-0.7, -0.7, 0.0], tex_coords: [0.0, 1.0] }, // B
						Vertex { position: [0.7, -0.7, 0.0], tex_coords: [1.0, 1.0] },
						Vertex { position: [0.7, 0.7, 0.0], tex_coords: [1.0, 0.0] }, // D
		], &[0 as u16, 1 as u16, 3 as u16, 1 as u16, 2 as u16, 3 as u16]);*/

				let game = Game::new();

				let (vertices, indices) = game.render();

				// println!("{:?}", indices);
				
				let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(vertices.as_slice()),
            wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        );
        let index_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(indices.as_slice()),
            wgpu::BufferUsage::INDEX | wgpu::BufferUsage::COPY_DST,
        );
        
        let num_indices = indices.len() as u32;
	
        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,
						vertex_buffer,
            index_buffer,
            num_indices,
						diffuse_texture,
						diffuse_bind_group,
            size,
						game,
        }
    }


    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

		#[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
				match event {
						WindowEvent::KeyboardInput {
								input: KeyboardInput {
										state,
										virtual_keycode: Some(virtual_key_code),
										..
								},
								..
						} => {
								match virtual_key_code {
										VirtualKeyCode::W => self.game.user_inputs.va_up = *state == ElementState::Pressed,
										VirtualKeyCode::S => self.game.user_inputs.va_down = *state == ElementState::Pressed,
										VirtualKeyCode::A => self.game.user_inputs.va_left = *state == ElementState::Pressed,
										VirtualKeyCode::D => self.game.user_inputs.va_right = *state == ElementState::Pressed,
										_ => {},
								}
								true
						},
						WindowEvent::CursorMoved {
								position,
								..
						} => {
								self.game.user_inputs.mouse_coords =
										Some(
												Vector::new(
														((position.x as f32 * 2_f32) - self.size.width as f32) / self.size.width as f32,
														-((position.y as f32 * 2_f32) - self.size.height as f32) / self.size.height as f32)
										);
								true
						},
						WindowEvent::MouseInput {
								device_id,
								state,
								button,
								..
						} => {
								if button == &MouseButton::Left && state == &ElementState::Released {
										self.game.user_inputs.click = true;
										true
								} else {
										false
								}
						},
						_ => false,
				}
    }

    fn update(&mut self) {
				self.game.update();
	
				let (vertices, indices) = self.game.render();

				self.vertex_buffer = self.device.create_buffer_with_data(
            bytemuck::cast_slice(vertices.as_slice()),
            wgpu::BufferUsage::VERTEX,
        );
				self.index_buffer = self.device.create_buffer_with_data(
            bytemuck::cast_slice(indices.as_slice()),
            wgpu::BufferUsage::INDEX,
        );
				self.num_indices = indices.len() as u32;
    }

    fn render(&mut self) {
        let frame = self.swap_chain.get_next_texture()
            .expect("Timeout getting texture");

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        },
                    }
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
						render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
            render_pass.set_index_buffer(&self.index_buffer, 0, 0);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(&[
            encoder.finish()
        ]);
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    use futures::executor::block_on;
    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    /*WindowEvent::KeyboardInput {
                        input,
                        ..
                    } => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    }*/
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &mut so w have to dereference it twice
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                state.render();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });

}
