use asterism::{
    animation::{AnimObject, BackElement, SimpleAnim},
    collision::AabbCollision,
    control::{KeyboardControl, MacroquadInputWrapper},
    physics::PointPhysics,
    resources::{QueuedResources, Transaction},
};
use json::*;
use macroquad::prelude::*;

const WIDTH: u32 = 255;
const HEIGHT: u32 = 255;
const PADDLE_OFF_Y: u32 = 36;
const PADDLE_HEIGHT: u32 = 36;
const PADDLE_WIDTH: u32 = 36;
const BALL_SIZE: u32 = 36;
const BALL_NUM: u8 = 3;
const WALL_DEPTH: u32 = 8;
const GOAL_WIDTH: u32 = 72;
//number of distinct ball sprites, number of rows in spritesheet - paddle rows
const DIST_BALL: u32 = 3;

const BASE_COLOR: Color = Color::new(0.86, 1., 0.86, 1.);
const FENCE_COLOR: Color = Color::new(0.94, 0.9, 0.54, 1.);

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum ActionID {
    MoveRight(Player),
    MoveLeft(Player),
    MoveUp(Player),
    MoveDown(Player),
    Quit,
}

impl Default for ActionID {
    fn default() -> Self {
        Self::MoveLeft(Player::P1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum CollisionID {
    Paddle(Player),
    Ball(usize),
    InertWall,
    Goal(Player),
}

#[derive(Clone, Copy)]
struct Ball {
    pos: Vec2,
    vel: Vec2,
    id: usize,
}

impl Default for CollisionID {
    fn default() -> Self {
        Self::Paddle(Player::P1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
enum PoolID {
    Points(Player),
}

fn window_conf() -> Conf {
    Conf {
        window_title: "clowder".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

struct Logics {
    control: KeyboardControl<ActionID, MacroquadInputWrapper>,
    physics: PointPhysics,
    collision: AabbCollision<CollisionID>,
    resources: QueuedResources<PoolID, u32>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = KeyboardControl::new();
                control.add_key_map(0, KeyCode::J, ActionID::MoveRight(Player::P1), true);
                control.add_key_map(0, KeyCode::L, ActionID::MoveLeft(Player::P1), true);
                control.add_key_map(0, KeyCode::I, ActionID::MoveUp(Player::P1), true);
                control.add_key_map(0, KeyCode::K, ActionID::MoveDown(Player::P1), true);
                control.add_key_map(0, KeyCode::Escape, ActionID::Quit, true);
                control
            },
            physics: PointPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                //bottom wall
                collision.add_entity_as_xywh(
                    Vec2::new(0.0, (HEIGHT - WALL_DEPTH) as f32),
                    Vec2::new(WIDTH as f32, WALL_DEPTH as f32),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::InertWall,
                );

                //left wall
                collision.add_entity_as_xywh(
                    Vec2::ZERO,
                    Vec2::new(WALL_DEPTH as f32, HEIGHT as f32),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::InertWall,
                );
                //top 1
                collision.add_entity_as_xywh(
                    Vec2::ZERO,
                    Vec2::new(((WIDTH / 2) - (GOAL_WIDTH / 2)) as f32, WALL_DEPTH as f32),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::InertWall,
                );
                //top 2
                collision.add_entity_as_xywh(
                    Vec2::new(((WIDTH / 2) + (GOAL_WIDTH / 2)) as f32, 0.0),
                    Vec2::new(((WIDTH / 2) - (GOAL_WIDTH / 2)) as f32, WALL_DEPTH as f32),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::InertWall,
                );
                //goal
                collision.add_entity_as_xywh(
                    Vec2::ZERO,
                    Vec2::new(WIDTH as f32, 0.0),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::Goal(Player::P1),
                );
                //right wall
                collision.add_entity_as_xywh(
                    Vec2::new((WIDTH - WALL_DEPTH) as f32, 0.0),
                    Vec2::new(WALL_DEPTH as f32, HEIGHT as f32),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::InertWall,
                );
                collision
            },

            resources: {
                let mut resources = QueuedResources::new();
                resources
                    .items
                    .insert(PoolID::Points(Player::P1), (0, 0, u32::MAX));
                resources
            },
        }
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Debug)]
enum Player {
    P1,
}

struct World {
    paddles: (Vec2, Vec2),
    balls: Vec<Ball>,
    score: (u8, u8),
    time: f64,
    interval: f64,
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut animation = SimpleAnim::new("src/clowder_sprite.png", "src/clowder_sprite.json");
    let mut world = World::new();
    let mut logics = Logics::new();

    //sets background color
    animation.set_background_color(BASE_COLOR);

    //adding the background elements to animation i.e. the walls
    //top left wall
    animation.b_elements.push(BackElement::new(
        0.0,
        0.0,
        ((WIDTH / 2) - (GOAL_WIDTH / 2)) as f32,
        WALL_DEPTH as f32,
        FENCE_COLOR,
    ));

    //top right wall
    animation.b_elements.push(BackElement::new(
        ((WIDTH / 2) + (GOAL_WIDTH / 2)) as f32,
        0.0,
        ((WIDTH / 2) - (GOAL_WIDTH / 2)) as f32,
        WALL_DEPTH as f32,
        FENCE_COLOR,
    ));

    //left wall
    animation.b_elements.push(BackElement::new(
        0.0,
        0.0,
        WALL_DEPTH as f32,
        HEIGHT as f32,
        FENCE_COLOR,
    ));

    //right wall
    animation.b_elements.push(BackElement::new(
        (WIDTH - WALL_DEPTH) as f32,
        0.0,
        WALL_DEPTH as f32,
        HEIGHT as f32,
        FENCE_COLOR,
    ));

    //bottom wall
    animation.b_elements.push(BackElement::new(
        0.0,
        (HEIGHT - WALL_DEPTH) as f32,
        WIDTH as f32,
        WALL_DEPTH as f32,
        FENCE_COLOR,
    ));

    //creates and adds ball objects
    for i in 0..BALL_NUM {
        world.balls.push(Ball::new(
            Vec2::new(
                rand::gen_range(BALL_SIZE as f32, (WIDTH - BALL_SIZE) as f32),
                rand::gen_range((BALL_SIZE * 2) as f32, (HEIGHT - BALL_SIZE) as f32),
            ),
            i.into(),
        ));
        //adds ball objecrs to animation, cycles through distinct ball entities in sheet
        animation.objects.push(AnimObject::new(
            animation.sheet.get_entity((i as u32 % DIST_BALL) as usize),
            world.balls[i as usize].pos,
        ));
    }
    //dog animation
    animation.objects.push(AnimObject::new(
        animation.sheet.get_entity(4),
        world.paddles.0,
    ));

    loop {
        if let Ok(cont) = world.update(&mut logics, &mut animation) {
            if !cont {
                break;
            }
        }
        animation.draw();
        next_frame().await;
    }
}

impl World {
    fn new() -> Self {
        Self {
            paddles: (
                Vec2::new((WIDTH / 2 - PADDLE_WIDTH / 2) as f32, PADDLE_OFF_Y as f32),
                Vec2::new(
                    (WIDTH / 2 - PADDLE_WIDTH / 2) as f32,
                    (HEIGHT - PADDLE_OFF_Y - PADDLE_HEIGHT) as f32,
                ),
            ),
            balls: Vec::new(),
            score: (0, 0),
            time: 0.0,
            interval: 1.0,
        }
    }

    fn update(&mut self, logics: &mut Logics, animation: &mut SimpleAnim) -> Result<bool> {
        self.project_control(&mut logics.control);
        logics.control.update(&());
        self.unproject_control(&logics.control, animation);

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics, animation);

        self.project_collision(&mut logics.collision, &mut logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision, animation);

        if logics.control.values[0][4].value != 0.0 {
            return Ok(false);
        }

        for contact in logics.collision.contacts.iter() {
            match (
                logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id,
            ) {
                (CollisionID::Goal(_player), CollisionID::Ball(i))
                | (CollisionID::Ball(i), CollisionID::Goal(_player)) => {
                    if i <= self.balls.len() {
                        animation.objects[self.balls[i].id].visible_false();
                        self.balls.remove(i);

                        logics
                            .resources
                            .transactions
                            .push((PoolID::Points(Player::P1), Transaction::Change(1)));
                    }
                }

                (CollisionID::InertWall, CollisionID::Ball(i))
                | (CollisionID::Ball(i), CollisionID::InertWall) => {
                    let sides_touched = logics.collision.sides_touched(contact.i, contact.j);

                    if self.balls[i].vel.x != 0.0 {
                        if sides_touched.x != 0.0 {
                            self.balls[i].vel.x = sides_touched.x * -1.0;
                        } else if sides_touched.y != 0.0 {
                            self.balls[i].vel.x = sides_touched.y * -1.0;
                        }
                    }

                    if self.balls[i].vel.y == 0.0 {
                        if sides_touched.x != 0.0 {
                            self.balls[i].vel.y = sides_touched.x * -1.0;
                        } else if sides_touched.y != 0.0 {
                            self.balls[i].vel.y = sides_touched.y * -1.0;
                        }
                    } else {
                        if sides_touched.y != 0.0 {
                            self.balls[i].vel.y *= -1.0;
                        }
                        if sides_touched.x != 0.0 {
                            self.balls[i].vel.x *= -1.0;
                        }
                    }
                }

                (CollisionID::Ball(i), CollisionID::Ball(j)) => {
                    let sides_touched = logics.collision.sides_touched(contact.i, contact.j);

                    if (self.balls[i].vel.x, self.balls[i].vel.y) == (0.0, 0.0) {
                        if sides_touched.x != 1.0 || sides_touched.y != 1.0 {
                            self.balls[i].vel = Vec2::new(sides_touched.x, sides_touched.y);
                        }
                    } else if (self.balls[j].vel.x, self.balls[j].vel.y) == (0.0, 0.0) {
                        if sides_touched.x != 1.0 || sides_touched.y != 1.0 {
                            self.balls[j].vel = Vec2::new(sides_touched.x, sides_touched.y);
                        }
                    } else {
                        if sides_touched.x != 0.0 {
                            self.balls[i].vel.x *= -1.0;
                            self.balls[j].vel.x *= -1.0;
                        }
                        if sides_touched.y != 0.0 {
                            self.balls[i].vel.y *= -1.0;
                            self.balls[j].vel.y *= -1.0;
                        }
                    }
                }

                (CollisionID::Ball(i), CollisionID::Paddle(player))
                | (CollisionID::Paddle(player), CollisionID::Ball(i)) => {
                    let sides_touched = logics.collision.sides_touched(contact.i, contact.j);

                    animation.activate_seq(self.balls[i].id, "Scared");

                    if (self.balls[i].vel.x, self.balls[i].vel.y) == (0.0, 0.0) {
                        if sides_touched.x != 1.0 || sides_touched.y != 1.0 {
                            self.balls[i].vel = Vec2::new(sides_touched.x, sides_touched.y);
                        }
                    } else {
                        if sides_touched.y != 0.0 {
                            self.balls[i].vel.y *= -1.0;
                        }
                        if sides_touched.x != 0.0 {
                            self.balls[i].vel.x *= -1.0;
                        }
                    }

                    self.change_angle(player, i);
                }
                _ => {}
            }
        }

        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources);

        for completed in logics.resources.completed.iter() {
            match completed {
                Ok(item_type) => match item_type {
                    PoolID::Points(player) => {
                        match player {
                            Player::P1 => print!("p1"),
                        }
                        println!(" scores! p1: {}", self.score.0);
                    }
                },
                Err(_) => {}
            }
        }
        Ok(true)
    }

    fn project_control(&self, control: &mut KeyboardControl<ActionID, MacroquadInputWrapper>) {
        control.mapping[0][0].is_valid = true;
        control.mapping[0][1].is_valid = true;
        control.mapping[0][2].is_valid = true;
        control.mapping[0][3].is_valid = true;
        control.mapping[0][4].is_valid = true;
    }

    fn unproject_control(
        &mut self,
        control: &KeyboardControl<ActionID, MacroquadInputWrapper>,
        animation: &mut SimpleAnim,
    ) {
        //if any button is being pressed, dog is running so cycle is active
        if control.values[0][0].value > 0.0
            || control.values[0][1].value > 0.0
            || control.values[0][2].value > 0.0
            || control.values[0][3].value > 0.0
        {
            animation.activate_seq(BALL_NUM as usize, "Running");
        } else {
            animation.deactivate_seq(BALL_NUM as usize, "Running");
        }

        //if moving left
        if control.values[0][0].value > 0.0 {
            animation.objects[BALL_NUM as usize].flip_x_false();
        }
        //if moving right
        else if control.values[0][1].value > 0.0 {
            animation.objects[BALL_NUM as usize].flip_x_true();
        }

        self.paddles.0.x = ((self.paddles.0.x - control.values[0][0].value as f32
            + control.values[0][1].value as f32) //confusing, incorporate ActionIds
            .max(0.0) as f32)
            .min((255 - PADDLE_WIDTH) as f32); //drive with data not code
        self.paddles.0.y = ((self.paddles.0.y as f32 - control.values[0][2].value as f32
            + control.values[0][3].value as f32)
            .max(0.0) as f32)
            .min((255 - PADDLE_HEIGHT) as f32);
    }

    fn project_physics(&self, physics: &mut PointPhysics) {
        physics.positions.clear();
        physics.velocities.clear();
        physics.accelerations.clear();
        for ball in self.balls.iter() {
            physics.add_physics_entity(ball.pos, ball.vel, Vec2::ZERO);
        }
    }

    fn unproject_physics(&mut self, physics: &PointPhysics, animation: &mut SimpleAnim) {
        for ((ball, pos), vel) in self
            .balls
            .iter_mut()
            .zip(physics.positions.iter())
            .zip(physics.velocities.iter())
        {
            ball.pos = *pos;
            ball.vel = *vel;
            //not moving
            if ball.vel != Vec2::ZERO {
                animation.activate_seq(ball.id, "Running");
            } else {
                // moving
                animation.deactivate_seq(ball.id, "Running");
            }

            //if moving left
            if ball.vel.x > 0.0 {
                animation.objects[ball.id].flip_x_true();
            }
            //moving right
            else {
                animation.objects[ball.id].flip_x_false();
            }
        }
    }

    fn project_collision(
        &self,
        collision: &mut AabbCollision<CollisionID>,
        control: &mut KeyboardControl<ActionID, MacroquadInputWrapper>,
    ) {
        collision.centers.resize_with(6, Default::default);
        collision.half_sizes.resize_with(6, Default::default);
        collision.velocities.resize_with(6, Default::default);
        collision.metadata.resize_with(6, Default::default);

        collision.add_entity_as_xywh(
            self.paddles.0,
            Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32),
            Vec2::new(
                -control.values[0][0].value + control.values[0][1].value,
                -control.values[0][2].value + control.values[0][3].value,
            ),
            true,
            true,
            CollisionID::Paddle(Player::P1),
        );

        for (i, ball) in self.balls.iter().enumerate() {
            collision.add_entity_as_xywh(
                ball.pos,
                Vec2::splat(BALL_SIZE as f32),
                ball.vel,
                true,
                false,
                CollisionID::Ball(i),
            );
        }
    }

    fn unproject_collision(
        &mut self,
        collision: &AabbCollision<CollisionID>,
        animation: &mut SimpleAnim,
    ) {
        for (i, ball) in self.balls.iter_mut().enumerate() {
            //if object is not paused
            //if !animation.objects[ball.id].paused
            // {

            // }
            ball.pos.x = (collision.centers[i + 7].x - collision.half_sizes[i + 7].x).trunc();
            ball.pos.y = (collision.centers[i + 7].y - collision.half_sizes[i + 7].y).trunc();
            animation.objects[ball.id].pos = ball.pos;

            //sets pivot around center and position
            animation.objects[ball.id].set_pivot(Some(Vec2::new(
                ball.pos.x + (BALL_SIZE / 2) as f32,
                ball.pos.y + (BALL_SIZE / 2) as f32,
            )));

            // animation.objects[ball.id].set_rotation( f32::sqrt(ball.vel.x * ball.vel.x + ball.vel.y * ball.vel.y));
        }

        //sets position and pivot around center
        animation.objects[BALL_NUM as usize].set_pivot(Some(Vec2::new(
            self.paddles.0.x + (PADDLE_WIDTH / 2) as f32,
            self.paddles.0.y + (PADDLE_HEIGHT / 2) as f32,
        )));
        animation.objects[BALL_NUM as usize].pos = self.paddles.0;
    }

    fn change_angle(&mut self, player: Player, ball_index: usize) {
        let ball = &mut self.balls[ball_index];

        let paddle_center = match player {
            Player::P1 => self.paddles.0.x + (PADDLE_WIDTH / 2) as f32,
        } as f32;
        let angle: f32 = ((ball.pos.x + (BALL_SIZE / 2) as f32 - paddle_center)
            .max(-(PADDLE_WIDTH as f32) / 2.0)
            .min(PADDLE_WIDTH as f32 / 2.0)
            / PADDLE_WIDTH as f32)
            .abs()
            * 80.0;
        let magnitude = f32::sqrt(ball.vel.x * ball.vel.x + ball.vel.y * ball.vel.y);
        ball.vel.x =
            angle.to_radians().sin() * magnitude * if ball.vel.x < 0.0 { -1.0 } else { 1.0 };
        ball.vel.y =
            angle.to_radians().cos() * magnitude * if ball.vel.y < 0.0 { -1.0 } else { 1.0 };
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID, u32>) {
        resources
            .items
            .entry(PoolID::Points(Player::P1))
            .or_insert((0, 0, u32::MAX));
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID, u32>) {
        for completed in resources.completed.iter() {
            match completed {
                Ok(item_type) => {
                    let value = resources.get_value_by_itemtype(item_type).unwrap();
                    match item_type {
                        PoolID::Points(player) => match player {
                            Player::P1 => self.score.0 = value as u8,
                        },
                    }
                }
                Err(_) => {}
            }
        }
    }

    /* /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, animation: &mut SimpleAnim) {
        clear_background(BASE_COLOR);

        animation.draw();
        //top 1 (left)
        draw_rectangle(
            0.0,
            0.0,
            ((WIDTH / 2) - (GOAL_WIDTH / 2)) as f32,
            WALL_DEPTH as f32,
            FENCE_COLOR,
        );

        //top 2 (right)
        draw_rectangle(
            ((WIDTH / 2) + (GOAL_WIDTH / 2)) as f32,
            0.0,
            ((WIDTH / 2) - (GOAL_WIDTH / 2)) as f32,
            WALL_DEPTH as f32,
            FENCE_COLOR,
        );

        //left wall
        draw_rectangle(0.0, 0.0, WALL_DEPTH as f32, HEIGHT as f32, FENCE_COLOR);

        //right wall
        draw_rectangle(
            (WIDTH - WALL_DEPTH) as f32,
            0.0,
            WALL_DEPTH as f32,
            HEIGHT as f32,
            FENCE_COLOR,
        );

        //bottom wall
        draw_rectangle(
            0.0,
            (HEIGHT - WALL_DEPTH) as f32,
            WIDTH as f32,
            WALL_DEPTH as f32,
            FENCE_COLOR,
        );

        //increments animation frames
        animation.incr_frames();
    }*/
}

impl Ball {
    fn new(newpos: Vec2, newid: usize) -> Self {
        Self {
            pos: newpos,
            vel: Vec2::ZERO,
            id: newid,
        }
    }
}
