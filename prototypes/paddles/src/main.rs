#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use ultraviolet::{Vec2, Vec3, geometry::Aabb};
use std::collections::BTreeMap;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const PADDLE_OFF_X: u8 = 16;
const PADDLE_HEIGHT: u8 = 48;
const PADDLE_WIDTH: u8 = 8;
const BALL_SIZE: u8 = 8;

trait Input {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

#[derive(Clone)]
enum KeyInput {
    Single(VirtualKeyCode),
    Pair(VirtualKeyCode, VirtualKeyCode)
}

impl Input for KeyInput {
    fn min(&self) -> f32 { 0.0 }
    fn max(&self) -> f32 { 1.0 }
}

#[derive(Clone)]
enum InputState {
    Off, PressUp, PressDown, On
}

enum ActionType {
    Instant,
    Continuous,
    Axis(f32, f32)
}

impl Default for ActionType {
    fn default() -> Self { Self::Instant }
}

#[derive(Default)]
struct Action<ID: Copy + Eq> {
    id: ID,
    action_type: ActionType
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionID {
    Move(Player),
    Serve(Player),
}

impl Default for ActionID {
    fn default() -> Self { Self::Move(Player::P1) }
}

struct InputMap<I: Input, ID: Copy + Eq> {
    inputs: Vec<(I, InputState)>,
    actions: Vec<Action<ID>>
        // Invariants: inputs.len() == actions.actions.len()
}

struct WinitKeyboardControl<ID: Copy + Eq> {
    mapping: Vec<InputMap<KeyInput, ID>>,
    values: Vec<Vec<f32>> // vector of values per mapping.
        // Invariants: mapping.len() == values.len(), mapping[i].inputs.len() == values[i].len() 
}

impl<ID: Copy + Eq> WinitKeyboardControl<ID> {
    fn new() -> Self {
        Self {
            mapping: Vec::new(),
            values: Vec::new()
        }
    }

    fn update(&mut self, events: &WinitInputHelper) {
        self.values.clear();
        self.values.resize_with(self.mapping.len(), Default::default);
        for (map, vals) in self.mapping.iter().zip(self.values.iter_mut()) {
            vals.resize_with(map.inputs.len(), Default::default);
            for (action_map, value) in map.inputs.iter().zip(vals.iter_mut()) {
                let (input, input_state) = action_map;
                let is_activated = |keycode: VirtualKeyCode| {
                    match input_state {
                        InputState::On => events.key_held(keycode),
                        InputState::Off => !events.key_held(keycode),
                        InputState::PressDown => events.key_pressed(keycode),
                        InputState::PressUp => events.key_released(keycode)
                    }
                };
                match input {
                    KeyInput::Single(keycode) => {
                        if is_activated(*keycode) {
                            *value = 1.0;
                        }
                    }
                    KeyInput::Pair(keycode_min, keycode_max) => {
                        *value = 0.0;
                        let min_performed = is_activated(*keycode_min);
                        let max_performed = is_activated(*keycode_max);
                        if min_performed {
                            *value += -1.0;
                        }
                        if max_performed {
                            *value += 1.0;
                        }
                    }
                }
            }
        }
    }

    // not sure how to use these
    pub fn get_action_by_index(&self, action_set: usize, idx: usize) -> f32 {
        self.values[action_set][idx]
    }

    // This gets the value of the first action whose `id` is `id`.
    pub fn get_action(&self, id: ID) -> Option<f32> {
        for (i, set) in self.mapping.iter().enumerate() {
            if let Some(j) = set.actions.iter().position(|act| act.id == id) {
                return Some(self.values[i][j]);
            }
        }
        None
    }

    pub fn get_action_in_set(&self, action_set: usize, id: ID) -> Option<f32> {
        if let Some(idx) = self.mapping[action_set].actions.iter().position(|act| act.id == id) {
            return Some(self.get_action_by_index(action_set, idx));
        }
        None
    }
}

struct PongPhysics {
    positions:Vec<Vec2>,
    velocities:Vec<Vec2>
}

impl PongPhysics {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
        }
    }

    fn update(&mut self) {
        for (pos, vel) in self.positions.iter_mut().zip(self.velocities.iter()) {
            *pos += *vel;
        }
    }
}

