#![allow(dead_code)]
#![allow(unused_imports)]

use bevy::prelude::*;
use asterism::{QueuedResources, resources::Transaction, AabbCollision, PointPhysics, KeyboardControl, BevyKeyboardControl};

const WIDTH: u8 = 150;
const HEIGHT: u8 = 150;
const PADDLE_OFF_X: u8 = 120;
const PADDLE_HEIGHT: u8 = 48;
const PADDLE_WIDTH: u8 = 8;
const BALL_SIZE: u8 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum ActionID {
    MoveUp(Player),
    MoveDown(Player),
    Serve(Player),
}

impl Default for ActionID {
    fn default() -> Self { Self::MoveDown(Player::P1) }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
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

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum PoolID {
    Points(Player)
}

struct Logics {
    control: BevyKeyboardControl<ActionID>,
    physics: PointPhysics<Vec2>,
    collision: AabbCollision<CollisionID, Vec2>,
    resources: QueuedResources<PoolID>,
}

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
enum Player {
    P1,
    P2
}

struct Ball {
    vel: Vec2
}

struct Paddle {
    player: Player
}

struct World {
    paddles: (Paddle, Paddle),
    ball: Ball,
    ball_vel: Vec2,
    serving: Option<Player>,
    score: (u8, u8)
}

struct Serving {
    serving: Option<Player>
}


fn main() {
    let logics = Logics::new();
    App::build()
        .add_default_plugins()
        .add_resource(logics)
        .add_resource(Serving { serving: Some(Player::P1) })
        .add_startup_system(setup.system())
        .add_system(control.system())
        .add_system(physics.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // eventually make the bounds of the window what theyre actually supposed to be
    // let bounds = Vec2::new(WIDTH as f32, HEIGHT as f32);
    commands
        .spawn(Camera2dComponents::default())
        // paddle 1
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(
                Vec3::new(-(PADDLE_OFF_X as f32), 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32)),
            ..Default::default()
        })
        .with(Paddle { player: Player::P1 })
        // paddle 2
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(
                Vec3::new(PADDLE_OFF_X as f32, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32)),
            ..Default::default()
        })
        .with(Paddle { player: Player::P2 })
        // ball
        .spawn(SpriteComponents {
            material: materials.add(Color::rgb(1.0, 0.75, 0.0).into()),
            transform: Transform::from_translation(
                Vec3::new(0.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32)),
            ..Default::default()
        })
        .with(Ball { vel: Vec2::new(0.0, 0.0) });
}

fn control(
    mut logics: ResMut<Logics>,
    mut serving: ResMut<Serving>,
    input: Res<Input<KeyCode>>,
    mut ball_query: Query<(&mut Ball, &Transform)>,
    mut paddles_query: Query<(&Paddle, &mut Transform)>
) {
    project_control(&serving, &mut logics);
    logics.control.update(&input);
    unproject_control(&logics, &mut serving, &mut ball_query, &mut paddles_query);
}

fn project_control(
    serving: &Serving,
    logics: &mut Logics,
) {
    let control = &mut logics.control;
    control.mapping[0][0].is_valid = true;
    control.mapping[0][1].is_valid = true;
    control.mapping[1][0].is_valid = true;
    control.mapping[1][1].is_valid = true;

    if let Some(player) = serving.serving {
        match player {
            Player::P1 => control.mapping[0][2].is_valid = true,
            Player::P2 => control.mapping[1][2].is_valid = true,
        }
    } else {
        control.mapping[0][2].is_valid = false;
        control.mapping[1][2].is_valid = false;
    }
}

fn unproject_control(
    logics: &Logics,
    mut serving: &mut Serving,
    ball_query: &mut Query<(&mut Ball, &Transform)>,
    paddles_query: &mut Query<(&Paddle, &mut Transform)>
) {
    let control = &logics.control;
    for (paddle, mut transform) in &mut paddles_query.iter() {
        let translation = transform.translation_mut();
        match paddle.player {
            Player::P1 => {
                *translation.y_mut() += control.values[0][0].value
                    - control.values[0][1].value;
                *translation.y_mut() = translation.y()
                .max(-(HEIGHT as f32)).min(HEIGHT as f32);
            }
            Player::P2 => {
                *translation.y_mut() += control.values[1][0].value
                    - control.values[1][1].value;
                *translation.y_mut() = translation.y()
                .max(-(HEIGHT as f32)).min(HEIGHT as f32);
            }
        }
    }

    for (mut ball, _) in &mut ball_query.iter() {
        if let Some(player) = serving.serving {
            match player {
                Player::P1 => {
                    let values = &control.values[0][2];
                    if values.changed_by == 1.0 && values.value != 0.0 {
                        ball.vel = Vec2::new(1.0, -1.0);
                        serving.serving = None;
                    }
                }
                Player::P2 => {
                    let values = &control.values[1][2];
                    if values.changed_by == 1.0 && values.value != 0.0 {
                        ball.vel = Vec2::new(-1.0, 1.0);
                        serving.serving = None;
                    }
                }
            }
        }
    }
}

