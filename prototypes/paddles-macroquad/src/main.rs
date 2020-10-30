use asterism::{QueuedResources, resources::Transaction, AabbCollision, PointPhysics, KeyboardControl, MacroQuadKeyboardControl};
use macroquad::{self as mq, *};

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
}

impl Default for ActionID {
    fn default() -> Self { Self::MoveDown(Player::P1) }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
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
                control.add_key_map(0,
                    mq::KeyCode::Q,
                    ActionID::MoveUp(Player::P1),
                );
                control.add_key_map(0,
                    mq::KeyCode::A,
                    ActionID::MoveDown(Player::P1),
                );
                control.add_key_map(0,
                    mq::KeyCode::W,
                    ActionID::Serve(Player::P1),
                );
                control.add_key_map(1,
                    mq::KeyCode::O,
                    ActionID::MoveUp(Player::P2),
                );
                control.add_key_map(1,
                    mq::KeyCode::L,
                    ActionID::MoveDown(Player::P2),
                );
                control.add_key_map(1,
                    mq::KeyCode::I,
                    ActionID::Serve(Player::P2),
                );
                control
            },
            physics: PointPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                collision.add_entity_as_xywh(-2.0, 0.0,
                    2.0, HEIGHT as f32,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::SideWall(Player::P1));
                collision.add_entity_as_xywh(WIDTH as f32, 0.0,
                    2.0, HEIGHT as f32,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::SideWall(Player::P2));
                collision.add_entity_as_xywh(0.0, -2.0,
                    WIDTH as f32, 2.0,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::TopWall);
                collision.add_entity_as_xywh(0.0, HEIGHT as f32,
                    WIDTH as f32, 2.0,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::BottomWall);
                collision
            },
            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert( PoolID::Points(Player::P1), 0.0 );
                resources.items.insert( PoolID::Points(Player::P2), 0.0 );
                resources
            }
        }
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Debug)]
enum Player {
    P1,
    P2
}


struct World {
    paddles: (u8, u8),
    ball: Vec2,
    ball_vel: Vec2,
    serving: Option<Player>,
    score: (u8, u8)
}

#[mq::main("Paddles")]
async fn main() {
    let mut world = World::new();
    let mut logics = Logics::new();

    loop {
        world.update(&mut logics);
        world.draw();
        next_frame().await;
    }
}

impl World {
    fn new() -> Self {
        Self {
            paddles: (HEIGHT / 2 - PADDLE_HEIGHT / 2, HEIGHT / 2 - PADDLE_HEIGHT / 2),
            ball: Vec2::new((WIDTH / 2 - BALL_SIZE / 2) as f32, (HEIGHT / 2 - BALL_SIZE / 2) as f32),
            ball_vel: Vec2::new(0.0, 0.0),
            serving: Some(Player::P1),
            score: (0, 0),
        }
    }