struct AabbCollision<ID: Copy + Eq> {
    bodies: Vec<Aabb>,
    velocities: Vec<Vec2>,
    metadata: Vec<CollisionData<ID>>,
    contacts: Vec<(usize, usize)>,
}

#[derive(Default, Clone, Copy)]
struct CollisionData<ID: Copy + Eq> {
    solid: bool, // true = participates in restitution, false = no
    fixed: bool, // collision system cannot move it
    id: ID
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CollisionID {
    Paddle(Player),
    Ball,
    TopWall,
    BottomWall,
    SideWall(Player),
}

impl Default for CollisionID {
    fn default() -> Self { Self::Ball }
}

impl<ID: Copy + Eq> AabbCollision<ID> {
    fn new() -> Self {
        Self {
            bodies: Vec::new(),
            velocities: Vec::new(),
            metadata: Vec::new(),
            contacts: Vec::new(),
        }
    }

    fn update(&mut self) {
        self.contacts.clear();
        for (i, body) in self.bodies.iter().enumerate() {
            for (j, body2) in self.bodies[i + 1..].iter().enumerate() {
                if body.intersects(body2) {
                    self.contacts.push((i, j + i + 1));
                }
            }
        }

        for (i, j) in self.contacts.iter() {
            let CollisionData { solid: i_solid, fixed: i_fixed, .. } =
                self.metadata[*i];
            let CollisionData { solid: j_solid, fixed: j_fixed, .. } =
                self.metadata[*j];

            if i_solid && j_solid {
                continue;
            }

            if !i_fixed && !j_fixed {
                let Vec2 { x: vel_i_x, y: vel_i_y } = self.velocities[*i];
                let Vec2 { x: vel_j_x, y: vel_j_y } = self.velocities[*j];
                let Aabb { min: Vec3 { x: min_i_x, y: min_i_y, .. },
                    max: Vec3 { x: max_i_x, y: max_i_y, ..} } = self.bodies[*i];
                let Aabb { min: Vec3 { x: min_j_x, y: min_j_y, .. },
                    max: Vec3 { x: max_j_x, y: max_j_y, ..} } = self.bodies[*j];

                let ( i_displace, j_displace ) = {
                    let vel_i_x = vel_i_x / (vel_i_x.abs() + vel_j_x.abs());
                    let vel_i_y = vel_i_y / (vel_i_y.abs() + vel_j_y.abs());
                    let vel_j_x = vel_j_x / (vel_i_x.abs() + vel_j_x.abs());
                    let vel_j_y = vel_j_y / (vel_i_y.abs() + vel_j_y.abs());

                    let displacement_x = Self::get_displacement(min_i_x, max_i_x, min_j_x, max_j_x);
                    let displacement_y = Self::get_displacement(min_i_y, max_i_y, min_j_y, max_j_y);

                    ( Vec3::new(displacement_x * vel_i_x, displacement_y * vel_i_y, 0.0),
                        Vec3::new(displacement_x * vel_j_x, displacement_y * vel_j_y, 0.0) )
                };

                self.bodies[*i].min += i_displace;
                self.bodies[*i].max += i_displace;
                self.bodies[*j].min += j_displace;
                self.bodies[*j].max += j_displace;
            } else {
                let i = if !j_fixed { j } else { i };
                let Aabb { min: Vec3 { x: min_i_x, y: min_i_y, .. },
                    max: Vec3 { x: max_i_x, y: max_i_y, ..} } = self.bodies[*i];
                let Aabb { min: Vec3 { x: min_j_x, y: min_j_y, .. },
                    max: Vec3 { x: max_j_x, y: max_j_y, ..} } = self.bodies[*j];
                let displace = {
                    let displacement_x = Self::get_displacement(min_i_x, max_i_x, min_j_x, max_j_x);
                    let displacement_y = Self::get_displacement(min_i_y, max_i_y, min_j_y, max_j_y);

                    if displacement_x < displacement_y {
                        Vec3::new(displacement_x, 0.0, 0.0)
                    } else {
                        Vec3::new(0.0, displacement_y, 0.0)
                    }
                };

                self.bodies[*i].min += displace;
                self.bodies[*i].max += displace;
            }
        }
    }

