#![deny(clippy::all)]
#![forbid(unsafe_code)]

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use ultraviolet::{Vec2, geometry::Aabb};
use asterism::{AabbCollision, QueuedResources, resources::Transaction, Linking};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 20;
const ITEM_SIZE: i8 = 10;
const PORTAL_SIZE: i8 = 14;

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    Still,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CollisionID {
    Player,
    Wall,
    Item,
    Portal,
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum PoolID {
    Points,
}

impl Default for CollisionID {
    fn default() -> Self { Self::Player }
}

struct World {
    x: i16,
    y: i16,
    vx: i16,
    vy: i16,
    score: u8,
    walls: Vec<Wall>,
    items: Vec<Collectible>,
    portals: Vec<Portal>,
    touching_portal: bool,
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
        Wall {x: x, y: y, w: w, h: h}
    }
}

struct Collectible {
    x: i16,
    y: i16,
}

impl Collectible {
    fn new(x:i16, y:i16) -> Self {
        Self {x: x, y: y}
    }
}

struct Portal {
    x: i16,
    y: i16,
    color: [u8; 4],
}

impl Portal {
    fn new(x:i16, y:i16, color:[u8; 4]) -> Self {
        Self {x: x, y: y, color: color}
    }
}

struct MazePhysics {
    pos: Vec2,
    vel: Vec2,
}

impl MazePhysics {
    fn new() -> Self {
        Self {
            pos: Vec2::new(0.0, 0.0),
            vel: Vec2::new(0.0, 0.0), 
        }
    }

    fn update(&mut self) {
        for i in 0..2 {
            self.pos[i] += self.vel[i];
        }
    }
}

struct Logics {
    physics: MazePhysics,
    collision: AabbCollision<CollisionID>,
    linking: Linking,
    resources: QueuedResources<PoolID>,
}

impl Logics {
    fn new(walls: &Vec<Wall>, portals: &Vec<Portal>) -> Self {
        Self {
            physics: MazePhysics::new(),
            collision: {
                let mut collision = AabbCollision::new();
                for wall in walls {
                    collision.add_collision_entity(
                        wall.x as f32, wall.y as f32,
                        wall.w as f32, wall.h as f32,
                        Vec2::new(0.0, 0.0),
                        true, true, CollisionID::Wall);
                }
                for portal in portals {
                    collision.add_collision_entity(portal.x as f32, portal.y as f32,
                        PORTAL_SIZE as f32, PORTAL_SIZE as f32,
                        Vec2::new(0.0, 0.0),
                        false, true, CollisionID::Portal);
                }
                collision
            },
            linking: {
                let mut linking = Linking::new();
                linking.add_link_map(2, vec![vec![1, 2], vec![0, 2], vec![0, 1]]);
                linking
            },
            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert( PoolID::Points, 0.0 );
                resources
            },
        }
    }
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("maze")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut hidpi_factor = window.scale_factor();
    
    let mut pixels = {
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, surface);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut world = World::new();
    let mut logics = Logics::new(&world.walls, &world.portals);

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame(), &world.walls, &world.items);
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
        if input.update(event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Adjust high DPI factor
            if let Some(factor) = input.scale_factor_changed() {
                hidpi_factor = factor;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            // Arrow key input from user
            let movement = ({
                let up = input.key_held(VirtualKeyCode::Up);
                let down = input.key_held(VirtualKeyCode::Down);

                if up {
                    Direction::Up 
                } else if down {
                    Direction::Down 
                } else {
                    Direction::Still 
                }

            }, {
                let left = input.key_held(VirtualKeyCode::Left);
                let right = input.key_held(VirtualKeyCode::Right);

                if left {
                    Direction::Left
                } else if right {
                    Direction::Right
                } else {
                    Direction::Still
                }
            });
            
            // Update internal state and request a redraw
            world.update(&mut logics, movement);
            window.request_redraw();
        }     
    });
}