    fn update(&mut self, logics: &mut Logics) {
        self.project_control(&mut logics.control);
        logics.control.update(&());
        self.unproject_control(&logics.control);

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        for contact in logics.collision.contacts.iter() {
            match (logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id) {
                (CollisionID::SideWall(player), CollisionID::Ball) => {
                    self.ball_vel = Vec2::new(0.0, 0.0);
                    self.ball = Vec2::new((WIDTH / 2 - BALL_SIZE / 2) as f32,
                        (HEIGHT / 2 - BALL_SIZE / 2) as f32);
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
                        self.ball_vel.set_y(self.ball_vel.y() * -1.0);
                    }
                (CollisionID::Ball, CollisionID::Paddle(player)) => {
                    if match player {
                        Player::P1 =>
                            (self.ball.x() - (PADDLE_OFF_X + PADDLE_WIDTH) as f32).abs()
                            > ((self.ball.y() + BALL_SIZE as f32) - self.paddles.0 as f32).abs().min((self.ball.y() - (self.paddles.0 + PADDLE_HEIGHT) as f32).abs()),
                        Player::P2 =>
                            ((self.ball.x() + BALL_SIZE as f32) - (WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32).abs()
                            > ((self.ball.y() + BALL_SIZE as f32) - self.paddles.1 as f32).abs().min((self.ball.y() - (self.paddles.1 + PADDLE_HEIGHT) as f32).abs()),
                    } {
                        self.ball_vel.set_y(self.ball_vel.y() * -1.0);
                    } else {
                        self.ball_vel.set_x(self.ball_vel.x() * -1.0);
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
    }

    fn project_control(&self, control: &mut MacroQuadKeyboardControl<ActionID>) {
        control.mapping[0][0].is_valid = true;
        control.mapping[0][1].is_valid = true;
        control.mapping[1][0].is_valid = true;
        control.mapping[1][1].is_valid = true;

        if (self.ball_vel.x(), self.ball_vel.y()) == (0.0, 0.0) {
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
        self.paddles.0 = ((self.paddles.0 as i16 -
                control.values[0][0].value as i16 +
                control.values[0][1].value as i16)
            .max(0) as u8).min(255 - PADDLE_HEIGHT);
        self.paddles.1 = ((self.paddles.1 as i16 -
                control.values[1][0].value as i16 +
                control.values[1][1].value as i16)
            .max(0) as u8).min(255 - PADDLE_HEIGHT);
        if (self.ball_vel.x(), self.ball_vel.y()) == (0.0, 0.0) {
            match self.serving {
                Some(Player::P1) => {
                    let values = &control.values[0][2];
                    if values.changed_by == 1.0 && values.value != 0.0 {
                        self.ball_vel.set_x(1.0);
                        self.ball_vel.set_y(1.0);
                    }
                }
                Some(Player::P2) => {
                    let values = &control.values[1][2];
                    if values.changed_by == 1.0 && values.value != 0.0 {
                        self.ball_vel.set_x(-1.0);
                        self.ball_vel.set_y(-1.0);
                    }
                }
                None => {}
            }
        }
    }

    fn project_physics(&self, physics: &mut PointPhysics<Vec2>) {
        physics.positions.resize_with(1, Vec2::default);
        physics.velocities.resize_with(1, Vec2::default);
        physics.accelerations.resize_with(1, Vec2::default);
        physics.add_physics_entity(0,
            self.ball,
            self.ball_vel,
            Vec2::new(0.0, 0.0));
    }

    fn unproject_physics(&mut self, physics: &PointPhysics<Vec2>) {
        self.ball.set_x(physics.positions[0].x().max(0.0).min((WIDTH - BALL_SIZE) as f32));
        self.ball.set_y(physics.positions[0].y().max(0.0).min((HEIGHT - BALL_SIZE) as f32));
        self.ball_vel = physics.velocities[0];
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID, Vec2>, control: &MacroQuadKeyboardControl<ActionID>) {
        collision.centers.resize_with(4, Default::default);
        collision.half_sizes.resize_with(4, Default::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);

        collision.add_entity_as_xywh(
            self.ball.x(), self.ball.y(),
            BALL_SIZE as f32, BALL_SIZE as f32,
            self.ball_vel,
            true, false, CollisionID::Ball);

        collision.add_entity_as_xywh(
            PADDLE_OFF_X as f32,
            self.paddles.0 as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            Vec2::new(0.0,
                control.values[0][1].value - control.values[0][0].value),
                true, true, CollisionID::Paddle(Player::P1));

        collision.add_entity_as_xywh(
            (WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32,
            self.paddles.1 as f32,
            PADDLE_WIDTH as f32,
            PADDLE_HEIGHT as f32,
            Vec2::new(0.0,
                control.values[1][1].value - control.values[1][0].value),
            true, true, CollisionID::Paddle(Player::P2));
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID, Vec2>) {
        self.ball.set_x(collision.centers[4].x() - collision.half_sizes[4].x());
        self.ball.set_y(collision.centers[4].y() - collision.half_sizes[4].y());
    }

    fn change_angle(&mut self, player: Player) {
        let paddle_center = match player {
            Player::P1 => self.paddles.0 + PADDLE_HEIGHT / 2,
            Player::P2 => self.paddles.1 + PADDLE_HEIGHT / 2
        } as f32;
        let angle: f32 = (((self.ball.y() + (BALL_SIZE / 2) as f32) - paddle_center)
            .max(-(PADDLE_HEIGHT as f32) / 2.0)
            .min(PADDLE_HEIGHT as f32 / 2.0) / PADDLE_HEIGHT as f32).abs() * 80.0;
        let magnitude = f32::sqrt(self.ball_vel.x().powi(2) + self.ball_vel.y().powi(2));
        self.ball_vel.set_x(angle.to_radians().cos() * magnitude
            * if self.ball_vel.x() < 0.0 { -1.0 } else { 1.0 });
        self.ball_vel.set_y(angle.to_radians().sin() * magnitude
            * if self.ball_vel.y() < 0.0 { -1.0 } else { 1.0 });
        if magnitude < 5.0 {
            self.ball_vel *= 1.1;
        }
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

    fn draw(&self) {
        mq::clear_background(mq::Color::new(0., 0., 0.5, 1.));
        mq::draw_rectangle(PADDLE_OFF_X as f32, self.paddles.0 as f32,
            PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32,
            mq::WHITE);
        mq::draw_rectangle((WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32, self.paddles.1 as f32,
            PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32,
            mq::WHITE);
        mq::draw_rectangle(self.ball.x(), self.ball.y(),
            BALL_SIZE as f32, BALL_SIZE as f32,
            mq::Color::new(1., 200., 0., 1.));
    }
}

