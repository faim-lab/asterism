use asterism::{
    animation::SimpleAnim,
    collision::AabbCollision,
    control::{KeyboardControl, MacroQuadKeyboardControl},
    entity_state::FlatEntityState,
    physics::PointPhysics,
    resources::{PoolInfo, QueuedResources, Transaction},
};
use json::*;
use macroquad::prelude::*;
use std::fs::File;
use std::io::{self, Write};

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const BASKET_OFF: u8 = 200;
const BASKET_WIDTH: u8 = 48;
const BASKET_HEIGHT: u8 = 32;
const APPLE_SIZE: u8 = 24;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum ActionID {
    MoveRight,
    MoveLeft,
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum CollisionID {
    Basket,
    Floor,
    Wall,
    Apple(usize),
}

impl Default for CollisionID {
    fn default() -> Self {
        Self::Basket
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum StateID {
    AppleFalling,
    AppleBouncing,
    AppleResting,
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum PoolID {
    Points,
}

impl PoolInfo for PoolID {
    fn max_value(&self) -> f64 {
        match self {
            Self::Points => std::u32::MAX as f64,
        }
    }

    fn min_value(&self) -> f64 {
        match self {
            Self::Points => std::u32::MIN as f64,
        }
    }
}

struct Apple {
    pos: Vec2,
    vel: Vec2,
    color: Color,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "apple catching".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

struct Logics {
    control: MacroQuadKeyboardControl<ActionID>,
    entity_state: FlatEntityState<StateID>,
    physics: PointPhysics<Vec2>,
    collision: AabbCollision<CollisionID, Vec2>,
    resources: QueuedResources<PoolID>,
}

struct World {
    basket: Vec2,
    basket_vel: Vec2,
    apples: Vec<Apple>,
    score: u32,
    time: f64,
    interval: f64,
}

#[macroquad::main(window_conf)]
async fn main() {
    let file = File::open("src/apple_tree_sprite.json").unwrap();
    let animation = SimpleAnim::new();
    animation
        .load_sprite_sheet("src/apple_tree_sprite.png", file)
        .await;
    let mut world = World::new();
    let mut logics = Logics::new();

    loop {
        if let Ok(cont) = world.update(&mut logics) {
            if !cont {
                break;
            }
        }
        world.draw(&mut animation);
        next_frame().await;
    }
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = MacroQuadKeyboardControl::new();
                control.add_key_map(0, KeyCode::Right, ActionID::MoveRight);
                control.add_key_map(0, KeyCode::Left, ActionID::MoveLeft);
                control.add_key_map(0, KeyCode::Escape, ActionID::Quit);
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
                    CollisionID::Wall,
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
                    CollisionID::Wall,
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
                    CollisionID::Wall,
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
                    CollisionID::Floor,
                );
                collision
            },
            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert(PoolID::Points, 0.0);
                resources
            },
            entity_state: FlatEntityState::new(),
        }
    }
}

impl World {
    fn new() -> Self {
        Self {
            basket: Vec2::new((WIDTH / 2 - BASKET_WIDTH / 2) as f32, BASKET_OFF as f32),
            basket_vel: Vec2::new(0.0, 0.0),
            apples: Vec::new(),
            score: 0,
            time: 0.0,
            interval: 1.0,
        }
    }

