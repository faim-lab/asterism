mod texture;
mod vector;
use vector::Vector;
mod make_square;
use make_square::render_thing;
use ultraviolet::{Vec2, geometry::Aabb};
use asterism::{Resources, resources::Transaction, AabbCollision, PointPhysics, control::*};

mod level1;

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};

// NOTE: much of the code below is from Carl; he gave me permission to use it.
// I've been spending a lot of time trying to completely understand how it works,
// what I need to do to make the game I have in mind and how I could possibly improve on it.

struct World {
    archer: (u8, u8), //(x, y)
    arrow: (u8, u8, u8, bool), //(x, y, arrow number, if it's "live")
    goblin: (u8, u8, u8), //(x, y, goblin number)
    //I don't really know how I'm going to get the game to recognize
    //these or their collisionIDs as their respective parts
}

enum ActionID {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Fire(Arrow),
}

impl Default for ActionID {
    fn default() -> Self { Self::MoveX(Archer)}
}

enum CollisionID {
    Archer,
    Arrow(ArrowNum),
    Goblin(GoblinNum),
    TopWall,
    BottomWall,
    LeftWall,
    RightWall,
}

enum ArrowNum {
    A1,
    A2,
    A3,
    A4,
    A5,
}

//do I need a default arrow?

enum GoblinNum {
    G1,
    //who knows how many goblins there will be? I certainly don't
}

impl Default for CollisionID {
    fn default() -> Self { Self::Archer }
}

struct Logics {
    control: WinitKeyboardControl<ActionID>,
    physics: PointPhysics,
    collision: AabbCollision<CollisionID>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = WinitKeyboardControl::new();
                control.add_key_map(0,
                    VirtualKeyCode::W,
                    ActionID::MoveUp,
                );
                control.add_key_map(0,
                    VirtualKeyCode::S,
                    ActionID::MoveDown,
                );
                control.add_key_map(0,
                    VirtualKeyCode::A,
                    ActionID::MoveLeft,
                );
                control.add_key_map(0,
                    VirtualKeyCode::D,
                    ActionID::MoveRight,
                );
                control.add_key_map(0,
                    VirtualKeyCode::Space,
                    // this probably needs some work, but I have the basic concept here
                    for i in ArrowNum {
                        if self.arrow.3 == false{
                            ActionID::Fire(i)
                        }
                    }
                );
                control
            },
            physics: PointPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                collision.add_collision_entity(-1.0, 0.0,
                    1.o, height as f32,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::LeftWall);
                collision.add_collision_entity(WIDTH as f32, 0.0,
                    1.0, height as f32,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::RightWall);
                collision.add_collision_entity(0.0, -1.0,
                    WIDTH as f32, 1.0,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::TopWall);
                collision.add_collision_entity(0.0, HEIGHT as f32,
                    WIDTH as f32, 1.0,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::BottomWall);
                collision 
            },
        }
    }
}

pub struct Game {
    view_area: ViewArea,
    
    renderable_components: RenderableComponentVec,
    position_components: PositionComponentVec,
    projectile_components: ProjectileComponentVec,

