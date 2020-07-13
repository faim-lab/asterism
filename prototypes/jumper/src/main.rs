#![deny(clippy::all)]
#![forbid(unsafe_code)]

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use ultraviolet::{Vec2, geometry::Aabb};
use asterism::{AabbCollision, control::*, FlatEntityState, PointPhysics};

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Player {
    P1
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActionID {
    Move(Player),
    Jump(Player),
}

impl Default for ActionID {
    fn default() -> Self { Self::Move(Player::P1) }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CollisionID {
    Player(Player),
    TopWall,
    BottomWall,
    LeftWall,
    RightWall,
    Ground,
    MovingPlatform,
}

impl Default for CollisionID {
    fn default() -> Self { Self::LeftWall }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum StateID {
    PlatformLeft,
    PlatformRight,
    PlayerGrounded,
    PlayerWalk,
    PlayerJump,
    PlayerFall,
}

struct Logics {
    control: WinitKeyboardControl<ActionID>,
    physics: PointPhysics,
    collision: AabbCollision<CollisionID>,
    entity_state: FlatEntityState<StateID>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = WinitKeyboardControl::new();
                control.mapping.push(
                    InputMap {
                        inputs: vec![(KeyInput::Pair(VirtualKeyCode::Left, VirtualKeyCode::Right),
                                    InputState::On),
                                    (KeyInput::Single(VirtualKeyCode::Space), InputState::Pressed),
                        ],
                        actions: vec![Action {
                            id: ActionID::Move(Player::P1),
                            action_type: ActionType::Axis(-1.0, 1.0)
                        }, Action {
                            id: ActionID::Jump(Player::P1),
                            action_type: ActionType::Continuous(-1.0)
                        }],
                        is_valid: vec![false; 2],
                    }
                );
                control
            },
            physics: PointPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                collision.add_collision_entity(-1.0, 0.0,
                    1.0, HEIGHT as f32,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::RightWall);
                collision.add_collision_entity(WIDTH as f32, 0.0,
                    1.0, HEIGHT as f32,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::LeftWall);
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
            entity_state: {
                let mut entity_state = FlatEntityState::new();
                entity_state.add_state_map(0,
                    vec![(StateID::PlatformLeft, vec![1]),
                    (StateID::PlatformRight, vec![0])
                    ]);
                entity_state.add_state_map(3,
                    vec![(StateID::PlayerGrounded, vec![1, 2]),
                    (StateID::PlayerWalk, vec![0, 2]),
                    (StateID::PlayerJump, vec![3]),
                    (StateID::PlayerFall, vec![0, 1])
                    ]);
                entity_state
            }
        }
    }
}

struct World {
    player: Entity,
    ground: Entity,
    platform: Entity,
}

struct Entity {
    x: u8, y: u8, w: u8, h: u8,
    vel: Vec2, err: Vec2,
}

