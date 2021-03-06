/// paddle ball mania: There are multiple balls at a time. This number does not change throughout the game. there's no serving field anymore, btw; i like the amount of chaos in this one
use asterism::{
    collision::{AabbCollision, Vec2 as AstVec2},
    control::{KeyboardControl, MacroQuadKeyboardControl},
    physics::PointPhysics,
    resources::{PoolInfo, QueuedResources, Transaction},
    Logic,
};
use macroquad::prelude::*;

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const PADDLE_OFF_X: u8 = 16;
const PADDLE_HEIGHT: u8 = 48;
const PADDLE_WIDTH: u8 = 8;
const BALL_SIZE: u8 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum ActionID {
    MoveUp(Player),
    MoveDown(Player),
    Serve(Player),
    Quit,
}

impl Default for ActionID {
    fn default() -> Self {
        Self::MoveDown(Player::P1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
enum CollisionID {
    Paddle(Player),
    Ball(usize),
    BounceWall,
    ScoreWall(Player),
}

impl Default for CollisionID {
    fn default() -> Self {
        Self::Ball(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum PoolID {
    Points(Player),
}

impl PoolInfo for PoolID {
    fn max_value(&self) -> f64 {
        match self {
            Self::Points(_) => std::u8::MAX as f64,
        }
    }

    fn min_value(&self) -> f64 {
        match self {
            Self::Points(_) => std::u8::MIN as f64,
        }
    }
}

struct Logics {
    control: MacroQuadKeyboardControl<ActionID>,
    physics: PointPhysics<Vec2>,
    collision: AabbCollision<CollisionID, Vec2>,
    resources: QueuedResources<PoolID>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = MacroQuadKeyboardControl::new();
                control.add_key_map(0, KeyCode::Q, ActionID::MoveUp(Player::P1));
                control.add_key_map(0, KeyCode::A, ActionID::MoveDown(Player::P1));
                control.add_key_map(0, KeyCode::W, ActionID::Serve(Player::P1));
                control.add_key_map(1, KeyCode::O, ActionID::MoveUp(Player::P2));
                control.add_key_map(1, KeyCode::L, ActionID::MoveDown(Player::P2));
                control.add_key_map(1, KeyCode::I, ActionID::Serve(Player::P2));
                control.add_key_map(2, KeyCode::Escape, ActionID::Quit);
                control
            },
            physics: PointPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                // left
                collision.add_entity_as_xywh(
                    Vec2::new(-2.0, 0.0),
                    Vec2::new(2.0, HEIGHT as f32),
                    Vec2::zero(),
                    true,
                    true,
                    CollisionID::ScoreWall(Player::P1),
                );
                // right
                collision.add_entity_as_xywh(
                    Vec2::new(WIDTH as f32, 0.0),
                    Vec2::new(2.0, HEIGHT as f32),
                    Vec2::zero(),
                    true,
                    true,
                    CollisionID::ScoreWall(Player::P2),
                );
                // top
                collision.add_entity_as_xywh(
                    Vec2::new(0.0, -2.0),
                    Vec2::new(WIDTH as f32, 2.0),
                    Vec2::zero(),
                    true,
                    true,
                    CollisionID::BounceWall,
                );
                // bottom
                collision.add_entity_as_xywh(
                    Vec2::new(0.0, HEIGHT as f32),
                    Vec2::new(WIDTH as f32, 2.0),
                    Vec2::zero(),
                    true,
                    true,
                    CollisionID::BounceWall,
                );
                collision
            },
            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert(PoolID::Points(Player::P1), 0.0);
                resources.items.insert(PoolID::Points(Player::P2), 0.0);
                resources
            },
        }
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Debug)]
enum Player {
    P1,
    P2,
}

struct Ball {
    pos: Vec2,
    vel: Vec2,
    starting_pos: Vec2,
}

impl Ball {
    fn new(pos: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::zero(),
            starting_pos: pos,
        }
    }
}

struct World {
    paddles: (u8, u8),
    balls: Vec<Ball>,
    score: (u32, u32),
}

