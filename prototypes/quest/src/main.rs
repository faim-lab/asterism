use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};

mod texture;

trait Render {
		fn render(&self) -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u16>);
}

struct Game {
		things: Vec<Box<dyn Render>>,
}

struct Player {
		position: Vector,
		facing: Vector,
		size: f32,
		texture: u32,
}

impl Game {
		fn new() -> Game {
				let mut things: Vec<Box<dyn Render>> = Vec::new();
				things.push(Box::new(Player::new()));						
				Game {
						things,
				}
		}
}

impl Player {
		fn new() -> Player {
				Player {
						position: Vector::new(0.5, 0_f32),
						facing: Vector::new(1_f32, 0_f32),
						size: 0.2,
						texture: 0,
				}
		}
}

// Rendering stuff below here
impl Game {
		fn render_all(&self) -> (Vec<Vertex>, Vec<u16>) {
				let mut indices_vec: Vec<u16> = Vec::new();
				let mut vertices_vec: Vec<Vertex> = Vec::new();
				for thing in self.things.iter() {
						let mut placeholder: (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u16>) = thing.render();
						indices_vec.append(&mut placeholder.2);
						let combined_vertices: Vec<([f32; 3], [f32; 2])> = placeholder.0.into_iter()
								.zip(placeholder.1.into_iter())
								.collect();
						for (pos, coords) in combined_vertices.iter() {
								vertices_vec.push(
										Vertex { position: *pos, tex_coords: *coords }
								);
						}
				}
				(vertices_vec, indices_vec)
		}
}

impl Render for Player {
		fn render(&self) -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u16>) {
				render_thing(&self.position, &self.facing, self.size, self.texture)
		}
}

use math::round;

// we're rendering everything as squares ¯\_(ツ)_/¯
// coords are a Vector with -1<=x<=1, -1<=y<=1
// facing is a Vector representing the orientation of the object, objects face towards 
// size represents the the side length of the square rendering the object, same scale as coords
// tex_number is the number representing which texture will be applied to the square
fn render_thing(coords: &Vector, facing: &Vector, size: f32, tex_number: u32)
					-> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u16>) {
		let indices: Vec<u16> = vec!(0, 1, 3, 1, 2, 3);
		let mut vert_coords: Vec<Vector> = Vec::new();

		// gotta make the squares clockwise
		let og_tl = Vector::new(-1.0 as f32, 1.0 as f32);
		let og_bl = Vector::new(-1.0 as f32, -1.0 as f32);
		let og_br = Vector::new(1.0 as f32, -1.0 as f32);
		let og_tr = Vector::new(1.0 as f32, 1.0 as f32);

		// rotate and scale
		let tl: Vector = og_tl.rotate(&facing).scale_by(size / 2_f32).add(coords);
		let bl: Vector = og_bl.rotate(&facing).scale_by(size / 2_f32).add(coords);
		let br: Vector = og_br.rotate(&facing).scale_by(size / 2_f32).add(coords);
		let tr: Vector = og_tr.rotate(&facing).scale_by(size / 2_f32).add(coords);

		vert_coords.push(tl);
		vert_coords.push(bl);
		vert_coords.push(br);
		vert_coords.push(tr);

		let mut vertices: Vec<[f32; 3]> = Vec::new();

		for i in vert_coords.iter() {
				vertices.push([i.x, i.y, 0_f32]);
		}

		(vertices, get_tex_coords(tex_number), indices)
}

fn get_tex_coords(tex_number: u32) -> Vec<[f32; 2]> {
		
		// get those tex coords
		// textures oughta be in one png file, of uniform size, arranged thusly:
		/*
		_____________________
		\ 0 \ 1 \ 2 \ 3 \ 4 \
		_____________________
		\ 5 \ 6 \ 7 \ 8 \ 9 \
		_____________________
		etc
		 */
		// also the png file should be a perfect square, leave blank space at the bottom if need be
		
		// number of textures per line, change this to match your texture file
		let tex_per_line: f32 = 5_f32;

		// calculated from the above
		let mut tex_coords: Vec<[f32; 2]> = Vec::new();
		let tex_width: f32 = 1_f32 / tex_per_line;
		let tl_tex = [
				((tex_number as f32 % tex_per_line) * tex_width),
				(round::floor(tex_number as f64 / tex_per_line as f64, 0) * 0.2) as f32
		];
		let bl_tex = [tl_tex[0], tl_tex[1] + tex_width];
		let br_tex = [tl_tex[0] + tex_width, tl_tex[1] + tex_width];
		let tr_tex = [tl_tex[0] + tex_width, tl_tex[1]];

		tex_coords.push(tl_tex);
		tex_coords.push(bl_tex);
		tex_coords.push(br_tex);
		tex_coords.push(tr_tex);

		tex_coords
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
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
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                },
            ],
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

				let (vertices, indices) = game.render_all();
				
				let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(vertices.as_slice()),
            wgpu::BufferUsage::VERTEX,
        );
        let index_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(indices.as_slice()),
            wgpu::BufferUsage::INDEX,
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
        false
    }

    fn update(&mut self) {

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

    // Since main can't be async, we're going to need to block
    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
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
                    }
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
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}

//pretty basic implementation of vectors because what even are crates
struct Vector {
		x: f32,
		y: f32,
}
#[allow(dead_code)]
impl Vector {
		fn new(x: f32, y: f32) -> Vector {
				Vector { x, y }
		}
		
		fn add(&self, other: &Vector) -> Vector {
				Vector::new(self.x + other.x, self.y + other.y)
		}

		fn subtract(&self, other: &Vector) -> Vector {
				Vector::new(self.x - other.x, self.y - other.y)
		}

		fn scale_by(&self, number: f32) -> Vector {
				Vector::new(self.x * number, self.y * number)
		}

		fn length(&self) -> f32 {
				self.x.hypot(self.y)
		}

		fn normalize(&self) -> Vector {
				self.scale_by(1_f32 / self.length())
		}
		
		fn equal_to(&self, other: &Vector) -> bool {
				self.x == other.x && self.y == other.y
		}
		
		fn is_opposite(&self, other: &Vector) -> bool {
				let sum = self.add(other);
				sum.equal_to(&Vector::new(0_f32, 0_f32))
		}

		fn dot_product(&self, other: &Vector) -> f32 {
				self.x * other.x + self.y * other.y
		}
		fn rotate(&self, other: &Vector) -> Vector {
				Vector::new(
						(self.x * other.x / (other.x.powi(2) + other.y.powi(2)).sqrt()) -
								(self.y * other.y / (other.x.powi(2) + other.y.powi(2)).sqrt())
								,
						(self.x * other.y / (other.x.powi(2) + other.y.powi(2)).sqrt()) +
								(self.y * other.x / (other.x.powi(2) + other.y.powi(2)).sqrt())
				)
		}
}