impl Entity {
    fn new(x: u8, y: u8, w: u8, h: u8) -> Self {
        Self {
            x: x, y: y, w: w, h: h,
            vel: Vec2::new(0.0, 0.0),
            err: Vec2::new(0.0, 0.0)
        }
    }
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("jumper")
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
        if input.update(&event) {
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
            player: Entity::new(50, 70, 10, 10),
            ground: Entity::new(0, 200, WIDTH, 55),
            platform: Entity::new(25, 50, 55, 9)
        }
    }

    fn update(&mut self, logics: &mut Logics, input: &WinitInputHelper) {
        self.project_control(&mut logics.control, &logics.entity_state);
        logics.control.update(input);
        self.unproject_control(&logics.control, &logics.entity_state);

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        self.project_entity_state(&mut logics.entity_state, &logics.collision);
        logics.entity_state.update();
        self.unproject_entity_state(&logics.entity_state);
    }

    fn project_control(&self, control: &mut WinitKeyboardControl<ActionID>, entity_state: &FlatEntityState<StateID>) {
        control.mapping[0].is_valid[0] = true;
        control.mapping[0].is_valid[1] = match entity_state.get_id_for_entity(1) {
            StateID::PlayerGrounded | StateID::PlayerWalk => true,
            _ => false,
        }
    }

    fn unproject_control(&mut self, control: &WinitKeyboardControl<ActionID>, entity_state: &FlatEntityState<StateID>) {
        self.player.vel.x = control.values[0][0];
        match entity_state.get_id_for_entity(1) {
            StateID::PlayerGrounded | StateID::PlayerWalk => self.player.vel.y = control.values[0][1],
            _ => {}
        } 
    }

    fn project_physics(&self, physics: &mut PointPhysics) {
        physics.accelerations.resize_with(2, Vec2::default);
        physics.positions.resize_with(2, Vec2::default);
        physics.velocities.resize_with(2, Vec2::default);

        let mut add_physics_entity = |i: usize, pos: Vec2, vel: Vec2, acc: Vec2| {
            physics.positions[i] = pos;
            physics.velocities[i] = vel;
            physics.accelerations[i] = acc;
        };

        add_physics_entity(0,
            Vec2::new(self.player.x as f32, self.player.y as f32),
            self.player.vel,
            Vec2::new(0.0, 0.03));
        add_physics_entity(1,
            Vec2::new(self.platform.x as f32, self.platform.y as f32),
            self.platform.vel,
            Vec2::new(0.0, 0.0));
    }

    fn unproject_physics(&mut self, physics: &PointPhysics) {
        let update_game_state = |i: usize, x: &mut u8, y: &mut u8, err: &mut Vec2, vel: &mut Vec2, w: u8, h: u8| {
            *x = physics.positions[i].x.trunc().max(0.0).min((WIDTH - w) as f32) as u8;
            *y = physics.positions[i].y.trunc().max(0.0).min((HEIGHT - h) as f32) as u8;
            *err = physics.positions[i] - Vec2::new(*x as f32, *y as f32);
            *vel = physics.velocities[i];
        };

        update_game_state(0, &mut self.player.x, &mut self.player.y, &mut self.player.err, &mut self.player.vel, self.player.w, self.player.h);
        update_game_state(1, &mut self.platform.x, &mut self.platform.y, &mut self.platform.err, &mut self.platform.vel, self.platform.w, self.platform.h);
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID>) {
        collision.bodies.resize_with(4, Aabb::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);

        collision.add_collision_entity(self.player.x as f32, self.player.y as f32,
            self.player.w as f32, self.player.h as f32,
            self.player.vel,
            true, false, CollisionID::Player(Player::P1));
        collision.add_collision_entity(self.ground.x as f32, self.ground.y as f32,
            self.ground.w as f32, self.ground.h as f32,
            self.ground.vel,
            true, true, CollisionID::Ground);
        collision.add_collision_entity(self.platform.x as f32, self.platform.y as f32,
            self.platform.w as f32, self.platform.h as f32,
            self.platform.vel,
            true, true, CollisionID::MovingPlatform);
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID>) {
        self.player.x = collision.bodies[4].min.x.trunc() as u8;
        self.player.y = collision.bodies[4].min.y.trunc() as u8;
    }


    fn project_entity_state(&self, entity_state: &mut FlatEntityState<StateID>, collision: &AabbCollision<CollisionID>) {
        // update condition table
        for state_conditions in entity_state.conditions.iter_mut() {
            state_conditions.clear();
        }
        entity_state.conditions[0].resize(2, false);
        entity_state.conditions[1].resize(4, false);
        if self.platform.x < 30 {
            entity_state.conditions[0][1] = true;
        }
        if self.platform.x > 150 {
            entity_state.conditions[0][0] = true;
        }
        for contact in collision.contacts.iter() {
            match (collision.metadata[contact.0].id,
                collision.metadata[contact.1].id) {
                (CollisionID::Player(..), CollisionID::Ground) => {
                    if self.player.vel.x == 0.0 {
                        entity_state.conditions[1][0] = true;
                    } else {
                        entity_state.conditions[1][1] = true;
                    }
                }
                _ => {
                    if self.player.vel.y < 0.0 {
                        entity_state.conditions[1][2] = true;
                    } else {
                        entity_state.conditions[1][3] = true;
                    }
                }
            }
        }
    }

    fn unproject_entity_state(&mut self, entity_state: &FlatEntityState<StateID>) {
        for (map, state) in entity_state.maps.iter().zip(entity_state.states.iter()) {
            match map.states[*state].id {
                StateID::PlatformLeft => {
                    self.platform.vel.x = -1.0;
                }
                StateID::PlatformRight => {
                    self.platform.vel.x = 1.0;
                }
                _ => {}
            }
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[128, 128, 255, 255]);
        }
        draw_rect(self.player.x, self.player.y, self.player.w, self.player.h, [0, 0, 0, 255], frame);
        draw_rect(self.ground.x, self.ground.y, self.ground.w, self.ground.h, [64, 64, 64, 255], frame);
        draw_rect(self.platform.x, self.platform.y, self.platform.w, self.platform.h, [64, 64, 64, 255], frame);
    }
}

fn draw_rect(x: u8, y: u8, w: u8, h: u8, color: [u8;4], frame: &mut [u8]) {
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
