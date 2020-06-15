#![deny(clippy::all)]
#![forbid(unsafe_code)]

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use ultraviolet::{Vec2};
const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const PADDLE_OFF_X: u8 = 16;
const PADDLE_HEIGHT: u8 = 48;
const PADDLE_WIDTH: u8 = 8;
const BALL_SIZE: u8 = 8;

struct WinitControl {
    actors: Vec<Player>,
    // loci: Vec<>,
    selected_actions: Vec<(Player, Action)>,
    // keymapping
}
impl WinitControl {
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
    fn update(&mut self) {
        for (pos, vel) in self.positions.iter_mut().zip(self.velocities.iter()) {
            *pos += *vel;
        }
    }
}


struct Logics {
    control:WinitControl,
    physics:PongPhysics
}

/// Representation of the application state. In this example, a box will bounce around the screen.
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
            .with_title("Hello Pixels")
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
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            paddles: (HEIGHT/2-PADDLE_HEIGHT/2, HEIGHT/2-PADDLE_HEIGHT/2),
            ball: (WIDTH/2-BALL_SIZE/2, HEIGHT/2-BALL_SIZE/2),
            ball_err: Vec2::new(0.0,0.0),
            ball_vel: Vec2::new(0.0,0.0),
            serving: Some(Player::P1)
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self, logics:&mut Logics, input:&WinitInputHelper) {
        logics.control.update(input);
        for choice in logics.control.selected_actions.iter() {
            match choice {
                (Player::P1, Action::Move(amt)) => self.paddles.0 += amt,
                (Player::P1, Action::Serve) => {
                    if let Some(Player::P1) = self.serving {
                        self.ball_vel = Vec2::new(8.0, 8.0);
                        self.serving = None;
                    }
                },
                (Player::P2, Action::Move(amt)) => self.paddles.1 += amt,
                (Player::P2, Action::Serve) => {
                    if let Some(Player::P2) = self.serving {
                        self.ball_vel = Vec2::new(-8.0, -8.0);
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

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0,0,0,255]);
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