impl World {
    /// Create a new `World` instance that can draw walls, portals, items, and player
    fn new() -> Self {
        Self {
            x: 58,
            y: 8,
            vx: 0,
            vy: 0,
            score: 0,
            walls: {
                vec![
                    // horizontal walls
                    Wall::new(8, 11, 43, 3),
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
                    Portal::new(110, 70, [0x4f, 0xe5, 0xff, 0xff]), // blue portal 0
                    Portal::new(280, 27, [0xfc, 0x8c, 0x03, 0xff]), // orange portal 1
                ]
            },
            touching_portal: false,
            just_teleported: false,
        }
    }

    /// Update the `World` internal state 
    fn update(&mut self, logics: &mut Logics, movement: ( Direction, Direction )) {
        // mapping of keyboard controls to velocities
        match movement.0 {
            Direction::Up => self.vy = -4,
            Direction::Down => self.vy = 4,
            _ => self.vy = 0,
        }
        match movement.1 {
            Direction::Left => self.vx = -4,
            Direction::Right => self.vx = 4,
            _ => self.vx = 0,
        }

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        // check if player is still touching portal after teleporting (and possibly moving a bit)
        self.touching_portal = false;

        self.project_collision(&mut logics.collision, &logics.physics);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        for contact in logics.collision.contacts.iter() {
            match (logics.collision.metadata[contact.0].id,
                logics.collision.metadata[contact.1].id) {
                    (CollisionID::Portal, CollisionID::Player) => {
                        if self.just_teleported {
                            self.touching_portal = true;
                        }
                    }
                    (CollisionID::Item, CollisionID::Player) => {
                    // add to score and remove touched item from game state
                    logics.resources.transactions.push(vec![(PoolID::Points, Transaction::Change(1))]);
                    self.items.remove(contact.0 - self.walls.len() - self.portals.len());
                }
                _ => {}
            }
        }
        
        if !self.touching_portal {
            self.just_teleported = false;
        }

        self.project_linking(&mut logics.linking, &logics.collision);
        logics.linking.update();
        self.unproject_linking(&logics.linking);

        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources);

