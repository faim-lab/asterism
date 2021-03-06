#![allow(clippy::upper_case_acronyms)]
use asterism::{
    collision::AabbCollision,
    control::{KeyboardControl, MacroquadInputWrapper},
    linking::GraphedLinking,
    resources::{QueuedResources, Transaction},
};
use macroquad::prelude::*;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 20;
const ITEM_SIZE: i8 = 10;
const PORTAL_SIZE: i8 = 14;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CollisionID {
    Player,
    Wall,
    Item,
    Portal(usize, usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Debug)]
enum PoolID {
    Points,
}

struct World {
    x: f32,
    y: f32,
    score: u8,
    walls: Vec<Wall>,
    items: Vec<Collectible>,
    portals: Vec<Portal>,
    just_teleported: bool,
}

struct Wall {
    x: i16,
    y: i16,
    w: i16,
    h: i16,
}

impl Wall {
    fn new(x: i16, y: i16, w: i16, h: i16) -> Wall {
        Wall { x, y, w, h }
    }
}

struct Collectible {
    x: i16,
    y: i16,
}

impl Collectible {
    fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

struct Portal {
    x: i16,
    y: i16,
    to: usize,
    color: Color,
}

impl Portal {
    fn new(x: i16, y: i16, to: usize, color: Color) -> Self {
        Self { x, y, to, color }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum ActionID {
    Move(Direction),
    Quit,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Logics {
    control: KeyboardControl<ActionID, MacroquadInputWrapper>,
    collision: AabbCollision<CollisionID>,
    linking: GraphedLinking,
    resources: QueuedResources<PoolID, u8>,
}

impl Logics {
    fn new(walls: &Vec<Wall>, portals: &Vec<Portal>) -> Self {
        Self {
            control: {
                let mut control = KeyboardControl::new();
                control.add_key_map(0, KeyCode::Up, ActionID::Move(Direction::Up), true);
                control.add_key_map(0, KeyCode::Down, ActionID::Move(Direction::Down), true);
                control.add_key_map(0, KeyCode::Right, ActionID::Move(Direction::Right), true);
                control.add_key_map(0, KeyCode::Left, ActionID::Move(Direction::Left), true);
                control.add_key_map(0, KeyCode::Escape, ActionID::Quit, true);
                control
            },
            collision: {
                let mut collision = AabbCollision::new();
                for wall in walls.iter() {
                    collision.add_entity_as_xywh(
                        Vec2::new(wall.x as f32, wall.y as f32),
                        Vec2::new(wall.w as f32, wall.h as f32),
                        Vec2::ZERO,
                        true,
                        true,
                        CollisionID::Wall,
                    );
                }
                let portal_size = Vec2::new(PORTAL_SIZE as f32, PORTAL_SIZE as f32);
                for (i, portal) in portals.iter().enumerate() {
                    collision.add_entity_as_xywh(
                        Vec2::new(portal.x as f32, portal.y as f32),
                        portal_size,
                        Vec2::ZERO,
                        false,
                        true,
                        CollisionID::Portal(portal.to, i),
                    );
                }
                collision
            },
            linking: GraphedLinking::new(),
            resources: {
                let mut resources = QueuedResources::new();
                resources
                    .items
                    .insert(PoolID::Points, (0, u8::MIN, u8::MAX));
                resources
            },
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = World::new();
    let mut logics = Logics::new(&world.walls, &world.portals);

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
        Self {
            x: 110.0,
            y: 100.0,
            score: 0,
            walls: {
                vec![
                    // horizontal walls
                    Wall::new(8, 11, 43, 3),
                    // test wall for flipped sign of displacement
                    Wall::new(47, 14, 3, 3),
                    Wall::new(94, 11, 218, 3),
                    Wall::new(94, 54, 46, 3),
                    Wall::new(180, 54, 86, 3),
                    Wall::new(223, 97, 43, 3),
                    Wall::new(8, 140, 46, 3),
                    Wall::new(266, 140, 46, 3),
                    Wall::new(51, 183, 132, 3),
                    Wall::new(223, 183, 43, 3),
                    Wall::new(8, 226, 218, 3),
                    Wall::new(266, 226, 46, 3),
                    // vertical walls
                    Wall::new(8, 11, 3, 218),
                    Wall::new(51, 54, 3, 89),
                    Wall::new(94, 54, 3, 132),
                    Wall::new(137, 54, 3, 89),
                    Wall::new(180, 11, 3, 175),
                    Wall::new(223, 97, 3, 132),
                    Wall::new(309, 11, 3, 218),
                    // borders
                    Wall::new(-1, -1, 322, 1),
                    Wall::new(-1, 240, 322, 1),
                    Wall::new(-1, -1, 1, 242),
                    Wall::new(320, -1, 1, 242),
                ]
            },
            items: {
                vec![
                    Collectible::new(154, 29),
                    Collectible::new(26, 198),
                    Collectible::new(195, 198),
                    Collectible::new(281, 198),
                ]
            },
            portals: {
                vec![
                    Portal::new(110, 70, 1, BLUE),   // blue portal 0
                    Portal::new(280, 27, 0, ORANGE), // orange portal 1
                ]
            },
            just_teleported: false,
        }
    }

    fn update(&mut self, logics: &mut Logics) -> bool {
        self.project_control(&mut logics.control);
        logics.control.update(&());
        match self.unproject_control(&logics.control) {
            Ok(_) => {}
            Err(_) => return false,
        }

        self.project_collision(&mut logics.collision, &logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        let mut touching_portal = false;

        for contact in logics.collision.contacts.iter() {
            match (
                logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id,
            ) {
                (CollisionID::Portal(..), CollisionID::Player)
                | (CollisionID::Player, CollisionID::Portal(..)) => touching_portal = true,
                (CollisionID::Item, CollisionID::Player)
                | (CollisionID::Player, CollisionID::Item) => {
                    // add to score and remove touched item from game state
                    logics
                        .resources
                        .transactions
                        .push((PoolID::Points, Transaction::Change(1)));
                    self.items
                        .remove(contact.i - self.walls.len() - self.portals.len() - 1);
                    dbg!(contact.j);
                }
                _ => {}
            }
        }

        if !touching_portal {
            self.just_teleported = false;
        }

        self.project_linking(&mut logics.linking, &logics.collision);
        logics.linking.update();
        self.unproject_linking(&logics.linking);

        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources);

        for completed in logics.resources.completed.iter() {
            match completed {
                Ok(item_type) => match item_type {
                    PoolID::Points => {
                        println!("You scored! Current points: {}", self.score);
                    }
                },
                Err(_) => {}
            }
        }
        true
    }

    fn project_control(&self, control: &mut KeyboardControl<ActionID, MacroquadInputWrapper>) {
        for map in control.mapping[0].iter_mut() {
            map.is_valid = true;
        }
    }

    fn unproject_control(
        &mut self,
        control: &KeyboardControl<ActionID, MacroquadInputWrapper>,
    ) -> Result<(), ()> {
        self.x += control
            .get_action(ActionID::Move(Direction::Right))
            .unwrap()
            .value
            - control
                .get_action(ActionID::Move(Direction::Left))
                .unwrap()
                .value;
        self.y += control
            .get_action(ActionID::Move(Direction::Down))
            .unwrap()
            .value
            - control
                .get_action(ActionID::Move(Direction::Up))
                .unwrap()
                .value;

        if control.get_action(ActionID::Quit).unwrap().value != 0.0 {
            Err(())
        } else {
            Ok(())
        }
    }

    fn project_collision(
        &self,
        collision: &mut AabbCollision<CollisionID>,
        control: &KeyboardControl<ActionID, MacroquadInputWrapper>,
    ) {
        collision
            .centers
            .resize_with(self.walls.len() + self.portals.len(), || Vec2::ZERO);
        collision
            .half_sizes
            .resize_with(self.walls.len() + self.portals.len(), || Vec2::ZERO);
        collision
            .velocities
            .resize_with(self.walls.len() + self.portals.len(), || Vec2::ZERO);
        collision
            .metadata
            .resize_with(self.walls.len() + self.portals.len(), || {
                asterism::collision::CollisionData {
                    solid: false,
                    fixed: false,
                    id: CollisionID::Player,
                }
            });

        // create collider for items

        let item_size = Vec2::new(ITEM_SIZE as f32, ITEM_SIZE as f32);
        for item in &self.items {
            collision.add_entity_as_xywh(
                Vec2::new(item.x as f32, item.y as f32),
                item_size,
                Vec2::ZERO,
                false,
                true,
                CollisionID::Item,
            );
        }

        // create collider for player
        collision.add_entity_as_xywh(
            Vec2::new(self.x as f32, self.y as f32),
            Vec2::new(BOX_SIZE as f32, BOX_SIZE as f32),
            Vec2::new(
                control
                    .get_action(ActionID::Move(Direction::Right))
                    .unwrap()
                    .value
                    + control
                        .get_action(ActionID::Move(Direction::Left))
                        .unwrap()
                        .value,
                control
                    .get_action(ActionID::Move(Direction::Up))
                    .unwrap()
                    .value
                    + control
                        .get_action(ActionID::Move(Direction::Down))
                        .unwrap()
                        .value,
            ),
            true,
            false,
            CollisionID::Player,
        );
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID>) {
        let player_idx = self.items.len() + self.portals.len() + self.walls.len();
        let pos = collision.centers[player_idx] - collision.half_sizes[player_idx];
        self.x = pos.x;
        self.y = pos.y;
    }

    // node 0: teleport to orange; node 1: teleport to blue; node 2: no teleporting
    fn project_linking(
        &self,
        linking: &mut GraphedLinking,
        collision: &AabbCollision<CollisionID>,
    ) {
        let mut touched_portal = false;
        for contact in collision.contacts.iter() {
            match (
                collision.metadata[contact.i].id,
                collision.metadata[contact.j].id,
            ) {
                (CollisionID::Portal(to, from), CollisionID::Player)
                | (CollisionID::Player, CollisionID::Portal(to, from)) => {
                    if !self.just_teleported {
                        touched_portal = true;
                        linking.add_link_map(from, {
                            let mut map = Vec::new();
                            map.push(vec![1]);
                            map.push(vec![0]);
                            map
                        });
                        linking.conditions[0][to] = true;
                        linking.positions[0] = from;
                    }
                }
                _ => {}
            }
        }
        if !touched_portal {
            linking.maps.clear();
            linking.conditions.clear();
            linking.positions.clear();
        }
    }

    fn unproject_linking(&mut self, linking: &GraphedLinking) {
        for (_, position) in linking.maps.iter().zip(linking.positions.iter()) {
            match position {
                1 => {
                    // teleport to orange portal
                    self.x = 277.0;
                    self.y = 24.0;
                }
                0 => {
                    // teleport to blue portal
                    self.x = 107.0;
                    self.y = 67.0;
                }
                _ => {}
            }
            self.just_teleported = true;
        }
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID, u8>) {
        if !resources.items.contains_key(&PoolID::Points) {
            resources
                .items
                .insert(PoolID::Points, (0, u8::MIN, u8::MAX));
        }
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID, u8>) {
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
    }

    fn draw(&self) {
        clear_background(SKYBLUE);

        for wall in self.walls.iter() {
            draw_rectangle(
                wall.x as f32,
                wall.y as f32,
                wall.w as f32,
                wall.h as f32,
                WHITE,
            );
        }

        for item in self.items.iter() {
            draw_rectangle(
                item.x as f32,
                item.y as f32,
                ITEM_SIZE as f32,
                ITEM_SIZE as f32,
                GREEN,
            );
        }

        for portal in self.portals.iter() {
            draw_rectangle(
                portal.x as f32,
                portal.y as f32,
                PORTAL_SIZE as f32,
                PORTAL_SIZE as f32,
                portal.color,
            );
        }

        draw_rectangle(
            self.x as f32,
            self.y as f32,
            BOX_SIZE as f32,
            BOX_SIZE as f32,
            PURPLE,
        );
    }
}
