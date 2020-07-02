#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![allow(dead_code)]

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use ultraviolet::{Vec2, Vec3, geometry::Aabb};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 20;

/// Size and point worth of items
const ITEM_SIZE: i8 = 10;
const ITEM_VAL: u8 = 1;

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    Still,
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
struct AabbCollision<ID: Copy + Eq> {
    bodies: Vec<Aabb>,
    velocities: Vec<Vec2>,
    metadata: Vec<CollisionData<ID>>,
    contacts: Vec<(usize, usize)>
}

#[derive(Default, Clone, Copy)]
struct CollisionData<ID: Copy + Eq> {
    solid: bool, // true = participates in restitution, false = no
    fixed: bool, // collision system cannot move it
    id: ID,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CollisionID {
    Player,
    Wall,
    Item,
}

impl Default for CollisionID {
    fn default() -> Self { Self::Player }
}

impl AabbCollision<CollisionID> {
    fn new() -> Self {
        Self {
            bodies: Vec::new(),
            velocities: Vec::new(),
            metadata: Vec::new(),
            contacts: Vec::new(),
        }
    }

    fn update(&mut self) {
        // todo: add cases for collision....
    }
}


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

    fn update(&mut self, all_items: &mut Vec<Collectible>) {
        // process queue 
        // destroy pickup if touched?
        self.score += self.score_change;
        if self.touched_item != None {
            all_items.remove(self.touched_item.unwrap());
        };
    }
}

struct Logics {
    // physics: MazePhysics,
    collision: AabbCollision<CollisionID>,
    resources: MazeResources,
}