    fn update(&mut self, logics: &mut Logics) -> Result<bool> {
        // this should probably go into a temporal matching? or chance logic but i dont want to write it right now
        if get_time() - self.time > self.interval {
            self.apples.push(Apple::new());
            self.time = get_time();
            while self.interval > 0.5 {
                self.interval *= 0.9;
            }
        }

        self.project_control(&mut logics.control);
        logics.control.update(&());
        self.unproject_control(&logics.control);

        // this should probably go in unproject_control, see maze-macroquad for
        // example
        if logics.control.values[0][2].value != 0.0 {
            return Ok(false);
        }

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        self.project_entity_state(&mut logics.entity_state, &logics.collision);
        logics.entity_state.update();
        self.unproject_entity_state(&logics.entity_state);

        for contact in logics.collision.contacts.iter() {
            match (
                logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id,
            ) {
                (CollisionID::Apple(i), CollisionID::Basket) => {
                    if i < self.apples.len()
                        && logics
                            .collision
                            .sides_touched(contact, &CollisionID::Apple(i))
                            .y
                            < 0.0
                    {
                        self.apples.remove(i);
                        logics
                            .resources
                            .transactions
                            .push(vec![(PoolID::Points, Transaction::Change(1.0))]);
                    }
                }
                (CollisionID::Apple(i), CollisionID::Wall) => {
                    if i < self.apples.len() {
                        self.apples.remove(i);
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
                            PoolID::Points => {
                                print!("current score: {}\r", self.score);
                                io::stdout().flush().unwrap();
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
        control.mapping[0][2].is_valid = true;
    }

    fn unproject_control(&mut self, control: &MacroQuadKeyboardControl<ActionID>) {
        self.basket_vel.x = -control.get_action(ActionID::MoveLeft).unwrap().value
            + control.get_action(ActionID::MoveRight).unwrap().value;
    }

    fn project_physics(&self, physics: &mut PointPhysics<Vec2>) {
        physics.positions.clear();
        physics.velocities.clear();
        physics.accelerations.clear();
        for apple in self.apples.iter() {
            physics.add_physics_entity(apple.pos, apple.vel, Vec2::new(0.0, 0.04));
        }
        physics.add_physics_entity(
            self.basket,
            Vec2::new(self.basket_vel.x, 0.0),
            Vec2::new(0.0, 0.0),
        );
    }

    fn unproject_physics(&mut self, physics: &PointPhysics<Vec2>) {
        for ((apple, pos), vel) in self
            .apples
            .iter_mut()
            .zip(physics.positions.iter())
            .zip(physics.velocities.iter())
        // since the basket physics is at the end of the vec, it won't be included in this
        // iteration because .zip() will truncate when it's done iterating through
        // self.apples, which is shorter
        {
            apple.pos = *pos;
            apple.vel = *vel;
        }
        self.basket = physics.positions[self.apples.len()];
        self.basket_vel = physics.velocities[self.apples.len()];
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID, Vec2>) {
        collision.centers.resize_with(4, Default::default);
        collision.half_sizes.resize_with(4, Default::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);

        collision.add_entity_as_xywh(
            self.basket.x,
            self.basket.y,
            BASKET_WIDTH as f32,
            BASKET_HEIGHT as f32,
            self.basket_vel,
            true,
            true,
            CollisionID::Basket,
        );

        for (i, apple) in self.apples.iter().enumerate() {
            collision.add_entity_as_xywh(
                apple.pos.x as f32,
                apple.pos.y as f32,
                APPLE_SIZE as f32,
                APPLE_SIZE as f32,
                apple.vel,
                true,
                false,
                CollisionID::Apple(i),
            );
        }
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID, Vec2>) {
        for (i, apple) in self.apples.iter_mut().enumerate() {
            apple.pos = collision
                .get_xy_pos_for_entity(CollisionID::Apple(i))
                .unwrap();
        }
    }

    fn project_entity_state(
        &self,
        entity_state: &mut FlatEntityState<StateID>,
        collision: &AabbCollision<CollisionID, Vec2>,
    ) {
        entity_state.maps.clear();
        entity_state.conditions.clear();
        entity_state.states.clear();
        for _ in self.apples.iter() {
            entity_state.add_state_map(
                0,
                vec![
                    (StateID::AppleFalling, vec![1, 2]),
                    (StateID::AppleBouncing, vec![0]),
                    (StateID::AppleResting, vec![0]),
                ],
            );
        }

        let mut bounce = Vec::new();
        for contact in collision.contacts.iter() {
            match (
                collision.metadata[contact.i].id,
                collision.metadata[contact.j].id,
            ) {
                (CollisionID::Apple(i), CollisionID::Floor) => {
                    if collision.sides_touched(contact, &CollisionID::Apple(i)).y < 0.0 {
                        if self.apples[i].vel.y < 1.0 {
                            entity_state.conditions[i][2] = true;
                        } else {
                            entity_state.conditions[i][1] = true;
                            bounce.push(i);
                        }
                    }
                }
                (CollisionID::Apple(i), CollisionID::Apple(j)) => {
                    if collision.sides_touched(contact, &CollisionID::Apple(i)).y < 0.0 {
                        if self.apples[i].vel.y < 1.0 {
                            entity_state.conditions[i][2] = true;
                        } else {
                            entity_state.conditions[i][1] = true;
                            bounce.push(i);
                        }
                    }
                    if collision.sides_touched(contact, &CollisionID::Apple(j)).y < 0.0 {
                        if self.apples[j].vel.y < 1.0 {
                            entity_state.conditions[j][2] = true;
                        } else {
                            entity_state.conditions[j][1] = true;
                            bounce.push(j);
                        }
                    }
                }
                _ => {}
            }
        }

        for idx in bounce.iter() {
            entity_state.conditions[*idx][0] = true;
        }
    }

    fn unproject_entity_state(&mut self, entity_state: &FlatEntityState<StateID>) {
        for (i, state) in entity_state.states.iter().enumerate() {
            match entity_state.maps[i].states[*state].id {
                StateID::AppleBouncing => {
                    self.apples[i].vel.y *= -0.6;
                }
                StateID::AppleFalling => {}
                StateID::AppleResting => {
                    self.apples[i].vel.y = 0.0;
                }
            }
        }
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID>) {
        if !resources.items.contains_key(&PoolID::Points) {
            resources.items.insert(PoolID::Points, 0.0);
        }
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID>) {
        for completed in resources.completed.iter() {
            match completed {
                Ok(item_types) => {
                    for item_type in item_types {
                        let value = resources.get_value_by_itemtype(item_type).unwrap();
                        match item_type {
                            PoolID::Points => self.score = value as u32,
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }
    fn draw(&self, animation: &SimpleAnim) {
        clear_background(Color::new(0., 0., 0.5, 1.));
        let mut basket = 2;

        draw_texture_ex(
            animation.sheet.image,
            0.0,
            0.0,
            WHITE,
            animation.sheet.create_param(1),
        );

        if self.score > 15 {
            basket = 5;
        } else if self.score > 10 {
            basket = 4;
        } else if self.score > 5 {
            basket = 3;
        }

        draw_texture_ex(
            animation.sheet.image,
            self.basket.x,
            self.basket.y,
            WHITE,
            animation.sheet.create_param(basket),
        );

        for apple in self.apples.iter() {
            draw_texture_ex(
                animation.sheet.image,
                apple.pos.x,
                apple.pos.y,
                WHITE,
                animation.sheet.create_param(0),
            );
        }
    }
}

impl Apple {
    fn new() -> Self {
        Self {
            pos: Vec2::new(rand::gen_range(0.1, (WIDTH - BASKET_WIDTH) as f32), 0.1),
            vel: Vec2::new(0.0, 0.0),
            color: Color::new(1.0, 0.0, 0.0, 1.0), // draw as rectangle (for debug, since collision bodies are rectangles :P) then differentiate colors: macroquad::color::hsl_to_rgb(0.0, 1.0, rand::gen_range(0.3, 0.7)),
        }
    }
}
