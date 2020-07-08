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
    contacts: Vec<(usize, usize)>,
    displacements: Vec<Option<Vec3>>,
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
            displacements: Vec::new(),
        }
    }

    fn update(&mut self) {
        self.contacts.clear();
        for (i, body) in self.bodies.iter().enumerate() {
            for (j, body2) in self.bodies[i + 1..].iter().enumerate() {
                if body.intersects(body2) {
                    self.contacts.push((i, j + i + 1));
                }
            }
        }

        for (i, j) in self.contacts.iter() {
            let CollisionData { solid: i_solid, fixed: i_fixed, .. } =
                self.metadata[*i];
            let CollisionData { solid: j_solid, fixed: j_fixed, .. } =
                self.metadata[*j];

            if !(i_solid && j_solid) || i_fixed && j_fixed {
                continue;
            }

            if !i_fixed && !j_fixed {
                let Vec2 { x: vel_i_x, y: vel_i_y } = self.velocities[*i];
                let Vec2 { x: vel_j_x, y: vel_j_y } = self.velocities[*j];
                let Aabb { min: Vec3 { x: min_i_x, y: min_i_y, .. },
                    max: Vec3 { x: max_i_x, y: max_i_y, ..} } = self.bodies[*i];
                let Aabb { min: Vec3 { x: min_j_x, y: min_j_y, .. },
                    max: Vec3 { x: max_j_x, y: max_j_y, ..} } = self.bodies[*j];

                let ( i_displace, j_displace ) = {
                    let vel_i_x = vel_i_x / (vel_i_x.abs() + vel_j_x.abs());
                    let vel_i_y = vel_i_y / (vel_i_y.abs() + vel_j_y.abs());
                    let vel_j_x = vel_j_x / (vel_i_x.abs() + vel_j_x.abs());
                    let vel_j_y = vel_j_y / (vel_i_y.abs() + vel_j_y.abs());

                    let displacement_x = Self::get_displacement(min_i_x, max_i_x, min_j_x, max_j_x);
                    let displacement_y = Self::get_displacement(min_i_y, max_i_y, min_j_y, max_j_y);

                    ( Vec3::new(displacement_x * vel_i_x, displacement_y * vel_i_y, 0.0),
                        Vec3::new(displacement_x * vel_j_x, displacement_y * vel_j_y, 0.0) )
                };

                self.bodies[*i].min += i_displace;
                self.bodies[*i].max += i_displace;
                self.bodies[*j].min += j_displace;
                self.bodies[*j].max += j_displace;
            } else { // if only one of the objects is fixed
                let i_swap = if !j_fixed {j} else {i};
                let j_swap = if !j_fixed {i} else {j}; // i_swap is unfixed, j_swap is fixed object

                let Aabb { min: Vec3 { x: min_i_x, y: min_i_y, .. },
                max: Vec3 { x: max_i_x, y: max_i_y, ..} } = self.bodies[*i_swap];
                let Aabb { min: Vec3 { x: min_j_x, y: min_j_y, .. },
                max: Vec3 { x: max_j_x, y: max_j_y, ..} } = self.bodies[*j_swap];

                let half_isize_x = (min_i_x + max_i_x) / 2.0;
                let half_isize_y = (min_i_y + max_i_y) / 2.0;
                let half_jsize_x = (min_j_x + max_j_x) / 2.0;
                let half_jsize_y = (min_j_y + max_j_y) / 2.0;

                let i_center = Self::find_center(self.bodies[*i_swap]);
                let j_center = Self::find_center(self.bodies[*j_swap]);

                // eventually have to have more vectors in self.velocities, and use self.velocities[*i_swap].x...
                let overlapped_before_x = {
                    let old_x_center = i_center.x - self.velocities[0].x;
                    (old_x_center - j_center.x).abs() < half_isize_x + half_jsize_x
                };

                let overlapped_before_y = {
                    let old_y_center = i_center.y - self.velocities[0].y;
                    (old_y_center - j_center.y).abs() < half_isize_y + half_jsize_y
                };
                
                // todo: use correct index when more velocities are added
                let displace = {
                    // overlapped horizontally
                    if !overlapped_before_x && overlapped_before_y {
                        if self.velocities[0].x < 0.0 {  // player collided from the right
                            Vec3::new(max_j_x - min_i_x, 0.0, 0.0)
                        } else {  // player collided from the left
                            Vec3::new(min_j_x - max_i_x, 0.0, 0.0)
                        }
                    } else if overlapped_before_x && !overlapped_before_y {  // overlapped vertically
                        if self.velocities[0].y < 0.0 {  // player collided from bottom
                            Vec3::new(0.0, max_j_y - min_i_y, 0.0)
                        } else {  // player collided from top
                            Vec3::new(0.0, min_j_y - max_i_y, 0.0)
                        }
                    } else {  // overlapped diagonally
                        if self.velocities[0].x < 0.0 && self.velocities[0].y < 0.0 {
                            Vec3::new(max_j_x - min_i_x, max_j_y - min_i_y, 0.0)
                        } else if self.velocities[0].x < 0.0 && self.velocities[0].y > 0.0 {
                            Vec3::new(max_j_x - min_i_x, min_j_y - max_i_y, 0.0)
                        } else if self.velocities[0].x > 0.0 && self.velocities[0].y < 0.0 {
                            Vec3::new(min_j_x - max_i_x, max_j_y - min_i_y, 0.0) 
                        } else {
                            Vec3::new(min_j_x - max_i_x, min_j_y - max_i_y, 0.0)
                        }
                    }
                };

                match self.displacements[*i_swap] {
                    None => {
                        self.displacements[*i_swap] = Some(displace);
                    },
                    _ => {
                        if displace.x < self.displacements[*i_swap].unwrap().x  {
                            self.displacements[*i_swap].unwrap().x = displace.x;
                        }
                        if displace.y < self.displacements[*i_swap].unwrap().y {
                            self.displacements[*i_swap].unwrap().y = displace.y;
                        }
                    }
                }
                
                /* let i_swap = if !j_fixed { j } else { i };
                let j_swap = if !j_fixed { i } else { j };
                let Aabb { min: Vec3 { x: min_i_x, y: min_i_y, .. },
                    max: Vec3 { x: max_i_x, y: max_i_y, ..} } = self.bodies[*i_swap];
                let Aabb { min: Vec3 { x: min_j_x, y: min_j_y, .. },
                    max: Vec3 { x: max_j_x, y: max_j_y, ..} } = self.bodies[*j_swap];
                let displace = {
                    let displacement_x = Self::get_displacement(min_i_x, max_i_x, min_j_x, max_j_x);
                    let displacement_y = Self::get_displacement(min_i_y, max_i_y, min_j_y, max_j_y);

                    if displacement_x == displacement_y {
                        Vec3::new(displacement_x, displacement_y, 0.0)
                    } else if displacement_x < displacement_y {
                        if min_i_x < min_j_x {
                            Vec3::new(-displacement_x, 0.0, 0.0)
                        } else {
                            Vec3::new(displacement_x, 0.0, 0.0)
                        }
                    } else {
                        if min_i_y < min_j_y {
                            Vec3::new(0.0, -displacement_y, 0.0)
                        } else {
                            Vec3::new(0.0, displacement_y, 0.0)
                        }
                    }
                };

                self.bodies[*i_swap].min += displace;
                self.bodies[*i_swap].max += displace; */
            }
        }
        
        // can condense later but for now temporary fix since i_swap can't be reached
        for i in 0..self.displacements.len() {
            match self.displacements[i] {
                None => {
                    continue;
                }
                _ => {
                    self.bodies[i].min += self.displacements[i].unwrap();
                    self.bodies[i].max += self.displacements[i].unwrap();
                }
            }
        }
    }


    fn find_center(body_data: Aabb) -> Vec2 {
        Vec2::new(
            (body_data.min.x + body_data.max.x) / 2.0,
            (body_data.min.y + body_data.max.y) / 2.0
        )
    }

    fn get_displacement(min_i: f32, max_i: f32, min_j: f32, max_j: f32)
        -> f32 {
            if max_i - min_j < max_j - min_i {
                max_i - min_j
            } else {
                max_j - min_i
            }
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
                    collision.displacements.push(None);
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
        // self.move_box(&movement);

        // temporary mapping of keyboard controls to velocities
        match movement.0 {
            Direction::Up => self.vy = -8,
            Direction::Down => self.vy = 8,
            _ => self.vy = 0,
        }
        match movement.1 {
            Direction::Left => self.vx = -8,
            Direction::Right => self.vx = 8,
            _ => self.vx = 0,
        }

        // remove comments when done
        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &logics.physics);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

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
            collision.displacements.push(None);
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
        collision.displacements.push(None);
        
        // project into physics logic to get position and velocity? 
        collision.velocities.push(physics.vel);
    }

    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID>) {
        // same question - pos[0] or pos.x?
        self.x = collision.bodies[collision.bodies.len() - 1].min.x.trunc() as i16;
        self.y = collision.bodies[collision.bodies.len() - 1].min.y.trunc() as i16;
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