impl Logics {
    fn new(walls: &Vec<Wall>) -> Self {
        Self {
            collision: {
                let mut collision = AabbCollision::new();
                // create collider for each wall
                for wall in walls {
                    collision.bodies.push(Aabb::new(
                        Vec3::new(wall.x as f32, wall.y as f32, 0.0),
                        Vec3::new((wall.x + wall.w) as f32, (wall.y + wall.h) as f32, 0.0)
                    ));
                    collision.metadata.push(CollisionData {
                        solid: true, 
                        fixed: true, 
                        id: CollisionID::Wall,
                    });
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
    /// Create a new `World` instance that can draw a moving box
    fn new() -> Self {
        Self {
            x: 58,
            y: 8,
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
            }
        }
    }

    /// Update the `World` internal state 
    fn update(&mut self, logics: &mut Logics, movement: ( Direction, Direction )) {
        // eventually get rid of this
        // won't tackle control logics for now so probably have to pass `movement` into the physics OL
        self.move_box(&movement);

        // remove comments when done
        // self.project_physics(&mut logics.physics);
        // logics.physics.update(maybe put movement here?);
        // self.unproject_physics(&logics.physics);

        // self.project_collision(&mut logics.collision, &logics.physics);
        // logics.collision.update();
        // self.unproject_collision(&logics.collision);

        self.project_resources(&mut logics.resources);
        logics.resources.update(&mut self.items);
        self.unproject_resources(&logics.resources);
        if logics.resources.score_change != 0 {
            println!("score: {}", self.score);
        }

    }

    fn project_physics(&self, physics: &mut MazePhysics) {
        physics.pos.x = self.x as f32;
        physics.pos.y = self.y as f32;
        physics.vel.x = self.vx as f32;
        physics.vel.y = self.vy as f32;
    }

    fn unproject_physics(&mut self, physics: &MazePhysics) {
        // is it physics.pos.x or physics.pos[0]? 
        self.x = physics.pos[0].trunc() as i16;
        self.y = physics.pos[1].trunc() as i16;
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID>, physics: &MazePhysics) {
        collision.bodies.resize_with(self.walls.len(), Aabb::default);
        collision.velocities.resize_with(self.walls.len(), Default::default);
        collision.metadata.resize_with(self.walls.len(), CollisionData::default);

        // create collider for each item
        for item in &self.items {
            collision.bodies.push(Aabb::new(
                Vec3::new(item.x as f32, item.y as f32, 0.0),
                Vec3::new((item.x + ITEM_SIZE as i16) as f32, (item.y + ITEM_SIZE as i16) as f32, 0.0)
            ));
            collision.metadata.push(CollisionData {
                solid: false, 
                fixed: true, 
                id: CollisionID::Item,
            });
        }
        // create collider for player
        collision.bodies.push(Aabb::new(
            Vec3::new(self.x as f32, self.y as f32, 0.0),
            Vec3::new((self.x + BOX_SIZE) as f32, (self.y + BOX_SIZE) as f32, 0.0)
        ));
        collision.metadata.push(CollisionData {
            solid: true,
            fixed: false,
            id: CollisionID::Player,
        });
        
        // project into physics logic to get position and velocity? 
        collision.velocities.push(physics.vel);
    }

    fn unproject_collision(&mut self) {
        // same question - pos[0] or pos.x?
        // self.x = collision.pos[0].trunc() as i16,
        // self.y = collision.pos[1].trunc() as i16,
    }

    fn project_resources(&self, resources:&mut MazeResources) {
        resources.score = self.score;
        let i = self.touch_pickup();
        match i {
            None => {
                resources.score_change = 0;
                resources.touched_item = None;
            },
            _ => {
                resources.score_change = ITEM_VAL;
                resources.touched_item = i;
            }
        }
    }

    fn unproject_resources(&mut self, resources: &MazeResources) {
        self.score = resources.score;
    }

    /// Move box according to arrow keys
    fn move_box(&mut self, movement: &(Direction, Direction)) {
        match movement.0 {
            Direction::Up => self.vy = -16,
            Direction::Down => self.vy = 16,
            _ => self.vy = 0,
        }
        match movement.1 {
            Direction::Left => self.vx = -16,
            Direction::Right => self.vx = 16,
            _ => self.vx = 0,
        }

        World::better_collision(self, &movement);

        self.y += self.vy;
        self.x += self.vx;
    }   

    /// Check if box is already touching from above
    fn touching_hz_above(&self, walls: &Vec<Wall>) -> bool {
        for a_wall in walls {
            if a_wall.x + a_wall.w > self.x
            && a_wall.x < self.x + BOX_SIZE {
                if a_wall.y + a_wall.h == self.y {
                    return true;
                }
            }
        }
        return false;
    }

    /// Check if box is already touching from below
    fn touching_hz_below(&self, walls: &Vec<Wall>) -> bool {
        for a_wall in walls {
            if a_wall.x + a_wall.w > self.x
            && a_wall.x < self.x + BOX_SIZE {
                if a_wall.y == self.y + BOX_SIZE {
                    return true;
                }
            }
        }
        return false;
    }
        
    /// Check if box is already touching from the left
    fn touching_vt_left(&self, walls: &Vec<Wall>) -> bool {
        for a_wall in walls {
            if a_wall.y < self.y + BOX_SIZE
            && a_wall.y + a_wall.h > self.y {
                if a_wall.x + a_wall.w == self.x {
                    return true;
                }
            }
        }
        return false;
    }

    /// Check if box is already touching from the right
    fn touching_vt_right(&self, walls: &Vec<Wall>) -> bool {
        for a_wall in walls {
            if a_wall.y < self.y + BOX_SIZE
            && a_wall.y + a_wall.h > self.y {
                if a_wall.x == self.x + BOX_SIZE {
                    return true;
                }
            }
        }
        return false;
    }

    fn corner_above_check(&self, walls: &Vec<Wall>) -> bool {
        for a_wall in walls {
            if a_wall.x + a_wall.w >= self.x
            && a_wall.x <= self.x + BOX_SIZE {
                if a_wall.y + a_wall.h == self.y {
                    return true;
                }
            }
        }
        return false;
    }

    fn corner_below_check(&self, walls: &Vec<Wall>) -> bool {
        for a_wall in walls {
            if a_wall.x + a_wall.w >= self.x
            && a_wall.x <= self.x + BOX_SIZE {
                if a_wall.y == self.y + BOX_SIZE {
                    return true;
                }
            }
        }
        return false;
    }

    fn corner_left_check(&self, walls: &Vec<Wall>) -> bool {
        for a_wall in walls {
            if a_wall.y <= self.y + BOX_SIZE
            && a_wall.y + a_wall.h >= self.y {
                if a_wall.x + a_wall.w == self.x {
                    return true;
                }
            }
        }
        return false;
    }

    fn corner_right_check(&self, walls:&Vec<Wall>) -> bool {
        for a_wall in walls {
            if a_wall.y <= self.y + BOX_SIZE
            && a_wall.y + a_wall.h >= self.y {
                if a_wall.x == self.x + BOX_SIZE {
                    return true;
                }
            }
        }
        return false;
    }

    /// Detect collision
    fn better_collision(&mut self, movement: &(Direction, Direction)) {
        let mut temp_vy: i16 = self.vy;
        let mut temp_vx: i16 = self.vx;

        let touching_above: bool = self.touching_hz_above(&self.walls);
        let touching_below: bool = self.touching_hz_below(&self.walls);
        let touching_left: bool = self.touching_vt_left(&self.walls);
        let touching_right: bool = self.touching_vt_right(&self.walls);

        if {touching_above && movement.0 == Direction::Up} || {touching_below && movement.0 == Direction::Down} {
            temp_vy = 0;
        }
        if {touching_left && movement.1 == Direction::Left} || {touching_right && movement.1 == Direction::Right} {
            temp_vx = 0;
        }

        // Don't move if two arrow keys are pressed in the direction of a corner that the box is perfectly touching
        if movement.0 != Direction::Still && movement.1 != Direction::Still 
        && touching_above == false && touching_below == false 
        && touching_left == false && touching_right == false {
            let touch_vt: bool;
            let touch_hz: bool;

            if movement.0 == Direction::Up {
                touch_vt = self.corner_above_check(&self.walls);
            } else {
                touch_vt = self.corner_below_check(&self.walls);
            }
            if movement.1 == Direction::Left {
                touch_hz = self.corner_left_check(&self.walls);
            } else {
                touch_hz = self.corner_right_check(&self.walls);
            }

            if touch_vt == true && touch_hz == true {
                temp_vy = 0;
                temp_vx = 0;
            }
        }

        let mut temp_y: i16 = self.y + temp_vy;
        let mut temp_x: i16 = self.x + temp_vx;

        if movement.0 != Direction::Still && temp_vy != 0 {
            for a_wall in &self.walls {
                if movement.0 == Direction::Up {
                    if a_wall.y + a_wall.h < self.y
                    && a_wall.y + a_wall.h > temp_y
                    && a_wall.x + a_wall.w > temp_x
                    && a_wall.x < temp_x + BOX_SIZE {
                        if i16::abs(a_wall.y + a_wall.h - self.y) < i16::abs(temp_vy) {
                            temp_vy = a_wall.y + a_wall.h - self.y;
                        }
                    }
                } else {
                    if a_wall.y > self.y + BOX_SIZE
                    && a_wall.y < temp_y + BOX_SIZE 
                    && a_wall.x < temp_x + BOX_SIZE
                    && a_wall.x + a_wall.w > temp_x {
                        if a_wall.y - self.y - BOX_SIZE < temp_vy {
                            temp_vy = a_wall.y - self.y - BOX_SIZE;
                        }
                    }
                }
            }
        }
        if movement.1 != Direction::Still && temp_vx != 0 {
            for a_wall in &self.walls {
                if movement.1 == Direction::Left {
                    if a_wall.x + a_wall.w < self.x
                    && a_wall.x + a_wall.w > temp_x
                    && a_wall.y < temp_y + BOX_SIZE
                    && a_wall.y + a_wall.h > temp_y {
                        if i16::abs(a_wall.x + a_wall.w - self.x) < i16::abs(temp_vx) {
                            temp_vx = a_wall.x + a_wall.w - self.x;
                        }
                    }
                } else {
                    if a_wall.x >= self.x + BOX_SIZE
                    && a_wall.x < temp_x + BOX_SIZE
                    && a_wall.y < temp_y + BOX_SIZE
                    && a_wall.y + a_wall.h > temp_y {
                        if a_wall.x - self.x - BOX_SIZE < temp_vx {
                            temp_vx = a_wall.x - self.x - BOX_SIZE;
                        }
                    } 
                }
            }
        }

        self.vx = temp_vx;
        self.vy = temp_vy;
    }

    /// Check if box is touching or overlapping a pickup - only can check for one at a time, not multiple
    fn touch_pickup(&self) -> Option<usize> {
        for i in 0..self.items.len() {
            if self.x < self.items[i].x + ITEM_SIZE as i16
            && self.x + BOX_SIZE >= self.items[i].x
            && self.y < self.items[i].y + ITEM_SIZE as i16
            && self.y + BOX_SIZE >= self.items[i].y {
                if i < self.items.len() {
                    return Some(i);
                }
            }
        }
        None
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
            } else {
                continue;
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
