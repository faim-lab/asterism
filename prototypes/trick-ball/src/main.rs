/// trick ball: Ball speed slows down after colliding with paddles. Speedup after colliding with walls
use asterism::{
    collision::{AabbCollision, Vec2 as AstVec2},
    control::{KeyboardControl, MacroQuadKeyboardControl},
    physics::PointPhysics,
    resources::{QueuedResources, Transaction},
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
    Ball,
    BounceWall,
    ScoreWall(Player),
}

impl Default for CollisionID {
    fn default() -> Self {
        Self::Ball
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum PoolID {
    Points(Player),
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
                    -2.0,
                    0.0,
                    2.0,
                    HEIGHT as f32,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::ScoreWall(Player::P1),
                );
                // right
                collision.add_entity_as_xywh(
                    WIDTH as f32,
                    0.0,
                    2.0,
                    HEIGHT as f32,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::ScoreWall(Player::P2),
                );
                // top
                collision.add_entity_as_xywh(
                    0.0,
                    -2.0,
                    WIDTH as f32,
                    2.0,
                    Vec2::new(0.0, 0.0),
                    true,
                    true,
                    CollisionID::BounceWall,
                );
                // bottom
                collision.add_entity_as_xywh(
                    0.0,
                    HEIGHT as f32,
                    WIDTH as f32,
                    2.0,
                    Vec2::new(0.0, 0.0),
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

struct World {
    paddles: (u8, u8),
    ball: Vec2,
    ball_vel: Vec2,
    serving: Option<Player>,
    score: (u8, u8),
}

fn window_conf() -> Conf {
    Conf {
        window_title: "trick ball".to_owned(),
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
            ball: Vec2::new(
                (WIDTH / 2 - BALL_SIZE / 2) as f32,
                (HEIGHT / 2 - BALL_SIZE / 2) as f32,
            ),
            ball_vel: Vec2::new(0.0, 0.0),
            serving: Some(Player::P1),
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
                (CollisionID::ScoreWall(player), CollisionID::Ball)
                | (CollisionID::Ball, CollisionID::ScoreWall(player)) => {
                    self.ball_vel = Vec2::new(0.0, 0.0);
                    self.ball = Vec2::new(
                        (WIDTH / 2 - BALL_SIZE / 2) as f32,
                        (HEIGHT / 2 - BALL_SIZE / 2) as f32,
                    );
                    match player {
                        Player::P1 => {
                            logics
                                .resources
                                .transactions
                                .push(vec![(PoolID::Points(Player::P2), Transaction::Change(1.0))]);
                            self.serving = Some(Player::P2);
                        }
                        Player::P2 => {
                            logics
                                .resources
                                .transactions
                                .push(vec![(PoolID::Points(Player::P1), Transaction::Change(1.0))]);
                            self.serving = Some(Player::P1);
                        }
                    }
                }

                (CollisionID::Ball, CollisionID::BounceWall)
                | (CollisionID::BounceWall, CollisionID::Ball) => {
                    self.ball_vel.y *= -1.0;
                    if self.ball_vel.magnitude() < 5.0 {
                        self.ball_vel *= 1.1;
                    }
                }

                (CollisionID::Paddle(player), CollisionID::Ball)
                | (CollisionID::Ball, CollisionID::Paddle(player)) => {
                    let sides_touched = logics.collision.sides_touched(contact, &CollisionID::Ball);
                    if match player {
                        Player::P1 => sides_touched.x == 1.0,
                        Player::P2 => sides_touched.x == -1.0,
                    } {
                        self.ball_vel.x *= -1.0;
                    } else {
                        self.ball_vel.y *= -1.0;
                    }
                    self.change_angle(player);
                    if self.ball_vel.magnitude() > 0.5 {
                        self.ball_vel *= 0.9;
                    }
                }

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
                                Player::P2 => print!("p2"),
                            }
                            println!(" scores! p1: {}, p2: {}", self.score.0, self.score.1);
                        }
                    }
                }
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