    fn get_displacement(min_i: f32, max_i: f32, min_j: f32, max_j: f32)
        -> f32 {
            if max_i - min_j < max_j - min_i {
                max_i - min_j
            } else {
                min_i - max_j
            }
    }
}

struct PongResources<ID: Copy + Ord> {
    items: BTreeMap<ID, f32>,
    transactions: Vec<Vec<(ID, Transaction)>>,
    completed: Vec<(bool, Option<Vec<ID>>)>
}

impl<ID: Copy + Ord> PongResources<ID> {
    fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            transactions: Vec::new(),
            completed: Vec::new(),
        }
    }

    fn update(&mut self) {
        self.completed.clear();
        'exchange: for exchange in self.transactions.iter() {
            let mut snapshot: BTreeMap<ID, f32> = BTreeMap::new();
            for (item_type, ..) in exchange {
                snapshot.insert( *item_type, *self.items.get(&item_type).unwrap() );
            }
            let mut item_types = vec![];
            for (item_type, change) in exchange.iter() {
                if !self.is_possible(item_type, change) {
                    self.completed.push((false, None));
                    for (item_type, val) in snapshot.iter() {
                        *self.items.get_mut(&item_type).unwrap() = *val;
                        continue 'exchange;
                    }
                }
                match change {
                    Transaction::Change(amt) => {
                        *self.items.get_mut(&item_type).unwrap() += *amt as f32;
                        item_types.push(*item_type);
                    }
                }
            }
            self.completed.push((true, Some(item_types)));
        }
        self.transactions.clear();
    }

    fn is_possible (&self, item_type: &ID, transaction: &Transaction) -> bool {
        if !self.items.contains_key(item_type) {
            false
        } else {
            let value = self.items.get(item_type);
            match transaction {
                Transaction::Change(amt) => {
                    if value.unwrap() + *amt as f32 > 0.0 {
                        true
                    } else { false }
                }
            }
        }
    }

    fn get_value_by_itemtype(&self, item_type: &ID) -> f32 {
        *self.items.get(item_type).unwrap()
    }

}

#[derive(Clone, Copy)]
enum Transaction {
    Change(i8),
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum PoolID {
    Points(Player)
}

struct Logics {
    control: WinitKeyboardControl<ActionID>,
    physics: PongPhysics,
    collision: AabbCollision<CollisionID>,
    resources: PongResources<PoolID>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = WinitKeyboardControl::new();
                control.mapping.push(
                    InputMap {
                        inputs: vec![(KeyInput::Pair(VirtualKeyCode::Q, VirtualKeyCode::A),
                                    InputState::On),
                        ],
                        actions: vec![]
                    }
                );
                control.mapping.push(
                    InputMap {
                        inputs: vec![(KeyInput::Pair(VirtualKeyCode::O, VirtualKeyCode::L),
                                    InputState::On)
                        ],
                        actions: vec![Action {
                            id: ActionID::Move(Player::P2),
                            action_type: ActionType::Axis(-1.0, 1.0)
                        }],
                    });
                control
            },
            physics: PongPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                collision.bodies.push(Aabb::new(
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(0.0, HEIGHT as f32, 0.0)));
                collision.bodies.push(Aabb::new(
                    Vec3::new(WIDTH as f32, 0.0, 0.0),
                    Vec3::new(WIDTH as f32 + 1.0, HEIGHT as f32, 0.0)));
                collision.bodies.push(Aabb::new(
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(WIDTH as f32, 0.0, 0.0)));
                collision.bodies.push(Aabb::new(
                    Vec3::new(0.0, HEIGHT as f32, 0.0),
                    Vec3::new(WIDTH as f32, HEIGHT as f32 + 1.0, 0.0)));
                for _ in 1..4 {
                    collision.velocities.push(Vec2::new(0.0, 0.0));
                }
                collision.metadata.push(CollisionData{ solid: true, fixed: true, id: CollisionID::SideWall(Player::P1) });
                collision.metadata.push(CollisionData{ solid: true, fixed: true, id: CollisionID::SideWall(Player::P2) });
                collision.metadata.push(CollisionData{ solid: true, fixed: true, id: CollisionID::TopWall });
                collision.metadata.push(CollisionData{ solid: true, fixed: true, id: CollisionID::BottomWall });
                collision
            },
            resources: {
                let mut resources = PongResources::new();
                resources.items.insert( PoolID::Points(Player::P1), 0.0 );
                resources.items.insert( PoolID::Points(Player::P2), 0.0 );
                resources
            }
        }
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
enum Player {
    P1,
    P2
}