    user_inputs: UserInputs,
}
impl Game {
    fn fire_arrow (&mut self) {
        if self.user_inputs.va_space {
            self.add_things(vec![((100_f32, 100_f32), (Vector::new(1_f32, 0_f32), 0.25, 2_u32))])
        }
    }
    pub fn new() -> Game {
            let position_component_vec = PositionComponentVec::new();
            let renderable_component_vec = RenderableComponentVec::new();
            let projectile_component_vec = ProjectileComponentVec::new();
            let user_inputs = UserInputs::new();
            let mut game: Game = Game {
                    view_area: ViewArea::new(),
                    position_components: position_component_vec,
                    renderable_components: renderable_component_vec,
                    projectile_components: projectile_component_vec,
                    user_inputs: user_inputs,
            };
            game.add_things(level1::make_level_1());
            game
    }
    pub fn add_things(&mut self, new_things_list: Vec<((f32, f32), (Vector, f32, u32))>) {
            for thing in new_things_list.into_iter() {
                    let pos_x: f32 = (thing.0).0;
                    let pos_y: f32 = (thing.0).1;

                    let facing: Vector = (thing.1).0;
                    let size: f32 = (thing.1).1;
                    let texture: u32 = (thing.1).2;
                    
                    self.position_components.add(
                            PositionComponent::new(
                                    pos_x, pos_y
                            )
                    );
                    self.projectile_components.add(
                            ProjectileComponent::new(
                                pos_x, pos_y
                        )
                    );

                    self.renderable_components.add(
                            RenderableComponent::new(
                                    None, facing, size, texture
                            )
                    );
            }
            self.renderable_components
                    .update_all_coords(&self.view_area, &self.position_components, &self.projectile_components);
    }
    fn render(&self) -> (Vec<Vertex>, Vec<u16>) {				
            self.renderable_components.render_all()
    }
    fn update(&mut self) {
            self.position_components.update_using_input(&self.user_inputs);
            self.projectile_components.update_using_input (&self.user_inputs);
            self.view_area.update_using_input(&self.user_inputs);
            self.fire_arrow();
            self.renderable_components
                    .update_all_coords(&self.view_area, &self.position_components, &self.projectile_components);
    }
}

impl World {
    fn new() -> Self {
        // what do I put here?
    }
    fn update (&mut self, logivs: &mut Logics, input: &WinitInputHelper) {
        self.project_control(&mut logics.control);
        logics.control.update(input);
        self.unproject_contrp;(&logics.control);

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        //INCOMPLETE
        for contact in logics.collision.contacts.iter() {
            match (logics.collision.metadata[contact.0].id,
                logics.collision.metadata[contact.1].id) {
                    (CollisionID::LeftWall, CollisionID::Archer) => {
                        //
                    }
                    (CollisionID::RightWall, CollisionID::Archer) => {
                        //
                    }
                    (CollisionID::TopWall, CollisionID::Archer) => {
                        //
                    }
                    (CollisionID::BottomWall, CollisionID::Archer) => {
                        //
                    }
                }
        }
    }
    fn project_control(&self, control: &mut WinitKeyboardControl<ActionID>) {
        //control.mapping[].is_valid[] = true?
    }

    fn unproject_control(&mut self, control: &WinitKeyboardControl<ActionID>) {
        //is this where the controls go?
        self.archer.0 = (self.archer.0 as i16 + control.get_action(ActionID::MoveLeft).unwrap() as i16);
        self.archer.0 = (self.archer.0 as i16 + control.get_action(ActionID::MoveRight).unwrap() as i16);
        self.archer.1 = (self.archer.1 as i16 + control.get_action(ActionID::MoveUp).unwrap() as i16);
        self.archer.1 = (self.archer.1 as i16 + control.get_action(ActionID::MoveDown).unwrap() as i16);
    }

    //the million dollar question is: does archer game really need physics?

    fn project_physics(&self, physics: &mut PointPhysics) {
        physics.positions.resize_with(1, Vec2::default);
        physics.velocities.resize_with(1, Vec2::default);
        physics.accelerations.resize_with(1, Vec2::default);
        physics.positions[0].x = self.archer.0 as f32 // + self.ball_err.x;
        physics.positions[0].y = self.archer.1 as f32 // + self.ball_err.y;
        // physics.velocities[0] = self.ball_vel;
        physics.accelerations[0] = Vec2::new(0.0, 0.0);
    }

