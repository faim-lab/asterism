#![deny(clippy::all)]
#![forbid(unsafe_code)]

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use ultraviolet::{Vec2, Vec3, geometry::Aabb};
const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const PADDLE_OFF_X: u8 = 16;
const PADDLE_HEIGHT: u8 = 48;
const PADDLE_WIDTH: u8 = 8;
const BALL_SIZE: u8 = 8;

struct WinitControl {
    _actors: Vec<Player>,
    // loci: Vec<>,
    selected_actions: Vec<(Player, Action)>,
    // keymapping
}

impl WinitControl {
    fn new() -> Self {
        Self {
            _actors: vec![Player::P1, Player::P2],
            selected_actions: Vec::new(),
        }
    }

// handles keyboard inputs
    fn update(&mut self, input:&WinitInputHelper) {
        self.selected_actions.clear();
        if input.key_held(VirtualKeyCode::Q) {
            self.selected_actions.push((Player::P1, Action::Move(-1)));
        } else if input.key_held(VirtualKeyCode::A) {
            self.selected_actions.push((Player::P1, Action::Move(1)));
        }
        if input.key_held(VirtualKeyCode::W) {
            self.selected_actions.push((Player::P1, Action::Serve));
        }
        if input.key_held(VirtualKeyCode::O) {
            self.selected_actions.push((Player::P2, Action::Move(-1)));
        } else if input.key_held(VirtualKeyCode::L) {
            self.selected_actions.push((Player::P2, Action::Move(1)));
        }
        if input.key_held(VirtualKeyCode::I) {
            self.selected_actions.push((Player::P2, Action::Serve));
        }
    }
}

struct PongPhysics {
    // "structure of arrays"
    positions:Vec<Vec2>,
    velocities:Vec<Vec2>
}
impl PongPhysics {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            velocities: Vec::new(),
        }
    }

    fn update(&mut self) {
        for (pos, vel) in self.positions.iter_mut().zip(self.velocities.iter()) {
            *pos += *vel;
        }
    }
}

// velocity shifts bodies out of whatever they interpenetrated before
// moving
struct PongCollision {
    bodies: Vec<Aabb>,
    velocities: Vec<Vec2>,
    contacts: Vec<(usize, usize)>,
}

impl PongCollision {
    fn new() -> Self {
        Self {
            bodies: Vec::new(),
            velocities: Vec::new(),
            contacts: Vec::new(),
        }
    }

    fn update(&mut self) {
        self.contacts.clear();
        for (i, body) in self.bodies.iter().enumerate() {
            for (j, body2) in self.bodies.iter().enumerate() {
                if i != j && body.intersects(body2) {
                    self.contacts.push((i, j));
                }
            }
        }

        for (i, ..) in self.contacts.iter() {
            let body = &mut self.bodies[*i];
            let vel = &mut self.velocities[*i];
            body.min.x -= vel.x;
            body.min.y -= vel.y;
            body.max.x -= vel.x;
            body.max.y -= vel.y;
        }
    }
}


struct Logics {
    control: WinitControl,
    physics: PongPhysics,
    collision: PongCollision,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: WinitControl::new(),
            physics: PongPhysics::new(),
            collision: PongCollision::new(),
        }
    }
}

enum Player {
    P1,
    P2
}
enum Action {
    Move(i8),
    Serve
}
struct World {
    paddles: (u8, u8),
    ball: (u8, u8),
    ball_err: Vec2,
    ball_vel: Vec2,
    serving: Option<Player>
}


fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("paddles")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut hidpi_factor = window.scale_factor();

    let mut pixels = {
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(WIDTH as u32, HEIGHT as u32, surface);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };
    let mut world = World::new();
    let mut logics = Logics::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
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

            // Update internal state and request a redraw
            world.update(&mut logics, &input);
            window.request_redraw();
        }
    });
}