fn window_conf() -> Conf {
    Conf {
        window_title: "paddles".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = World::new();
    let mut logics = Logics::new();

    loop {
        if let Ok(cont) = world.update(&mut logics) {
            if !cont {
                break;
            }
        }
        world.draw();
        next_frame().await;
    }
}

impl World {
    fn new() -> Self {
        Self {
            paddles: (
                HEIGHT / 2 - PADDLE_HEIGHT / 2,
                HEIGHT / 2 - PADDLE_HEIGHT / 2,
            ),
            balls: vec![
                Ball::new(Vec2::new((WIDTH / 2 - BALL_SIZE / 2) as f32, 50.0)),
                Ball::new(Vec2::new((WIDTH / 2 - BALL_SIZE / 2) as f32, 100.0)),
                Ball::new(Vec2::new((WIDTH / 2 - BALL_SIZE / 2) as f32, 150.0)),
                Ball::new(Vec2::new((WIDTH / 2 - BALL_SIZE / 2) as f32, 200.0)),
            ],
            score: (0, 0),
        }
    }

    fn update(&mut self, logics: &mut Logics) -> Result<bool, ()> {
        self.project_control(&mut logics.control);
        logics.control.update(&());
        self.unproject_control(&logics.control);

        if logics.control.values[2][0].value != 0.0 {
            return Ok(false);
        }

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        for contact in logics.collision.contacts.iter() {
            match (
                logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id,
            ) {
                (CollisionID::Ball(i), CollisionID::ScoreWall(player))
                | (CollisionID::ScoreWall(player), CollisionID::Ball(i)) => {
                    self.balls[i].vel = Vec2::zero();
                    self.balls[i].pos = self.balls[i].starting_pos;
                    match player {
                        Player::P1 => {
                            logics
                                .resources
                                .transactions
                                .push(vec![(PoolID::Points(Player::P2), Transaction::Change(1.0))]);
                        }
                        Player::P2 => {
                            logics
                                .resources
                                .transactions
                                .push(vec![(PoolID::Points(Player::P1), Transaction::Change(1.0))]);
                        }
                    }
                }

                (CollisionID::BounceWall, CollisionID::Ball(i))
                | (CollisionID::Ball(i), CollisionID::BounceWall) => {
                    self.balls[i].vel.y *= -1.0;
                }

                (CollisionID::Ball(i), CollisionID::Ball(j)) => {
                    let sides_touched = logics
                        .collision
                        .sides_touched(contact, &CollisionID::Ball(i));
                    if sides_touched.x != 0.0 {
                        // restituted left/right
                        self.balls[i].vel.x *= -1.0;
                        self.balls[j].vel.x *= -1.0;
                    }
                    if sides_touched.y != 0.0 {
                        // restituted up/down
                        self.balls[i].vel.y *= -1.0;
                        self.balls[j].vel.y *= -1.0;
                    }
                }

                (CollisionID::Paddle(player), CollisionID::Ball(i))
                | (CollisionID::Ball(i), CollisionID::Paddle(player)) => {
                    if match player {
                        Player::P1 => {
                            logics
                                .collision
                                .sides_touched(contact, &CollisionID::Ball(i))
                                .x
                                == 1.0
                        }
                        Player::P2 => {
                            logics
                                .collision
                                .sides_touched(contact, &CollisionID::Ball(i))
                                .x
                                == -1.0
                        }
                    } {
                        self.balls[i].vel.x *= -1.0;
                    } else {
                        self.balls[i].vel.y *= -1.0;
                    }
                    self.change_angle(i, player);
                    if self.balls[i].vel.magnitude() < 5.0 {
                        self.balls[i].vel *= 1.1;
                    }
                }

                _ => {}
            }
        }

        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources);

        for completed in logics.resources.completed.iter() {
            match completed {
                Ok(item_types) => {
                    for item_type in item_types {
                        match item_type {
                            PoolID::Points(player) => {
                                match player {
                                    Player::P1 => print!("p1"),
                                    Player::P2 => print!("p2"),
                                }
                                println!(" scores! p1: {}, p2: {}", self.score.0, self.score.1);
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        }

        Ok(true)
    }

    fn project_control(&self, control: &mut MacroQuadKeyboardControl<ActionID>) {
        control.mapping[0][0].is_valid = true;
        control.mapping[0][1].is_valid = true;
        control.mapping[1][0].is_valid = true;
        control.mapping[1][1].is_valid = true;
        control.mapping[2][0].is_valid = true;
        control.mapping[0][2].is_valid = true;
        control.mapping[1][2].is_valid = true;
    }

    fn unproject_control(&mut self, control: &MacroQuadKeyboardControl<ActionID>) {
        self.paddles.0 = ((self.paddles.0 as i16
            - control
                .get_action_in_set(0, ActionID::MoveUp(Player::P1))
                .unwrap()
                .value as i16
            + control
                .get_action_in_set(0, ActionID::MoveDown(Player::P1))
                .unwrap()
                .value as i16)
            .max(0) as u8)
            .min(255 - PADDLE_HEIGHT);
        self.paddles.1 = ((self.paddles.1 as i16
            - control
                .get_action_in_set(1, ActionID::MoveUp(Player::P2))
                .unwrap()
                .value as i16
            + control
                .get_action_in_set(1, ActionID::MoveDown(Player::P2))
                .unwrap()
                .value as i16)
            .max(0) as u8)
            .min(255 - PADDLE_HEIGHT);

        for Ball { vel: ball_vel, .. } in self.balls.iter_mut() {
            if (ball_vel.x, ball_vel.y) == (0.0, 0.0) {
                let values = control
                    .get_action_in_set(0, ActionID::Serve(Player::P1))
                    .unwrap();
                if values.changed_by == 1.0 && values.value != 0.0 {
                    ball_vel.x = 1.0;
                    ball_vel.y = 1.0;
                }
                let values = control
                    .get_action_in_set(1, ActionID::Serve(Player::P2))
                    .unwrap();
                if values.changed_by == 1.0 && values.value != 0.0 {
                    ball_vel.x = -1.0;
                    ball_vel.y = -1.0;
                }
            }
        }
    }

    fn project_physics(&self, physics: &mut PointPhysics<Vec2>) {
        physics.positions.clear();
        physics.velocities.clear();
        physics.accelerations.clear();
        for Ball { pos, vel, .. } in self.balls.iter() {
            physics.add_physics_entity(*pos, *vel, Vec2::new(0.0, 0.0));
        }
    }

    fn unproject_physics(&mut self, physics: &PointPhysics<Vec2>) {
        for (ball, phys_pos) in self.balls.iter_mut().zip(physics.positions.iter()) {
            ball.pos = *phys_pos;
        }
    }

    fn project_collision(
        &self,
        collision: &mut AabbCollision<CollisionID, Vec2>,
        control: &MacroQuadKeyboardControl<ActionID>,
    ) {
        collision.centers.resize_with(4, Default::default);
        collision.half_sizes.resize_with(4, Default::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);

        let ball_size = Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32);
        for (i, Ball { pos, vel, .. }) in self.balls.iter().enumerate() {
            collision.add_entity_as_xywh(
                Vec2::new(pos.x, pos.y),
                ball_size,
                *vel,
                true,
                false,
                CollisionID::Ball(i),
            );
        }

        collision.add_entity_as_xywh(
            Vec2::new(PADDLE_OFF_X as f32, self.paddles.0 as f32),
            Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32),
            Vec2::new(0.0, control.values[0][1].value - control.values[0][0].value),
            true,
            true,
            CollisionID::Paddle(Player::P1),
        );

        collision.add_entity_as_xywh(
            Vec2::new(
                (WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32,
                self.paddles.1 as f32,
            ),
            Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32),
            Vec2::new(0.0, control.values[1][1].value - control.values[1][0].value),
            true,
            true,
            CollisionID::Paddle(Player::P2),
        );
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID, Vec2>) {
        for (i, ball) in self.balls.iter_mut().enumerate() {
            ball.pos = collision
                .get_xy_pos_for_entity(CollisionID::Ball(i))
                .unwrap();
        }
    }

    fn change_angle(&mut self, ball_idx: usize, player: Player) {
        let ball = &mut self.balls[ball_idx];
        let paddle_center = match player {
            Player::P1 => self.paddles.0 + PADDLE_HEIGHT / 2,
            Player::P2 => self.paddles.1 + PADDLE_HEIGHT / 2,
        } as f32;
        let angle: f32 = (((ball.pos.y + (BALL_SIZE / 2) as f32) - paddle_center)
            .max(-(PADDLE_HEIGHT as f32) / 2.0)
            .min(PADDLE_HEIGHT as f32 / 2.0)
            / PADDLE_HEIGHT as f32)
            .abs()
            * 80.0;
        let magnitude = ball.vel.magnitude();
        ball.vel.x =
            angle.to_radians().cos() * magnitude * if ball.vel.x < 0.0 { -1.0 } else { 1.0 };
        ball.vel.y =
            angle.to_radians().sin() * magnitude * if ball.vel.y < 0.0 { -1.0 } else { 1.0 };
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID>) {
        if !resources.items.contains_key(&PoolID::Points(Player::P1)) {
            resources.items.insert(PoolID::Points(Player::P1), 0.0);
        }
        if !resources.items.contains_key(&PoolID::Points(Player::P2)) {
            resources.items.insert(PoolID::Points(Player::P2), 0.0);
        }
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID>) {
        for completed in resources.completed.iter() {
            match completed {
                Ok(item_types) => {
                    for item_type in item_types {
                        let value = resources.get_value_by_itemtype(item_type).unwrap() as u32;
                        match item_type {
                            PoolID::Points(player) => match player {
                                Player::P1 => self.score.0 = value,
                                Player::P2 => self.score.1 = value,
                            },
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    fn draw(&self) {
        clear_background(Color::new(0., 0., 0.5, 1.));
        draw_rectangle(
            PADDLE_OFF_X as f32,
            self.paddles.0 as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            WHITE,
        );
        draw_rectangle(
            (WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32,
            self.paddles.1 as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            WHITE,
        );
        for Ball { pos, .. } in self.balls.iter() {
            draw_rectangle(
                pos.x,
                pos.y,
                BALL_SIZE as f32,
                BALL_SIZE as f32,
                Color::new(1., 0.75, 0., 1.),
            );
        }
    }
}