fn physics(
    mut logics: ResMut<Logics>,
    mut ball_query: Query<(&mut Ball, &mut Transform)>
) {
    project_physics(&mut logics, &mut ball_query);
    logics.physics.update();
    unproject_physics(&logics, &mut ball_query);
}

fn project_physics(
    logics: &mut Logics,
    ball_query: &mut Query<(&mut Ball, &mut Transform)>
) {
    let physics = &mut logics.physics;
    physics.positions.resize_with(1, Vec2::default);
    physics.velocities.resize_with(1, Vec2::default);
    physics.accelerations.resize_with(1, Vec2::default);
    for (ball, transform) in &mut ball_query.iter() {
        physics.add_physics_entity(0,
            {
                let translation = transform.translation();
                Vec2::new(translation.x(), translation.y())
            },
            ball.vel,
            Vec2::new(0.0, 0.0));
    }
}

fn unproject_physics(
    logics: &Logics,
    ball_query: &mut Query<(&mut Ball, &mut Transform)>,
) {
    let physics = &logics.physics;
    for (mut ball, mut transform) in &mut ball_query.iter() {
        ball.vel = physics.velocities[0];
        transform.translate(Vec3::new(
                ball.vel.x(),
                ball.vel.y(),
                0.0));
    }
}


impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = BevyKeyboardControl::new();
                control.add_key_map(0,
                    KeyCode::Q,
                    ActionID::MoveUp(Player::P1),
                );
                control.add_key_map(0,
                    KeyCode::A,
                    ActionID::MoveDown(Player::P1),
                );
                control.add_key_map(0,
                    KeyCode::W,
                    ActionID::Serve(Player::P1),
                );
                control.add_key_map(1,
                    KeyCode::O,
                    ActionID::MoveUp(Player::P2),
                );
                control.add_key_map(1,
                    KeyCode::L,
                    ActionID::MoveDown(Player::P2),
                );
                control.add_key_map(1,
                    KeyCode::I,
                    ActionID::Serve(Player::P2),
                );
                control
            },
            physics: PointPhysics::new(),
            collision: AabbCollision::new(),
            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert( PoolID::Points(Player::P1), 0.0 );
                resources.items.insert( PoolID::Points(Player::P2), 0.0 );
                resources
            },
        }
    }
}

/*
 * this is the old world.update() function
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
                    match player {
                        Player::P1 => {
                            logics.resources.transactions.push(vec![(PoolID::Points(Player::P2), Transaction::Change(1))]);
                            self.serving = Some(Player::P2);
                        }
                        Player::P2 => {
                            logics.resources.transactions.push(vec![(PoolID::Points(Player::P1), Transaction::Change(1))]);
                            self.serving = Some(Player::P1);
                        }
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
            if *completed {
                for item_type in item_types {
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

these are project/unproject fns

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID>, control: &BevyKeyboardControl<ActionID>) {
        collision.bodies.resize_with(4, Aabb::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);
        collision.add_collision_entity(
            self.ball.0 as f32, self.ball.1 as f32,
            BALL_SIZE as f32, BALL_SIZE as f32,
            self.ball_vel,
            true, false, CollisionID::Ball);
        collision.add_collision_entity(
            PADDLE_OFF_X as f32, self.paddles.0 as f32,
            PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32,
            Vec2::new(0.0, -control.values[0][0].value + control.values[0][1].value),
                true, true, CollisionID::Paddle(Player::P1));
        collision.add_collision_entity(
            (WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32, self.paddles.1 as f32,
            PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32,
            Vec2::new(0.0, -control.values[1][0].value + control.values[1][1].value),
            true, true, CollisionID::Paddle(Player::P2));
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

    fn project_resources(&self, resources: &mut QueuedResources<PoolID>) {
        if !resources.items.contains_key(&PoolID::Points(Player::P1)) {
            resources.items.insert(PoolID::Points(Player::P1), 0.0);
        }
        if !resources.items.contains_key(&PoolID::Points(Player::P2)) {
            resources.items.insert(PoolID::Points(Player::P1), 0.0);
        }
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID>) {
        for (completed, item_types) in resources.completed.iter() {
            if *completed {
                for item_type in item_types {
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
*/