        for (completed, item_types) in logics.resources.completed.iter() {
            if *completed {
                for item_type in item_types {
                    match item_type {
                        PoolID::Points => {
                            println!("You scored! Current points: {}", self.score);
                        }
                    }
                }
            }
        }
    }

    fn project_physics(&self, physics: &mut MazePhysics) {
        physics.pos.x = self.x as f32;
        physics.pos.y = self.y as f32;
        physics.vel.x = self.vx as f32;
        physics.vel.y = self.vy as f32;
    }

    fn unproject_physics(&mut self, physics: &MazePhysics) { 
        self.x = physics.pos[0].trunc() as i16;
        self.y = physics.pos[1].trunc() as i16;
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID>, physics: &MazePhysics) {
        collision.bodies.resize_with(self.walls.len() + self.portals.len(), Aabb::default);
        collision.velocities.resize_with(self.walls.len() + self.portals.len(), Default::default);
        collision.metadata.resize_with(self.walls.len() + self.portals.len(), Default::default);
        // create collider for items
        for item in &self.items {
            collision.add_collision_entity(item.x as f32, item.y as f32,
                ITEM_SIZE as f32, ITEM_SIZE as f32,
                Vec2::new(0.0, 0.0),
                false, true, CollisionID::Item);
        }
        // create collider for player
        collision.add_collision_entity(self.x as f32, self.y as f32,
            BOX_SIZE as f32, BOX_SIZE as f32,
            physics.vel,
            true, false, CollisionID::Player);
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID>) {
        self.x = collision.bodies[collision.bodies.len() - 1].min.x.trunc() as i16;
        self.y = collision.bodies[collision.bodies.len() - 1].min.y.trunc() as i16;
    }

    // node 0: teleport to orange; node 1: teleport to blue; node 2: no teleporting
    fn project_linking(&self, linking: &mut Linking, collision: &AabbCollision<CollisionID>) {
        let mut teleport:Option<usize> = None;
        for contact in collision.contacts.iter() {
            match (collision.metadata[contact.0].id,
                collision.metadata[contact.1].id) {
                (CollisionID::Portal, CollisionID::Player) => {
                    if !self.just_teleported && !self.touching_portal {
                        teleport = Some(contact.0 - self.walls.len());
                    } else {
                        teleport = Some(2);
                    }
                }
                _ => {}
            }
        }
        
        if let Some(choice) = teleport {
            let next_pos = choice;
            linking.conditions[0][next_pos] = true;
        }
    }
    
    fn unproject_linking(&mut self, linking: &Linking) {
        let mut updated:Option<usize> = None;
        for (.., pos) in linking.maps.iter().zip(linking.positions.iter()) {
            updated = Some(*pos);
        }

        if let Some(new_pos) = updated {
            match new_pos {
                0 => {
                    // teleport to orange portal
                    self.x = 277;
                    self.y = 24;
                    self.just_teleported = true;
                }
                1 => {
                    // teleport to blue portal
                    self.x = 107;
                    self.y = 67;
                    self.just_teleported = true;
                }
                _ => {}
            }
        }
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID>) {
        if !resources.items.contains_key(&PoolID::Points) {
            resources.items.insert(PoolID::Points, 0.0);
        }
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID>) {
        for (completed, item_types) in resources.completed.iter() {
            if *completed {
                for item_type in item_types {
                    let value = resources.get_value_by_itemtype(item_type).min(255.0) as u8;
                    match item_type {
                        PoolID::Points => self.score = value,
                    }
                }
            }
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8], walls: &Vec<Wall>, items: &Vec<Collectible>) {
        fn inside_all_walls(x:i16, y:i16, walls: &Vec<Wall>) -> bool {
            for a_wall in walls {
                if x >= a_wall.x
                && x < a_wall.x + a_wall.w
                && y >= a_wall.y
                && y < a_wall.y + a_wall.h {
                    return true;
                }
            } 
            return false;
        }

        let is_box = |x, y| -> bool {
            if x >= self.x
            && x < self.x + BOX_SIZE
            && y >= self.y
            && y < self.y + BOX_SIZE {
                return true;
            } else {
                return false;
            }
        };

        fn is_item(x:i16, y:i16, items: &Vec<Collectible>) -> bool {
            for an_item in items.iter() {
                if x >= an_item.x
                && x < an_item.x + ITEM_SIZE as i16
                && y >= an_item.y
                && y < an_item.y + ITEM_SIZE as i16 {
                    return true;
                }
            }
            return false;
        }

        let is_portal_1 = |x, y| -> bool {
            if x >= self.portals[0].x
            && x < self.portals[0].x + PORTAL_SIZE as i16
            && y >= self.portals[0].y
            && y < self.portals[0].y + PORTAL_SIZE as i16 {
                return true;
            } else {
                return false;
            }
        };

        let is_portal_2 = |x, y| -> bool {
            if x >= self.portals[1].x
            && x < self.portals[1].x + PORTAL_SIZE as i16
            && y >= self.portals[1].y
            && y < self.portals[1].y + PORTAL_SIZE as i16 {
                return true;
            } else {
                return false;
            }
        };

        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0x48, 0xb2, 0xe8, 0xff]);
        }

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let rgba = if inside_all_walls(x, y, walls) {
                [0xff, 0xff, 0xff, 0xff]
            } else if is_box(x, y) {
                [0x5e, 0x48, 0xe8, 0xff]
            } else if is_item(x, y, items) {
                [0x95, 0xed, 0xc1, 0xff]
            } else if is_portal_1(x, y) {
                self.portals[0].color
            } else if is_portal_2(x, y) {
                self.portals[1].color
            } else {
                continue;
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
