#![allow(clippy::upper_case_acronyms)]
use asterism::{
    collision::{magnitude, AabbCollision},
    control::{KeyboardControl, MacroquadInputWrapper, Values},
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
    MoveUp,
    MoveDown,
    Serve,
    Quit,
}

impl Default for ActionID {
    fn default() -> Self {
        Self::MoveDown
    }
}

type CollisionID = usize;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
struct PoolID(usize);

struct Logics {
    control: KeyboardControl<ActionID, MacroquadInputWrapper>,
    physics: PointPhysics,
    collision: AabbCollision<CollisionID>,
    resources: QueuedResources<PoolID, u8>, // i hope you don't ever need more than 255 points
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = KeyboardControl::new();
                control.add_key_map(0, KeyCode::Q, ActionID::MoveUp);
                control.add_key_map(0, KeyCode::A, ActionID::MoveDown);
                control.add_key_map(0, KeyCode::W, ActionID::Serve);
                control.add_key_map(1, KeyCode::O, ActionID::MoveUp);
                control.add_key_map(1, KeyCode::L, ActionID::MoveDown);
                control.add_key_map(1, KeyCode::I, ActionID::Serve);
                control.add_key_map(2, KeyCode::Escape, ActionID::Quit);
                control
            },
            physics: PointPhysics::new(),
            collision: AabbCollision::new(),
            resources: QueuedResources::new(),
        }
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Debug)]
#[allow(dead_code)]
enum Player {
    P1,
    P2,
}

struct World {
    positions: Vec<Vec2>,                  // physics
    sizes: Vec<Vec2>,                      // collision
    velocities: Vec<Vec2>,                 // physics/collision
    collision_metadata: Vec<(bool, bool)>, // collision: (solid, fixed)
    mappings: Vec<Vec<bool>>,              // control
    keys_pressed: Vec<Vec<bool>>, // control. this is almost exactly what KeyboardControl.values is though
    resources: Vec<(u8, u8, u8)>, // rsrc
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
        if !world.update(&mut logics) {
            break;
        }
        world.draw();
        next_frame().await;
    }
}

impl World {
    fn new() -> Self {
        let positions = vec![
            Vec2::new(-2.0, 0.0), // four walls
            Vec2::new(WIDTH as f32, 0.0),
            Vec2::new(0.0, -2.0),
            Vec2::new(0.0, HEIGHT as f32),
            Vec2::new(
                (WIDTH - BALL_SIZE) as f32 / 2.0,
                (HEIGHT - BALL_SIZE) as f32 / 2.0,
            ), // ball
            Vec2::new(PADDLE_OFF_X as f32, (HEIGHT - PADDLE_HEIGHT) as f32 / 2.0), // paddle 1
            Vec2::new(
                (WIDTH - PADDLE_OFF_X) as f32,
                (HEIGHT - PADDLE_HEIGHT) as f32 / 2.0,
            ), // paddle 2
        ];

        let sizes = vec![
            Vec2::new(2.0, HEIGHT as f32),
            Vec2::new(2.0, HEIGHT as f32),
            Vec2::new(WIDTH as f32, 2.0),
            Vec2::new(WIDTH as f32, 2.0),
            Vec2::new(BALL_SIZE as f32, BALL_SIZE as f32),
            Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32),
            Vec2::new(PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32),
        ];

        let collision_metadata = vec![
            (true, true),
            (true, true),
            (true, true),
            (true, true),
            (true, false),
            (true, true),
            (true, true),
        ];

        // check that positions, sizes, and metadata lengths match up
        assert_eq!(positions.len(), sizes.len());
        assert_eq!(positions.len(), collision_metadata.len());

        let mappings = vec![vec![true, true, true], vec![true, true, false], vec![true]];

        let velocities = positions.iter().map(|_| Vec2::ZERO).collect();

        let keys_pressed = mappings
            .iter()
            .map(|keyset| keyset.iter().map(|_| false).collect::<Vec<_>>())
            .collect();

