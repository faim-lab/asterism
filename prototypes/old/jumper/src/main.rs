#![deny(clippy::all)]
#![forbid(unsafe_code)]

use asterism::{
    collision::AabbCollision, control::KeyboardControl, control::WinitKeyboardControl,
    entity_state::FlatEntityState, physics::PointPhysics, Logic,
};
use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use ultraviolet::Vec2;
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum ActionID {
    MoveLeft,
    MoveRight,
    Jump,
}

impl Default for ActionID {
    fn default() -> Self {
        Self::MoveLeft
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CollisionID {
    Player,
    TopWall,
    BottomWall,
    LeftWall,
    RightWall,
    Ground,
    MovingPlatform,
    Enemy,
}

impl Default for CollisionID {
    fn default() -> Self {
        Self::LeftWall
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum StateID {
    PlatformLeft,
    PlatformRight,
    PlayerGrounded,
    PlayerWalk,
    PlayerJump,
    PlayerFall,
    EnemyGrounded,
    EnemyNotGrounded,
    EnemyLeft,
    EnemyRight,
}

struct Logics {
    control: WinitKeyboardControl<ActionID>,
    physics: PointPhysics<Vec2>,
    collision: AabbCollision<CollisionID, Vec2>,
    entity_state: FlatEntityState<StateID>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = WinitKeyboardControl::new();
                control.add_key_map(0, VirtualKeyCode::Left, ActionID::MoveLeft);
                control.add_key_map(0, VirtualKeyCode::Right, ActionID::MoveRight);
                control.add_key_map(0, VirtualKeyCode::Space, ActionID::Jump);
                control
            },
            physics: PointPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                collision.add_entity_as_xywh(
                    Vec2::new(-1.0, 0.0),
                    Vec2::new(1.0, HEIGHT as f32),
                    Vec2::zero(),
                    true,
                    true,
                    CollisionID::LeftWall,
                );
                collision.add_entity_as_xywh(
                    Vec2::new(WIDTH as f32, 0.0),
                    Vec2::new(1.0, HEIGHT as f32),
                    Vec2::zero(),
                    true,
                    true,
                    CollisionID::RightWall,
                );
                collision.add_entity_as_xywh(
                    Vec2::new(0.0, -1.0),
                    Vec2::new(WIDTH as f32, 1.0),
                    Vec2::zero(),
                    true,
                    true,
                    CollisionID::TopWall,
                );
                collision.add_entity_as_xywh(
                    Vec2::new(0.0, HEIGHT as f32),
                    Vec2::new(WIDTH as f32, 1.0),
                    Vec2::zero(),
                    true,
                    true,
                    CollisionID::BottomWall,
                );
                collision
            },
            entity_state: {
                let mut entity_state = FlatEntityState::new();
                entity_state.add_state_map(
                    0,
                    vec![
                        (StateID::PlatformLeft, vec![1]),
                        (StateID::PlatformRight, vec![0]),
                    ],
                );
                entity_state.add_state_map(
                    3,
                    vec![
                        (StateID::PlayerGrounded, vec![1, 2, 3]),
                        (StateID::PlayerWalk, vec![0, 2, 3]),
                        (StateID::PlayerJump, vec![3]),
                        (StateID::PlayerFall, vec![0, 1]),
                    ],
                );
                entity_state.add_state_map(
                    1,
                    vec![
                        (StateID::EnemyGrounded, vec![1]),
                        (StateID::EnemyNotGrounded, vec![0]),
                    ],
                );
                entity_state.add_state_map(
                    1,
                    vec![
                        (StateID::EnemyLeft, vec![1]),
                        (StateID::EnemyRight, vec![0]),
                    ],
                );
                entity_state
            },
        }
    }
}

struct World {
    player: Entity,
    ground: Entity,
    platform: Entity,
    enemy: Entity,
}

struct Entity {
    x: u8,
    y: u8,
    w: u8,
    h: u8,
    vel: Vec2,
    err: Vec2,
    acc: Vec2,
}