struct World {
    paddles: (u8, u8),
    ball: (u8, u8),
    ball_err: Vec2,
    ball_vel: Vec2,
    serving: Option<Player>,
    score: (u8, u8)
}


fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("paddles")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut hidpi_factor = window.scale_factor();

    let mut pixels = {
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(WIDTH as u32, HEIGHT as u32, surface);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };
    let mut world = World::new();
    let mut logics = Logics::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                    .map_err(|e| panic!("pixels.render() failed: {}", e))
                    .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Adjust high DPI factor
            if let Some(factor) = input.scale_factor_changed() {
                hidpi_factor = factor;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update(&mut logics, &input);
            window.request_redraw();
        }
    });
}

impl World {
    fn new() -> Self {
        Self {
            paddles: (HEIGHT/2-PADDLE_HEIGHT/2, HEIGHT/2-PADDLE_HEIGHT/2),
            ball: (WIDTH/2-BALL_SIZE/2, HEIGHT/2-BALL_SIZE/2),
            ball_err: Vec2::new(0.0,0.0),
            ball_vel: Vec2::new(0.0,0.0),
            serving: Some(Player::P1),
            score: (0, 0),
        }
    }

    fn update(&mut self, logics: &mut Logics, input: &WinitInputHelper) {
        self.project_control(&mut logics.control);
        logics.control.update(input);
        self.unproject_control(&logics.control);

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        for contact in logics.collision.contacts.iter() {
            match (logics.collision.metadata[contact.0].id,
                logics.collision.metadata[contact.1].id) {
                (CollisionID::SideWall(player), CollisionID::Ball) => {
                    self.ball_vel = Vec2::new(0.0, 0.0);
                    self.ball = (WIDTH / 2 - BALL_SIZE / 2,
                        HEIGHT / 2 - BALL_SIZE / 2);
                    self.serving = Some(player);
                    match player {
                        Player::P1 => logics.resources.transactions.push(vec![(PoolID::Points(Player::P2), Transaction::Change(1))]),
                        Player::P2 => logics.resources.transactions.push(vec![(PoolID::Points(Player::P1), Transaction::Change(1))]),
                    }
                }
                (CollisionID::TopWall, CollisionID::Ball) |
                    (CollisionID::BottomWall, CollisionID::Ball) => {
                        self.ball_vel.y *= -1.0;
                    }
                (CollisionID::Ball, CollisionID::Paddle(player)) => {
                    if match player {
                        Player::P1 =>
                            (self.ball.0 as i16 - (PADDLE_OFF_X + PADDLE_WIDTH) as i16).abs()
                            > ((self.ball.1 + BALL_SIZE) as i16 - self.paddles.0 as i16).abs().min((self.ball.1 as i16 - (self.paddles.0 + PADDLE_HEIGHT) as i16).abs()),
                        Player::P2 =>
                            ((self.ball.0 + BALL_SIZE) as i16 - (WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as i16).abs()
                            > ((self.ball.1 + BALL_SIZE) as i16 - self.paddles.1 as i16).abs().min((self.ball.1 as i16 - (self.paddles.1 + PADDLE_HEIGHT) as i16).abs()),
                    } {
                        self.ball_vel.y *= -1.0;
                    } else {
                        self.ball_vel.x *= -1.0;
                    }
                    self.change_angle(player);
                },
                _ => {}
            }
        }
        
        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources);

        for (completed, item_types) in logics.resources.completed.iter() {
            if let Some(types) = item_types {
                if *completed {
                    for item_type in types {
                        match item_type {
                            PoolID::Points(player) => {
                                match player {
                                    Player::P1 => print!("p1"),
                                    Player::P2 => print!("p2")
                                }
                                println!(" scores! p1: {}, p2: {}", self.score.0, self.score.1);
                            }
                        }
                    }
                }
            }
        }
    }