impl World {
    fn new() -> Self {
        Self {
            paddles: (HEIGHT/2-PADDLE_HEIGHT/2, HEIGHT/2-PADDLE_HEIGHT/2),
            ball: (WIDTH/2-BALL_SIZE/2, HEIGHT/2-BALL_SIZE/2),
            ball_err: Vec2::new(0.0,0.0),
            ball_vel: Vec2::new(0.0,0.0),
            serving: Some(Player::P1)
        }
    }

    fn update(&mut self, logics:&mut Logics, input:&WinitInputHelper) {
        logics.control.update(input);
        for choice in logics.control.selected_actions.iter() {
            match choice {
                (Player::P1, Action::Move(amt)) => {
                    if *amt < 0 {
                        self.paddles.0 -= amt.abs() as u8;
                    } else {
                        self.paddles.0 += *amt as u8;
                    }
                },
                (Player::P1, Action::Serve) => {
                    if let Some(Player::P1) = self.serving {
                        self.ball_vel = Vec2::new(3.0, 3.0);
                        self.serving = None;
                    }
                },
                (Player::P2, Action::Move(amt)) => {
                    if *amt < 0 {
                        self.paddles.1 -= amt.abs() as u8;
                    } else {
                        self.paddles.1 += *amt as u8;
                    }
                },
                (Player::P2, Action::Serve) => {
                    if let Some(Player::P2) = self.serving {
                        self.ball_vel = Vec2::new(-3.0, -3.0);
                        self.serving = None;
                    }
                }
            }
        }

        //project game state to collision volumes (ball, paddles, walls)
        //update collision
        //unproject to game state (ball velocity and position)
        //now, go through the contacts and perform game specific actions
        // - if the ball touched left or right side of screen, reset to serving
        // - if the ball touched a paddle or top or bottom of screen, reflect it normal to the collision surface and increase its speed slightly


        //projection and unprojection
        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        // game specific collision stuff
        for contact in logics.collision.contacts.iter() {
            match (contact.0, contact.1) {
                // collisions: wall x wall = nothing.
                // paddle x wall = stop (nothing)
                // ball x top, bottom wall = bounce (x-axis)
                // ball x paddle = bounce (y-axis)
                // operate on contact.0, compare w/ contact.1
                (4, 0) | (4, 1) => {
                    self.ball_vel = Vec2::new(0.0, 0.0);
                    self.ball = (WIDTH / 2 - BALL_SIZE / 2, HEIGHT / 2 - BALL_SIZE / 2);
                    if contact.1 == 0 {
                        self.serving = Some(Player::P2);
                        println!("p2 scores");
                    } else {
                        self.serving = Some(Player::P1);
                        println!("p1 scores");
                    }
                },
                (4, 2) | (4, 3) => {
                    self.ball_vel.y *= -1.0;
                    self.ball.1 = (self.ball.1 as f32 + self.ball_vel.y.trunc()) as u8;
                },
                (4, 5) | (4, 6) => {
                    self.ball_vel.x *= -1.0;
                    self.ball.0 = (self.ball.0 as f32 + self.ball_vel.x.trunc()) as u8;

                },
                _ => {}
            }
        }
    }
    fn project_physics(&self, physics:&mut PongPhysics) {
        physics.positions.resize_with(1, Vec2::default);
        physics.velocities.resize_with(1, Vec2::default);
        physics.positions[0].x = self.ball.0 as f32 + self.ball_err.x;
        physics.positions[0].y = self.ball.1 as f32 + self.ball_err.y;
        physics.velocities[0] = self.ball_vel;
    }

    fn unproject_physics(&mut self, physics:&PongPhysics) {
        self.ball.0 = physics.positions[0].x.trunc() as u8;
        self.ball.1 = physics.positions[0].y.trunc() as u8;
        self.ball_err = physics.positions[0] - Vec2::new(self.ball.0 as f32, self.ball.1 as f32);
        self.ball_vel = physics.velocities[0];
    }