    fn unproject_physics(&mut self, physics: &PointPhysics) {
        self.archer.0 = physics.positions[0].x.trunc().max(0.0).min((WIDTH - BALL_SIZE) as f32) as u8;
        self.archer.1 = physics.positions[0].y.trunc().max(0.0).min((HEIGHT - BALL_SIZE) as f32) as u8;
        // self.ball_err = physics.positions[0] - Vec2::new(self.ball.0 as f32, self.ball.1 as f32);
        // self.ball_vel = physics.velocities[0];
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID>, control: &WinitKeyboardControl<ActionID>) {
        collision.bodies.resize_with(4, Aabb::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);
        collision.add_collision_entity(
            //add archer collision entity once I figure out how to move the collision entity itself
        )
        collision.add_collision_entity(
            //add collision entities for each of the arrows
        )
        collision.add_collision_entity(
            //add goblin collision entities when goblins become a part of the game    
        );
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID>) {
        self.archer.0 = collision.bodies[4].min.x.trunc() as u8;
        self.archer.1 = collision.bodies[4].min.y.trunc() as u8;
        //what does this do?
    }
}



//---------- view area ----------

struct ViewArea {
    position: Vector,
}

impl ViewArea {
    const LENGTH: f32 = 100_f32;
    const WIDTH: f32 = 100_f32;
    const SPEED: f32 = 0.5;
    fn new() -> ViewArea {
            ViewArea::default()
    }
    fn pos_cords_to_render_cords(&self, thing_position: &PositionComponent) -> (f32, f32) {
            let potential_render_x: f32 = (thing_position.x - self.position.x) / (ViewArea::LENGTH / 2_f32);
            let potential_render_y: f32 = -(thing_position.y - self.position.y) / (ViewArea::WIDTH / 2_f32);
            (potential_render_x, potential_render_y)
    }
    fn arrow_pos_cords_to_render_cords(&self, thing_position: &ProjectileComponent) -> (f32, f32) {
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
            self.position.x = self.position.x.min(250_f32 - (ViewArea::LENGTH / 2_f32));

            self.position.y = self.position.y.max(-50_f32 + (ViewArea::WIDTH / 2_f32));
            self.position.y = self.position.y.min(250_f32 - (ViewArea::WIDTH / 2_f32));
    }
}

impl Default for ViewArea {
    fn default() -> ViewArea {
            ViewArea {
                    position: Vector {
                            x: 100_f32,
                            y: 100_f32,
                    }
            }
    }
}



//---------- position ---------
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
}

#[derive(Debug)]
struct PositionComponentVec {
    parts: Vec<PositionComponent>,
}

impl PositionComponentVec {
    const PLAYERSPEED: f32 = 0.5;
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
                movement_vec = movement_vec.normalize().scale_by(PositionComponentVec::PLAYERSPEED);
        }

        self.parts[256].x += movement_vec.x;
        self.parts[256].y += movement_vec.y;
    }

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



//---------- projectiles ---------
#[derive(Debug)]
struct ProjectileComponent {
    x: f32,
    y: f32,
}



impl ProjectileComponent {
    fn new(x: f32, y: f32) -> ProjectileComponent {
            ProjectileComponent {
                    x,
                    y,
            }
    }
}

#[derive(Debug)]
struct ProjectileComponentVec {
    parts: Vec<ProjectileComponent>,
}

impl ProjectileComponentVec {
    const ARROWSPEED: f32 = 0.2;
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
                movement_vec = movement_vec.normalize().scale_by(ProjectileComponentVec::ARROWSPEED);
        }

        self.parts[257].x += movement_vec.x;
        self.parts[257].y += movement_vec.y;
    }

    fn new() -> ProjectileComponentVec {
            ProjectileComponentVec {
                    parts: Vec::new(),
            }
    }
    fn add(&mut self, projectile_component: ProjectileComponent) {
            self.parts.push(projectile_component);
    }
    fn get_render_positions(&self, view_area: &ViewArea) -> Vec<Vector> {
            let mut render_positions: Vec<Vector> = Vec::new();
            let (render_x, render_y): (f32, f32) = view_area
                .arrow_pos_cords_to_render_cords(&self.parts[257]);
            render_positions.push(Vector::new(render_x, render_y));
            //println!("{:?}", render_positions);
            render_positions
    }
}



// ---------- input ----------

struct UserInputs {
    pub va_up: bool,
    pub va_down: bool,
    pub va_left: bool,
    pub va_right: bool,
    pub va_space: bool,
}