        if (self.ball_vel.x, self.ball_vel.y) == (0.0, 0.0) {
            match self.serving {
                Some(Player::P1) => control.mapping[0][2].is_valid = true,
                Some(Player::P2) => control.mapping[1][2].is_valid = true,
                None => {}
            }
        } else {
            control.mapping[0][2].is_valid = false;
            control.mapping[1][2].is_valid = false;
        }
    }

    fn unproject_control(&mut self, control: &MacroQuadKeyboardControl<ActionID>) {
        self.paddles.0 = ((self.paddles.0 as i16
            + control
                .get_action_in_set(0, ActionID::MoveDown(Player::P1))
                .unwrap()
                .value as i16
            - control
                .get_action_in_set(0, ActionID::MoveUp(Player::P1))
                .unwrap()
                .value as i16)
            .max(0) as u8)
            .min(255 - PADDLE_HEIGHT);
        self.paddles.1 = ((self.paddles.1 as i16
            + control
                .get_action_in_set(1, ActionID::MoveDown(Player::P2))
                .unwrap()
                .value as i16
            - control
                .get_action_in_set(1, ActionID::MoveUp(Player::P2))
                .unwrap()
                .value as i16)
            .max(0) as u8)
            .min(255 - PADDLE_HEIGHT);

        if (self.ball_vel.x, self.ball_vel.y) == (0.0, 0.0) {
            match self.serving {
                Some(Player::P1) => {
                    let values = control
                        .get_action_in_set(0, ActionID::Serve(Player::P1))
                        .unwrap();
                    if values.changed_by == 1.0 && values.value != 0.0 {
                        self.ball_vel.x = 1.0;
                        self.ball_vel.y = 1.0;
                    }
                }
                Some(Player::P2) => {
                    let values = control
                        .get_action_in_set(1, ActionID::Serve(Player::P2))
                        .unwrap();
                    if values.changed_by == 1.0 && values.value != 0.0 {
                        self.ball_vel.x = -1.0;
                        self.ball_vel.y = -1.0;
                    }
                }
                None => {}
            }
        }
    }

    fn project_physics(&self, physics: &mut PointPhysics<Vec2>) {
        physics.positions.clear();
        physics.velocities.clear();
        physics.accelerations.clear();
        physics.add_physics_entity(self.ball, self.ball_vel, Vec2::new(0.0, 0.0));
    }

    fn unproject_physics(&mut self, physics: &PointPhysics<Vec2>) {
        self.ball.x = physics.positions[0]
            .x
            .max(0.0)
            .min((WIDTH - BALL_SIZE) as f32);
        self.ball.y = physics.positions[0]
            .y
            .max(0.0)
            .min((HEIGHT - BALL_SIZE) as f32);
        self.ball_vel = physics.velocities[0];
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

        collision.add_entity_as_xywh(
            self.ball.x,
            self.ball.y,
            BALL_SIZE as f32,
            BALL_SIZE as f32,
            self.ball_vel,
            true,
            false,
            CollisionID::Ball,
        );

        collision.add_entity_as_xywh(
            PADDLE_OFF_X as f32,
            self.paddles.0 as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            Vec2::new(0.0, control.values[0][1].value - control.values[0][0].value),
            true,
            true,
            CollisionID::Paddle(Player::P1),
        );

        collision.add_entity_as_xywh(
            (WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32,
            self.paddles.1 as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            Vec2::new(0.0, control.values[1][1].value - control.values[1][0].value),
            true,
            true,
            CollisionID::Paddle(Player::P2),
        );
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID, Vec2>) {
        self.ball = collision.get_xy_pos_for_entity(CollisionID::Ball).unwrap();
    }

    fn change_angle(&mut self, player: Player) {
        let paddle_center = match player {
            Player::P1 => self.paddles.0 + PADDLE_HEIGHT / 2,
            Player::P2 => self.paddles.1 + PADDLE_HEIGHT / 2,
        } as f32;
        let angle: f32 = (((self.ball.y + (BALL_SIZE / 2) as f32) - paddle_center)
            .max(-(PADDLE_HEIGHT as f32) / 2.0)
            .min(PADDLE_HEIGHT as f32 / 2.0)
            / PADDLE_HEIGHT as f32)
            .abs()
            * 80.0;
        let magnitude = self.ball_vel.magnitude();
        self.ball_vel.x =
            angle.to_radians().cos() * magnitude * if self.ball_vel.x < 0.0 { -1.0 } else { 1.0 };
        self.ball_vel.y =
            angle.to_radians().sin() * magnitude * if self.ball_vel.y < 0.0 { -1.0 } else { 1.0 };
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
        for (completed, item_types) in resources.completed.iter() {
            if *completed {
                for item_type in item_types {
                    let value = resources.get_value_by_itemtype(item_type) as u8;
                    match item_type {
                        PoolID::Points(player) => match player {
                            Player::P1 => self.score.0 = value,
                            Player::P2 => self.score.1 = value,
                        },
                    }
                }
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
        draw_rectangle(
            self.ball.x,
            self.ball.y,
            BALL_SIZE as f32,
            BALL_SIZE as f32,
            Color::new(1., 0.75, 0., 1.),
        );
    }
}
