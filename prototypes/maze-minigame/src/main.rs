#![deny(clippy::all)]
#![forbid(unsafe_code)]

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use ultraviolet::{Vec2, geometry::Aabb};
use asterism::AabbCollision;// , QueuedResources, WinitKeyboardControl, PointPhysics};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 20;

// Size and point worth of items
const ITEM_SIZE: i8 = 10;
const ITEM_VAL: u8 = 1;
// Size of portals
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

impl Default for CollisionID {
    fn default() -> Self { Self::Player }
}


/// Representation of the application state
struct World {
    x: i16,
    y: i16,
    vx: i16,
    vy: i16,
    score: u8,
    walls: Vec<Wall>,
    items: Vec<Collectible>,
    portals: Vec<Portal>,
}

/// Walls of the maze
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

/// Items that can be obtained and added to score
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
    // todo: would like to add color
}

impl Portal {
    fn new(x:i16, y:i16) -> Self {
        Self {x: x, y: y}
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

    // possibly add keyboard input as a parameter
    fn update(&mut self) {
        for i in 0..2 {
            self.pos[i] += self.vel[i];
        }
    }
}

// thanks paddles
struct MazeResources {
    score: u8,
    score_change: u8,
    touched_item: Option<usize>,
    // can add other things later like keys, food
}

impl MazeResources {
    fn new() -> Self {
        Self {
            score: 0,
            score_change: 0,
            touched_item: None,
        }
    }

    fn update(&mut self, all_items: &mut Vec<Collectible>, walls: &Vec<Wall>) {
        self.score += self.score_change;
        if self.touched_item != None {
            all_items.remove(self.touched_item.unwrap() - walls.len());
        };
        // reset
        self.score_change = 0;
        self.touched_item = None;
    }
}

struct Logics {
    physics: MazePhysics,
    collision: AabbCollision<CollisionID>,
    resources: MazeResources,
}

impl Logics {
    fn new(walls: &Vec<Wall>) -> Self {
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
                collision
            },
            resources: MazeResources::new(),
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
    let mut logics = Logics::new(&world.walls);

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
    /// Create a new `World` instance that can draw walls, items, and player
    fn new() -> Self {
        Self {
            x: 58,
            y: 208,
            vx: 16,
            vy: 16,
            score: 0,
            walls: {
                vec![
                    // create horizontal walls
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
                    // create vertical walls
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
                    Collectible::new(112, 72),
                    Collectible::new(26, 198),
                    Collectible::new(195, 198),
                    Collectible::new(195, 29),
                    Collectible::new(281, 198),
                ]
            },
            portals: {
                vec![
                    Portal::new(153, 27),
                ]
            }
        }
    }

    /// Update the `World` internal state 
    fn update(&mut self, logics: &mut Logics, movement: ( Direction, Direction )) {
        // eventually get rid of this
        // won't tackle control logics for now so probably have to pass `movement` into the physics OL

        // temporary mapping of keyboard controls to velocities
        match movement.0 {
            Direction::Up => self.vy = -10,
            Direction::Down => self.vy = 10,
            _ => self.vy = 0,
        }
        match movement.1 {
            Direction::Left => self.vx = -10,
            Direction::Right => self.vx = 10,
            _ => self.vx = 0,
        }

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &logics.physics);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        for contact in logics.collision.contacts.iter() {
            match (logics.collision.metadata[contact.0].id,
                logics.collision.metadata[contact.1].id) {
                (CollisionID::Item, CollisionID::Player) => {
                    logics.resources.score_change = ITEM_VAL;
                    logics.resources.touched_item = Some(contact.0);
                    // not sure if there's a better way to place the print statement
                    println!("score: {}", self.score + ITEM_VAL);
                },
                _ => {}
            }
        }
        
        self.project_resources(&mut logics.resources);
        logics.resources.update(&mut self.items, &self.walls);
        self.unproject_resources(&logics.resources);
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
        collision.bodies.resize_with(self.walls.len(), Aabb::default);
        collision.velocities.resize_with(self.walls.len(), Default::default);
        collision.metadata.resize_with(self.walls.len(), Default::default);
        
        // create collider for each item
        for item in &self.items {
            collision.add_collision_entity(item.x as f32, item.y as f32,
                ITEM_SIZE as f32, ITEM_SIZE as f32,
                Vec2::new(0.0, 0.0),
                false, true, CollisionID::Item);
        }
        // create collider for portals
        for portal in &self.portals {
            collision.add_collision_entity(portal.x as f32, portal.y as f32,
                PORTAL_SIZE as f32, PORTAL_SIZE as f32,
                Vec2::new(0.0, 0.0),
                false, true, CollisionID::Portal);
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

    fn project_resources(&self, resources:&mut MazeResources) {
        resources.score = self.score;
    }

    fn unproject_resources(&mut self, resources: &MazeResources) {
        self.score = resources.score;
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

        let is_portal = |x, y| -> bool {
            for a_portal in self.portals.iter() {
                if x >= a_portal.x
                && x < a_portal.x + PORTAL_SIZE as i16
                && y >= a_portal.y
                && y < a_portal.y + PORTAL_SIZE as i16 {
                    return true;
                }
            }
            return false;
        };

        // draw background first
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
            } else if is_portal(x, y) {
                [0xfc, 0x8c, 0x03, 0xff]
            } else {
                continue;
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