    fn project_control(&self, control: &mut WinitKeyboardControl<ActionID>) {
        for map in control.mapping.iter_mut() {
            map.inputs.resize(2, (KeyInput::Single(VirtualKeyCode::H), InputState::On));
            map.actions.resize_with(2, Action::default);
        }
        if (self.ball_vel.x, self.ball_vel.y) == (0.0, 0.0) {
            match self.serving {
                Some(Player::P1) => {
                    control.mapping[0].inputs.push(
                        (KeyInput::Single(VirtualKeyCode::W),
                         InputState::PressDown));
                    control.mapping[0].actions.push(
                        Action { id: ActionID::Serve(Player::P1),
                            action_type: ActionType::Instant });
                }
                Some(Player::P2) => {
                    control.mapping[1].inputs.push(
                        (KeyInput::Single(VirtualKeyCode::I),
                         InputState::PressDown));
                    control.mapping[1].actions.push(
                        Action { id: ActionID::Serve(Player::P2),
                            action_type: ActionType::Instant });
                }
                None => {}
            }
        }
    }

    fn unproject_control(&mut self, control: &WinitKeyboardControl<ActionID>) {
        self.paddles.0 = ((self.paddles.0 as i16 + control.get_action(ActionID::Move(Player::P1)).unwrap() as i16).max(0) as u8).min(255 - PADDLE_HEIGHT);
        self.paddles.1 = ((self.paddles.1 as i16 + control.get_action(ActionID::Move(Player::P2)).unwrap() as i16).max(0) as u8).min(255 - PADDLE_HEIGHT);
        if (self.ball_vel.x, self.ball_vel.y) == (0.0, 0.0) {
            match self.serving {
                Some(Player::P1) => {
                    if control.get_action(ActionID::Serve(Player::P1)).unwrap() == 1.0 {
                        self.ball_vel = Vec2::new(1.0, 1.0);
                    }
                }
                Some(Player::P2) => {
                    if control.get_action(ActionID::Serve(Player::P2)).unwrap() == 1.0 {
                        self.ball_vel = Vec2::new(-1.0, -1.0);
                    }
                }
                None => {}
            }
        }
    }

    fn project_physics(&self, physics: &mut PongPhysics) {
        physics.positions.resize_with(1, Vec2::default);
        physics.velocities.resize_with(1, Vec2::default);
        physics.positions[0].x = self.ball.0 as f32 + self.ball_err.x;
        physics.positions[0].y = self.ball.1 as f32 + self.ball_err.y;
        physics.velocities[0] = self.ball_vel;
    }