    fn project_collision(&self, collision: &mut PongCollision, control: &WinitControl) {
        collision.bodies.resize_with(7, Aabb::default);
        collision.velocities.resize_with(7, Default::default);
        let colliders = [
            Aabb::new(
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(0.0, HEIGHT as f32, 0.0)),
            // right
            Aabb::new(
                Vec3::new(WIDTH as f32, 0.0, 0.0),
                Vec3::new(WIDTH as f32 + 1.0, HEIGHT as f32, 0.0)),
            // top
            Aabb::new(
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(WIDTH as f32, 0.0, 0.0)),
            // bottom
            Aabb::new(
                Vec3::new(0.0, HEIGHT as f32, 0.0),
                Vec3::new(WIDTH as f32, HEIGHT as f32 + 1.0, 0.0)),
            // ball
            Aabb::new(
                Vec3::new(self.ball.0 as f32, self.ball.1 as f32, 0.0),
                Vec3::new(self.ball.0 as f32 + BALL_SIZE as f32, self.ball.1 as f32 + BALL_SIZE as f32, 0.0)),
            // paddles
            Aabb::new(
                Vec3::new((PADDLE_OFF_X + PADDLE_WIDTH - 1) as f32, self.paddles.0 as f32, 0.0),
                Vec3::new((PADDLE_OFF_X + PADDLE_WIDTH) as f32, self.paddles.0 as f32 + PADDLE_HEIGHT as f32, 0.0)),
            Aabb::new(
                Vec3::new((WIDTH - PADDLE_OFF_X - PADDLE_WIDTH) as f32, self.paddles.1 as f32, 0.0),
                Vec3::new((WIDTH - PADDLE_OFF_X - PADDLE_WIDTH + 1) as f32, self.paddles.1 as f32 + PADDLE_HEIGHT as f32, 0.0))];
        for (i, body) in colliders.iter().enumerate() {
            collision.bodies[i] = *body;
        }
        for vel in collision.velocities[..4].iter_mut() {
            *vel = Vec2::new(0.0, 0.0);
        }
        collision.velocities[4] = self.ball_vel;
        let mut p1_vel: f32 = 0.0;
        let mut p2_vel: f32 = 0.0;
        for choice in &control.selected_actions {
            match choice {
                (Player::P1, Action::Move(amt)) => {
                    p1_vel = *amt as f32;
                }
                (Player::P2, Action::Move(amt)) => {
                    p2_vel = *amt as f32;
                }
                _ => {}
            }
        }
        collision.velocities[5] = Vec2::new(0.0, p1_vel);
        collision.velocities[6] = Vec2::new(0.0, p2_vel);
    }

    fn unproject_collision(&mut self, collision: &PongCollision) {
        self.ball.0 = collision.bodies[4].min.x.trunc() as u8;
        self.ball.1 = collision.bodies[4].min.y.trunc() as u8;
        self.paddles.0 = collision.bodies[5].min.y.trunc() as u8;
        self.paddles.1 = collision.bodies[6].min.y.trunc() as u8;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0,0,128,255]);
        }
        draw_rect(PADDLE_OFF_X, self.paddles.0,
                  PADDLE_WIDTH, PADDLE_HEIGHT,
                  [255,255,255,255],
                  frame);
        draw_rect(WIDTH-PADDLE_OFF_X-PADDLE_WIDTH, self.paddles.1,
                  PADDLE_WIDTH, PADDLE_HEIGHT,
                  [255,255,255,255],
                  frame);
        draw_rect(self.ball.0, self.ball.1,
                  BALL_SIZE, BALL_SIZE,
                  [255,255,255,255],
                  frame);
    }
}
fn draw_rect(x:u8, y:u8, w:u8, h:u8, color:[u8;4], frame:&mut [u8]) {
    let x = x.min(WIDTH-1) as usize;
    let w = (w as usize).min(WIDTH as usize-x);
    let y = y.min(HEIGHT-1) as usize;
    let h = (h as usize).min(HEIGHT as usize-y);
    for row in 0..h {
        let row_start = (WIDTH as usize)*4*(y+row);
        let slice = &mut frame[(row_start+x*4)..(row_start+(x+w)*4)];
        for pixel in slice.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }
}