        Self {
            positions,
            sizes,
            velocities,
            collision_metadata,
            mappings,
            keys_pressed,
            resources: vec![(0, 0, 255), (0, 0, 255)],
        }
    }

    fn update(&mut self, logics: &mut Logics) -> bool {
        self.project_control(&mut logics.control);
        logics.control.update_input(&());
        self.unproject_control(&logics.control);

        if self.keys_pressed[0][0] {
            self.positions[5].y = (self.positions[5].y - 1.0).max(0.0);
        }
        if self.keys_pressed[0][1] {
            self.positions[5].y = (self.positions[5].y + 1.0).min(HEIGHT as f32 - self.sizes[5].y);
        }
        if self.keys_pressed[0][2] {
            self.velocities[4].x = 1.0;
            self.velocities[4].y = 1.0;
            self.mappings[0][2] = false;
        }
        if self.keys_pressed[1][0] {
            self.positions[6].y = (self.positions[6].y - 1.0).max(0.0);
        }
        if self.keys_pressed[1][1] {
            self.positions[6].y = (self.positions[6].y + 1.0).min(HEIGHT as f32 - self.sizes[6].y);
        }
        if self.keys_pressed[1][2] {
            self.velocities[4].x = -1.0;
            self.velocities[4].y = -1.0;
            self.mappings[1][2] = false;
        }

        if self.keys_pressed[2][0] {
            return false;
        }

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
                // can't believe we're regressing to summer 2020 asterism contacts
                (4, 0) => {
                    self.positions[4] = Vec2::new(
                        (WIDTH / 2 - BALL_SIZE / 2) as f32,
                        (HEIGHT / 2 - BALL_SIZE / 2) as f32,
                    );
                    self.velocities[4] = Vec2::ZERO;
                    // doing this seems so pointless when i can just go self.resources[0] += 1.0 lol. like i know the indirection is the point but there's so much of it
                    logics
                        .resources
                        .transactions
                        .push(vec![(PoolID(1), Transaction::Change(1))]);
                    self.mappings[1][2] = true;
                }
                (4, 1) => {
                    self.positions[4] = Vec2::new(
                        (WIDTH / 2 - BALL_SIZE / 2) as f32,
                        (HEIGHT / 2 - BALL_SIZE / 2) as f32,
                    );
                    self.velocities[4] = Vec2::ZERO;
                    logics
                        .resources
                        .transactions
                        .push(vec![(PoolID(0), Transaction::Change(1))]);
                    self.mappings[0][2] = true;
                }

                (4, 2) | (4, 3) => {
                    self.velocities[4].y *= -1.0;
                }

                (4, 5) => {
                    let sides_touched = logics.collision.sides_touched(contact, &4);
                    if sides_touched.x > 0.0 {
                        self.velocities[4].x *= -1.0;
                    } else {
                        self.velocities[4].y *= -1.0;
                    }
                    // self.change_angle(player);
                    if magnitude(self.velocities[4]) < 4.0 {
                        self.velocities[4] *= 1.1;
                    }
                }
                (4, 6) => {
                    let sides_touched = logics.collision.sides_touched(contact, &4);
                    if sides_touched.x < 0.0 {
                        self.velocities[4].x *= -1.0;
                    } else {
                        self.velocities[4].y *= -1.0;
                    }
                    // self.change_angle(player);
                    if magnitude(self.velocities[4]) < 4.0 {
                        self.velocities[4] *= 1.1;
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
                        println!(
                            "p{} scores! p1: {}, p2: {}",
                            item_type.0 + 1,
                            self.resources[0].0,
                            self.resources[1].0
                        );
                    }
                }
                Err(_) => {}
            }
        }

        true
    }

    fn project_control(&self, control: &mut KeyboardControl<ActionID, MacroquadInputWrapper>) {
        for (logic_set, set) in control.mapping.iter_mut().zip(self.mappings.iter()) {
            for (logic_action, act_valid) in logic_set.iter_mut().zip(set.iter()) {
                logic_action.is_valid = *act_valid;
            }
        }
    }

    fn unproject_control(&mut self, control: &KeyboardControl<ActionID, MacroquadInputWrapper>) {
        for (logic_set, set) in control.values.iter().zip(self.keys_pressed.iter_mut()) {
            for (Values { value, .. }, pressed) in logic_set.iter().zip(set.iter_mut()) {
                *pressed = *value > 0.0;
            }
        }
    }

    fn project_physics(&self, physics: &mut PointPhysics) {
        physics.clear();
        physics.accelerations.clear();
        for (pos, vel) in self.positions.iter().zip(self.velocities.iter()) {
            physics.add_physics_entity(*pos, *vel, Vec2::ZERO);
        }
    }

    fn unproject_physics(&mut self, physics: &PointPhysics) {
        // only have to unproject positions bc all accelerations above are zero
        for (logic_pos, pos) in physics.positions.iter().zip(self.positions.iter_mut()) {
            *pos = *logic_pos;
        }
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID>) {
        collision.clear();
        // this iterator stuff is mildly horrifying
        for (i, (((pos, size), vel), (solid, fixed))) in self
            .positions
            .iter()
            .zip(self.sizes.iter())
            .zip(self.velocities.iter())
            .zip(self.collision_metadata.iter())
            .enumerate()
        {
            collision.add_entity_as_xywh(*pos, *size, *vel, *solid, *fixed, i);
        }
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID>) {
        // honestly i regret the centers/half sizes thing lol
        // it was 90% because of bevy and i think i assumed that macroquad did a similar thing with drawing
        // which it super doesn't. i should take it out/move some 181g stuff over
        for ((centers, half_sizes), pos) in
            (collision.centers.iter().zip(collision.half_sizes.iter()))
                .zip(self.positions.iter_mut())
        {
            *pos = *centers - *half_sizes;
        }
    }

    /* not yet
     *
     * fn change_angle(&mut self, player: Player) {
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
    } */

    fn project_resources(&self, resources: &mut QueuedResources<PoolID, u8>) {
        for (i, res) in self.resources.iter().enumerate() {
            resources.items.insert(PoolID(i), *res);
        }
    }

    // "i hope this works!"
    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID, u8>) {
        for (i, res) in self.resources.iter_mut().enumerate() {
            *res = *resources.items.get(&PoolID(i)).unwrap();
        }
    }

    fn draw(&self) {
        clear_background(Color::new(0., 0., 0.5, 1.));
        for (pos, size) in self.positions.iter().zip(self.sizes.iter()) {
            draw_rectangle(pos.x, pos.y, size.x, size.y, WHITE);
        }
    }
}