    fn unproject_physics(&mut self, physics: &PongPhysics) {
        self.ball.0 = physics.positions[0].x.trunc().max(0.0).min((WIDTH - BALL_SIZE) as f32) as u8;
        self.ball.1 = physics.positions[0].y.trunc().max(0.0).min((HEIGHT - BALL_SIZE) as f32) as u8;
        self.ball_err = physics.positions[0] - Vec2::new(self.ball.0 as f32, self.ball.1 as f32);
        self.ball_vel = physics.velocities[0];
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID>, control: &WinitKeyboardControl<ActionID>) {
        collision.bodies.resize_with(4, Aabb::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, CollisionData::default);
        collision.bodies.push(Aabb::new(
                Vec3::new(self.ball.0 as f32, self.ball.1 as f32, 0.0),
                Vec3::new(self.ball.0 as f32 + BALL_SIZE as f32, self.ball.1 as f32 + BALL_SIZE as f32, 0.0)));
        collision.bodies.push(Aabb::new(
                Vec3::new((PADDLE_OFF_X) as f32, self.paddles.0 as f32, 0.0),
                Vec3::new((PADDLE_OFF_X + PADDLE_WIDTH) as f32, self.paddles.0 as f32 + PADDLE_HEIGHT as f32, 0.0)));
        collision.bodies.push(Aabb::new(
                Vec3::new((WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32, self.paddles.1 as f32, 0.0),
                Vec3::new((WIDTH - PADDLE_OFF_X) as f32, self.paddles.1 as f32 + PADDLE_HEIGHT as f32, 0.0)));

        collision.velocities.push(self.ball_vel);
        let p1_vel: f32 = control.values[0][0] + control.values[0][1];
        let p2_vel: f32 = control.values[1][0] + control.values[1][1];
        collision.velocities.push(Vec2::new(0.0, p1_vel));
        collision.velocities.push(Vec2::new(0.0, p2_vel));

        collision.metadata.push(CollisionData { solid: true, fixed: false, id: CollisionID::Ball });
        collision.metadata.push(CollisionData { solid: true, fixed: true, id: CollisionID::Paddle(Player::P1) });
        collision.metadata.push(CollisionData { solid: true, fixed: true, id: CollisionID::Paddle(Player::P2) });
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID>) {
        self.ball.0 = collision.bodies[4].min.x.trunc() as u8;
        self.ball.1 = collision.bodies[4].min.y.trunc() as u8;
    }

    fn change_angle(&mut self, player: Player) {
        let Vec2{x, y} = &mut self.ball_vel;
        let paddle_center = match player {
            Player::P1 => self.paddles.0 + PADDLE_HEIGHT / 2,
            Player::P2 => self.paddles.1 + PADDLE_HEIGHT / 2
        } as f32;
        let angle: f32 = (((self.ball.1 + BALL_SIZE / 2) as f32 - paddle_center)
            .max(- (PADDLE_HEIGHT as f32) / 2.0)
            .min(PADDLE_HEIGHT as f32 / 2.0) / PADDLE_HEIGHT as f32).abs() * 80.0;
        let magnitude = f32::sqrt(*x * *x + *y * *y);
        *x = angle.to_radians().cos() * magnitude
            * if *x < 0.0 { -1.0 } else { 1.0 };
        *y = angle.to_radians().sin() * magnitude
            * if *y < 0.0 { -1.0 } else { 1.0 };
        if magnitude < 5.0 {
            self.ball_vel *= Vec2::new(1.1, 1.1);
        }
    }

    fn project_resources(&self, resources: &mut PongResources<PoolID>) {
        if !resources.items.contains_key(&PoolID::Points(Player::P1)) {
            resources.items.insert(PoolID::Points(Player::P1), 0.0);
        }
        if !resources.items.contains_key(&PoolID::Points(Player::P2)) {
            resources.items.insert(PoolID::Points(Player::P1), 0.0);
        }
    }

    fn unproject_resources(&mut self, resources: &PongResources<PoolID>) {
        for (completed, item_types) in resources.completed.iter() {
            if let Some(types) = item_types {
                if *completed {
                    for item_type in types {
                        let value = resources.get_value_by_itemtype(item_type).min(255.0) as u8;
                        match item_type {
                            PoolID::Points(player) =>  {
                                match player {
                                    Player::P1 => self.score.0 = value,
                                    Player::P2 => self.score.1 = value,
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0,0,128,255]);
        }
        draw_rect(PADDLE_OFF_X, self.paddles.0,
            PADDLE_WIDTH, PADDLE_HEIGHT,
            [255,255,255,255],
            frame);
        draw_rect(WIDTH-PADDLE_OFF_X-PADDLE_WIDTH, self.paddles.1,
            PADDLE_WIDTH, PADDLE_HEIGHT,
            [255,255,255,255],
            frame);
        draw_rect(self.ball.0, self.ball.1,
            BALL_SIZE, BALL_SIZE,
            [255,200,0,255],
            frame);
    }
}

fn draw_rect(x:u8, y:u8, w:u8, h:u8, color:[u8;4], frame:&mut [u8]) {
    let x = x.min(WIDTH-1) as usize;
    let w = (w as usize).min(WIDTH as usize-x);
    let y = y.min(HEIGHT-1) as usize;
    let h = (h as usize).min(HEIGHT as usize-y);
    for row in 0..h {
        let row_start = (WIDTH as usize)*4*(y+row);
        let slice = &mut frame[(row_start+x*4)..(row_start+(x+w)*4)];
        for pixel in slice.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }
}