impl UserInputs {
    fn new() -> UserInputs {
            UserInputs {
                    va_up: false,
                    va_down: false,
                    va_left: false,
                    va_right: false,
                    va_space: false,
            }
    }
}

// ---------- rendering ----------
#[derive(Debug)]
struct RenderableComponent {
    position: Option<Vector>,
    facing: Vector,
    size: f32,
    texture: u32,
}

impl RenderableComponent {
    fn new(position: Option<Vector>,
                 facing: Vector,
                 size: f32,
                 texture: u32,) -> RenderableComponent {
            RenderableComponent {
                    position,
                    facing,
                    size,
                    texture,
            }
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
                    Some(i) => Some(render_thing(&i, &self.facing, self.size, self.texture)),
                    None => None,
            }
    }
}

#[derive(Debug)]
struct RenderableComponentVec {
    // should be option<RenderablePart> but there are bigger problems for now
    parts: Vec<RenderableComponent>,
}

impl RenderableComponentVec {
    fn new() -> RenderableComponentVec {
            RenderableComponentVec {
                    parts: Vec::new(),
            }
    }
    fn add(&mut self, new_component: RenderableComponent) {
            self.parts.push(new_component);
    }
    fn update_all_coords(&mut self, view_area: &ViewArea, position_component_vec: &PositionComponentVec, projectile_component_vec: &ProjectileComponentVec) {
            let new_coords = position_component_vec.get_render_positions(view_area);
            for i in 0..self.parts.len() {
                    self.parts[i].update_coordinates(new_coords[i])
            }
            let new_coords = projectile_component_vec.get_render_positions(view_area);
            self.parts[257].update_coordinates(new_coords[257])
    }
    fn render_all(&self) -> (Vec<Vertex>, Vec<u16>) {
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
}














//---------- wgpu stuff ----------

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[allow(dead_code)]
struct State {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    size: winit::dpi::PhysicalSize<u32>,

    num_indices: u32,

                game: Game,

                diffuse_texture: texture::Texture,
                diffuse_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

//const VERTICES: &[Vertex] = &[
//    Vertex { position: [-0.1, 0.1, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // 0
//    Vertex { position: [-0.1, -0.1, 0.0], tex_coords: [0.28081453, 0.949397057], }, // 1
//    Vertex { position: [0.1, -0.1, 0.0], tex_coords: [0.85967, 0.84732911], }, // 2
//    Vertex { position: [0.1, 0.1, 0.0], tex_coords: [0.9414737, 0.2652641], }, // 
//];

//const INDICES: &[u16] = &[
//    0, 1, 2,
//    0, 2, 3,
//];

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

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PresentMode {
    Immediate = 0,
    Mailbox = 1,
    Fifo = 2,
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
        ).await.unwrap(); // Get used to seeing this

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
        let (diffuse_texture, cmd_buffer) = texture::Texture::from_bytes(&device, diffuse_bytes, "texture.png").unwrap();

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
            label: Some("texture_bind_group_layout"),
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
                entry_point: "main", // 1.
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor { // 2.
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

            primitive_topology: wgpu::PrimitiveTopology::TriangleList, // 1.

            sample_count: 1, // 5.
            sample_mask: !0, // 6.
            alpha_to_coverage_enabled: false, // 7.
        });

        let game = Game::new();

        let (vertices, indices) = game.render();

        let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&vertices),
            wgpu::BufferUsage::VERTEX,
        );

        let index_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&indices),
            wgpu::BufferUsage::INDEX,
        );

        let num_indices = indices.len() as u32;

        Self {
            surface,
            adapter,
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
                            VirtualKeyCode::Space => self.game.user_inputs.va_space = *state == ElementState::Pressed,
                            _ => (),
                    }
                    true
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
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); // NEW!
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
        .with_title("Goblins!")
        .build(&event_loop)
        .unwrap();
    use futures::executor::block_on;

    let mut state = block_on(State::new(&window));

    let mut world = World::new();
    let mut logics = Logics::new();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                state.update();
                state.render();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
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
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });
}
