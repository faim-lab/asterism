use asterism::{
    animation::{SimpleAnim, AnimObject},
    collision::AabbCollision,
    control::{KeyboardControl, MacroquadInputWrapper},
    physics::PointPhysics,
    resources::{QueuedResources, Transaction},
    Logic,
};
use json::*;
use macroquad::prelude::*;

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

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
enum PoolID {
    Points,
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
    control: KeyboardControl<ActionID, MacroquadInputWrapper>,
    physics: PointPhysics,
    collision: AabbCollision<CollisionID>,
    resources: QueuedResources<PoolID, u32>,
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
    let mut animation = SimpleAnim::new("src/apple_tree_sprite.png", "src/apple_tree_sprite.json");
    let mut world = World::new();
    let mut logics = Logics::new();
    animation.set_background_color(BLUE);


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

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = KeyboardControl::new();
                control.add_key_map(0, KeyCode::Right, ActionID::MoveRight, true);
                control.add_key_map(0, KeyCode::Left, ActionID::MoveLeft, true);
                control.add_key_map(0, KeyCode::Escape, ActionID::Quit, true);
                control
            },
            physics: PointPhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                // left
                collision.add_entity_as_xywh(
                    Vec2::new(-2.0, 0.0),
                    Vec2::new(2.0, HEIGHT as f32),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::Wall,
                );
                // right
                collision.add_entity_as_xywh(
                    Vec2::new(WIDTH as f32, 0.0),
                    Vec2::new(2.0, HEIGHT as f32),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::Wall,
                );
                // top
                collision.add_entity_as_xywh(
                    Vec2::new(0.0, -2.0),
                    Vec2::new(WIDTH as f32, 2.0),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::Wall,
                );
                // bottom
                collision.add_entity_as_xywh(
                    Vec2::new(0.0, HEIGHT as f32),
                    Vec2::new(WIDTH as f32, 2.0),
                    Vec2::ZERO,
                    true,
                    true,
                    CollisionID::Floor,
                );
                collision
            },
            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert(PoolID::Points, (0,0,u32::MAX));
                resources
            },
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

    fn update(&mut self, logics: &mut Logics, animation: &mut SimpleAnim) -> Result<bool> {
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
        self.unproject_physics(&logics.physics, animation);

        self.project_collision(&mut logics.collision);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        for contact in logics.collision.contacts.iter() {
            match (
                logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id,
            ) {
                (CollisionID::Basket, CollisionID::Apple(i)) => {
                    if i < self.apples.len()
                        && logics
                            .collision
                            .sides_touched(contact.i, contact.j)
                            .y
                            < 0.0
                    {
                        self.apples.remove(i);
                        logics
                            .resources
                            .transactions
                            .push((PoolID::Points, Transaction::Change(1)));
                    }
                }
                (CollisionID::Wall, CollisionID::Apple(i)) => {
                    if i < self.apples.len() {
                        self.apples.remove(i);
                    }
                }
                (CollisionID::Floor, CollisionID::Apple(i)) => {
                    let apple = &mut self.apples[i];
                    if apple.vel.y >= 0.1 {
                        apple.vel.y *= -0.6;
                    }
                }
                (CollisionID::Apple(i), CollisionID::Apple(j)) => {
                    let sides_touched = logics.collision.sides_touched(contact.i, contact.j);
                    let apple_i = &mut self.apples[i];

                    if sides_touched.y > 0.0 && apple_i.vel.y >= 0.1 {
                        apple_i.vel.y *= -0.6;
                    }

                    let apple_j = &mut self.apples[j];
                    if sides_touched.y < 0.0 && apple_j.vel.y >= 0.1 {
                        apple_j.vel.y *= -0.6;
                    }
                }
                _ => {}
            }
        }

        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources, animation);

        for completed in logics.resources.completed.iter() {
            match completed {
                Ok(item_type) => {
                    match item_type {
                        PoolID::Points => {
                            println!("current score: {}\r", self.score);
                        }
                    }
                }
                Err(_) => {}
            }
        }

        Ok(true)
    }

    fn project_control(&self, control: &mut KeyboardControl<ActionID, MacroquadInputWrapper>) {
        control.mapping[0][0].is_valid = true;
        control.mapping[0][1].is_valid = true;
        control.mapping[0][2].is_valid = true;
    }

    fn unproject_control(&mut self, control: &KeyboardControl<ActionID, MacroquadInputWrapper>) {
        self.basket_vel.x = -control.get_action(ActionID::MoveLeft).unwrap().value
            + control.get_action(ActionID::MoveRight).unwrap().value;
    }

    fn project_physics(&self, physics: &mut PointPhysics) {
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

    fn unproject_physics(&mut self, physics: &PointPhysics, animation: &mut SimpleAnim) {
        
        animation.objects.clear();
        animation.objects.push(AnimObject::new(animation.sheet.get_entity(1), Vec2::ZERO));
        animation.objects.push(AnimObject::new(animation.sheet.get_entity(2),self.basket));
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
            animation.objects.push(AnimObject::new(animation.sheet.get_entity(0), apple.pos));
        }
        self.basket = physics.positions[self.apples.len()];
        self.basket_vel = physics.velocities[self.apples.len()];
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID>) {
        collision.centers.resize_with(4, Default::default);
        collision.half_sizes.resize_with(4, Default::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);

        collision.add_entity_as_xywh(
            self.basket,
            Vec2::new(BASKET_WIDTH as f32, BASKET_HEIGHT as f32),
            self.basket_vel,
            true,
            true,
            CollisionID::Basket,
        );

        let apple_size = Vec2::new(APPLE_SIZE as f32, APPLE_SIZE as f32);
        for (i, apple) in self.apples.iter().enumerate() {
            collision.add_entity_as_xywh(
                apple.pos,
                apple_size,
                apple.vel,
                true,
                false,
                CollisionID::Apple(i),
            );
        }
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID>) {
        self.basket = {
            let col = collision.get_ident_data(4);
            col.center - col.half_size
        };
        for (i, apple) in self.apples.iter_mut().enumerate() {
            apple.pos = {
                let col = collision.get_ident_data(i + 5);
                col.center - col.half_size
            }
        }
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID, u32>) {
        resources
            .items
            .entry(PoolID::Points)
            .or_insert((0, 0, u32::MAX));
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID, u32>, animation: &mut SimpleAnim) {
        for completed in resources.completed.iter() {
            match completed {
                Ok(item_type) => {
                    let value = resources.get_value_by_itemtype(item_type).unwrap();
                    match item_type {
                        PoolID::Points => self.score = value,
                    }
                }
                Err(_) => {}
            }
        }

        if self.score > 15
        {
            animation.activate_seq(0, "Full");
        }
        else if self.score > 10
        {
            animation.activate_seq(0, "Almost Full");
        }
        else if self.score > 5 {
            animation.activate_seq(0, "Not Empty");
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