impl Entity {
    fn new(x: u8, y: u8, w: u8, h: u8) -> Self {
        Self {
            x,
            y,
            w,
            h,
            vel: Vec2::new(0.0, 0.0),
            err: Vec2::new(0.0, 0.0),
            acc: Vec2::new(0.0, 0.03),
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
            player: Entity::new(20, 70, 20, 20),
            ground: Entity::new(0, 200, WIDTH, 55),
            platform: Entity::new(25, 175, 55, 9),
            enemy: Entity::new(90, 100, 10, 10),
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

        for contact in logics.collision.contacts.iter() {
            match (
                logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id,
            ) {
                (CollisionID::Player, CollisionID::MovingPlatform)
                | (CollisionID::MovingPlatform, CollisionID::Player) => {
                    if logics
                        .collision
                        .sides_touched(contact, &CollisionID::Player)
                        .y
                        == -1.0
                    {
                        self.player.x = (self.player.x as f32 + self.platform.vel.x).trunc() as u8;
                    }
                }
                _ => {}
            }
        }

        self.project_entity_state(&mut logics.entity_state, &logics.collision);
        logics.entity_state.update();
        self.unproject_entity_state(&logics.entity_state);
    }

    fn project_control(
        &self,
        control: &mut WinitKeyboardControl<ActionID>,
        entity_state: &FlatEntityState<StateID>,
    ) {
        control.mapping[0][0].is_valid = true;
        control.mapping[0][1].is_valid = true;
        control.mapping[0][2].is_valid = match entity_state.get_id_for_entity(1) {
            StateID::PlayerGrounded | StateID::PlayerWalk => true,
            _ => false,
        }
    }

    fn unproject_control(
        &mut self,
        control: &WinitKeyboardControl<ActionID>,
        entity_state: &FlatEntityState<StateID>,
    ) {
        self.player.vel.x = -control.values[0][0].value + control.values[0][1].value;
        match entity_state.get_id_for_entity(1) {
            StateID::PlayerGrounded | StateID::PlayerWalk => {
                let values = &control.values[0][2];
                if values.changed_by > 0.0 {
                    self.player.vel.y = -values.value * 2.0;
                }
            }
            _ => {}
        }
    }

    fn project_physics(&self, physics: &mut PointPhysics<Vec2>) {
        physics.accelerations.clear();
        physics.positions.clear();
        physics.velocities.clear();

        physics.add_physics_entity(
            Vec2::new(
                self.player.x as f32 + self.player.err.x,
                self.player.y as f32 + self.player.err.y,
            ),
            self.player.vel,
            self.player.acc,
        );
        physics.add_physics_entity(
            Vec2::new(self.platform.x as f32, self.platform.y as f32),
            self.platform.vel,
            Vec2::new(0.0, 0.0),
        );
        physics.add_physics_entity(
            Vec2::new(
                self.enemy.x as f32 + self.enemy.err.x,
                self.enemy.y as f32 + self.enemy.err.y,
            ),
            self.enemy.vel,
            self.enemy.acc,
        );
    }

    fn unproject_physics(&mut self, physics: &PointPhysics<Vec2>) {
        let update_game_state =
            |i: usize, x: &mut u8, y: &mut u8, err: &mut Vec2, vel: &mut Vec2, w: u8, h: u8| {
                *x = physics.positions[i]
                    .x
                    .trunc()
                    .max(0.0)
                    .min((WIDTH - w) as f32) as u8;
                *y = physics.positions[i]
                    .y
                    .trunc()
                    .max(0.0)
                    .min((HEIGHT - h) as f32) as u8;
                *err = physics.positions[i] - Vec2::new(*x as f32, *y as f32);
                *vel = physics.velocities[i];
            };

        update_game_state(
            0,
            &mut self.player.x,
            &mut self.player.y,
            &mut self.player.err,
            &mut self.player.vel,
            self.player.w,
            self.player.h,
        );
        update_game_state(
            1,
            &mut self.platform.x,
            &mut self.platform.y,
            &mut self.platform.err,
            &mut self.platform.vel,
            self.platform.w,
            self.platform.h,
        );
        update_game_state(
            2,
            &mut self.enemy.x,
            &mut self.enemy.y,
            &mut self.enemy.err,
            &mut self.enemy.vel,
            self.enemy.w,
            self.enemy.h,
        );
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID, Vec2>) {
        collision.centers.resize_with(4, Vec2::default);
        collision.half_sizes.resize_with(4, Vec2::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);

        collision.add_entity_as_xywh(
            Vec2::new(
                self.player.x as f32 + self.player.err.x,
                self.player.y as f32 + self.player.err.y,
            ),
            Vec2::new(self.player.w as f32, self.player.h as f32),
            self.player.vel,
            true,
            false,
            CollisionID::Player,
        );
        collision.add_entity_as_xywh(
            Vec2::new(self.ground.x as f32, self.ground.y as f32),
            Vec2::new(self.ground.w as f32, self.ground.h as f32),
            self.ground.vel,
            true,
            true,
            CollisionID::Ground,
        );
        collision.add_entity_as_xywh(
            Vec2::new(self.platform.x as f32, self.platform.y as f32),
            Vec2::new(self.platform.w as f32, self.platform.h as f32),
            self.platform.vel,
            true,
            true,
            CollisionID::MovingPlatform,
        );
        collision.add_entity_as_xywh(
            Vec2::new(
                self.enemy.x as f32 + self.enemy.err.x,
                self.enemy.y as f32 + self.enemy.err.y,
            ),
            Vec2::new(self.enemy.w as f32, self.enemy.h as f32),
            self.enemy.vel,
            true,
            false,
            CollisionID::Enemy,
        );
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID, Vec2>) {
        let player_pos_f32 = Vec2::new(
            collision.centers[4].x - collision.half_sizes[4].x,
            collision.centers[4].y - collision.half_sizes[4].y,
        );
        self.player.x = player_pos_f32.x.trunc() as u8;
        self.player.y = player_pos_f32.y.trunc() as u8;
        self.player.err = player_pos_f32 - Vec2::new(self.player.x as f32, self.player.y as f32);
        let enemy_pos_f32 = Vec2::new(
            collision.centers[7].x - collision.half_sizes[7].x,
            collision.centers[7].y - collision.half_sizes[7].y,
        );
        self.enemy.x = enemy_pos_f32.x.trunc() as u8;
        self.enemy.y = enemy_pos_f32.y.trunc() as u8;
        self.enemy.err = enemy_pos_f32 - Vec2::new(self.enemy.x as f32, self.enemy.y as f32);
    }

    fn project_entity_state(
        &self,
        entity_state: &mut FlatEntityState<StateID>,
        collision: &AabbCollision<CollisionID, Vec2>,
    ) {
        // update condition table

        // platform and enemy left right
        if self.platform.x < 30 {
            entity_state.conditions[0][1] = true;
        }
        if self.platform.x > 150 {
            entity_state.conditions[0][0] = true;
        }
        if self.enemy.x < 30 {
            entity_state.conditions[3][1] = true;
        }
        if self.enemy.x > 150 {
            entity_state.conditions[3][0] = true;
        }

        // player grounded/walk/fall/jump
        if self.player.vel.y < 0.0 {
            entity_state.conditions[1][2] = true;
        } else {
            entity_state.conditions[1][3] = true;
        }
        entity_state.conditions[2][1] = true;
        for contact in collision.contacts.iter() {
            match collision.metadata[contact.i].id {
                CollisionID::Player => match collision.metadata[contact.j].id {
                    CollisionID::Ground | CollisionID::MovingPlatform => {
                        if collision.sides_touched(contact, &CollisionID::Player).y == -1.0 {
                            if self.player.vel.x == 0.0 {
                                entity_state.conditions[1][0] = true;
                            } else {
                                entity_state.conditions[1][1] = true;
                            }
                            entity_state.conditions[1][2] = false;
                            entity_state.conditions[1][3] = false;
                        }
                    }
                    _ => {}
                },
                CollisionID::Enemy => match collision.metadata[contact.j].id {
                    CollisionID::Ground | CollisionID::MovingPlatform => {
                        if collision.sides_touched(contact, &CollisionID::Enemy).y == -1.0 {
                            entity_state.conditions[2][0] = true;
                            entity_state.conditions[2][1] = false;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn unproject_entity_state(&mut self, entity_state: &FlatEntityState<StateID>) {
        for (map, state) in entity_state.maps.iter().zip(entity_state.states.iter()) {
            match map.states[*state].id {
                StateID::PlatformLeft => self.platform.vel.x = -1.0,
                StateID::PlatformRight => self.platform.vel.x = 1.0,
                StateID::PlayerWalk | StateID::PlayerGrounded => self.player.vel.y = 0.0,
                StateID::PlayerJump | StateID::PlayerFall => {}
                StateID::EnemyLeft => self.enemy.vel.x = -1.0,
                StateID::EnemyRight => self.enemy.vel.x = 1.0,
                StateID::EnemyGrounded => self.enemy.vel.y = 0.0,
                StateID::EnemyNotGrounded => {}
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
        draw_rect(
            self.player.x,
            self.player.y,
            self.player.w,
            self.player.h,
            [0, 0, 0, 255],
            frame,
        );
        draw_rect(
            self.ground.x,
            self.ground.y,
            self.ground.w,
            self.ground.h,
            [64, 64, 64, 255],
            frame,
        );
        draw_rect(
            self.platform.x,
            self.platform.y,
            self.platform.w,
            self.platform.h,
            [64, 64, 64, 255],
            frame,
        );
        draw_rect(
            self.enemy.x,
            self.enemy.y,
            self.enemy.w,
            self.enemy.h,
            [64, 64, 64, 255],
            frame,
        );
    }
}

fn draw_rect(x: u8, y: u8, w: u8, h: u8, color: [u8; 4], frame: &mut [u8]) {
    let x = x.min(WIDTH - 1) as usize;
    let w = (w as usize).min(WIDTH as usize - x);
    let y = y.min(HEIGHT - 1) as usize;
    let h = (h as usize).min(HEIGHT as usize - y);
    for row in 0..h {
        let row_start = (WIDTH as usize) * 4 * (y + row);
        let slice = &mut frame[(row_start + x * 4)..(row_start + (x + w) * 4)];
        for pixel in slice.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }
}
