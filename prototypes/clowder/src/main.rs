#![deny(clippy::all)]
#![forbid(unsafe_code)]

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use ultraviolet::Vec2;
use asterism::{QueuedResources, resources::Transaction, AabbCollision, PointPhysics,
	       WinitKeyboardControl, KeyboardControl};

const WIDTH: u8 = 255;
const HEIGHT: u8 = 255;
const PADDLE_OFF_Y: u8 = 24;
const PADDLE_HEIGHT: u8 = 16;
const PADDLE_WIDTH: u8 = 24;
const BALL_SIZE: u8 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum ActionID {
    MoveRight(Player),
    MoveLeft(Player),
    MoveUp(Player),
    MoveDown(Player),
}

impl Default for ActionID {
    fn default() -> Self { Self::MoveLeft(Player::P1)}
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum CollisionID {
    Paddle(Player),
    Ball(B),
    InertWall(Location),
    Goal(Player),
}

impl Default for CollisionID {
    fn default() -> Self { Self::Ball(B::B1) }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
enum PoolID {
    Points(Player)
}

struct Logics {
    control: WinitKeyboardControl<ActionID>,
    physics: PointPhysics<Vec2>,
    collision: AabbCollision<CollisionID, Vec2>,
    resources: QueuedResources<PoolID>,
}

impl Logics {
    fn new() -> Self {
        Self {
            control: {
                let mut control = WinitKeyboardControl::new();
                control.add_key_map(0,
                    VirtualKeyCode::J,
                    ActionID::MoveRight(Player::P1),
                );
                control.add_key_map(0,
                    VirtualKeyCode::L,
                    ActionID::MoveLeft(Player::P1),
                );
		control.add_key_map(0,
                    VirtualKeyCode::I,
                    ActionID::MoveUp(Player::P1),
                );
                control.add_key_map(0,
                    VirtualKeyCode::K,
                    ActionID::MoveDown(Player::P1),
                );
                control
            },
            physics: PointPhysics::new(),
           collision: {
                let mut collision = AabbCollision::new();
	       collision.add_entity_as_xywh(0.0, HEIGHT as f32,
                    WIDTH as f32, 0.0,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::InertWall(Location::Bottom));
	       collision.add_entity_as_xywh(0.0, BALL_SIZE as f32,((WIDTH/2) - BALL_SIZE) as f32,
					    0.0,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::InertWall(Location::TopL));
	       collision.add_entity_as_xywh(((WIDTH/2) + (BALL_SIZE *2) ) as f32, BALL_SIZE as f32,
					    ((WIDTH/2) - BALL_SIZE) as f32, 0.0,
                    Vec2::new(0.0, 0.0),
                    true, true, CollisionID::InertWall(Location::TopR));
	       collision.add_entity_as_xywh(0.0,0.0,
					       0.0, HEIGHT as f32,
					       Vec2::new(0.0, 0.0),
                    true, true, CollisionID::InertWall(Location::Left));
                collision.add_entity_as_xywh(((WIDTH/2) -BALL_SIZE) as f32,0.0,(BALL_SIZE * 2) as f32, 0.0,
                    Vec2::new(0.0, 0.0),
					     true, true, CollisionID::Goal(Player::P1));
	       collision.add_entity_as_xywh(WIDTH as f32,0.0,
					       0.0, HEIGHT as f32,
					       Vec2::new(0.0, 0.0),
                    true, true, CollisionID::InertWall(Location::Right));
	       collision
           },
	    
            resources: {
                let mut resources = QueuedResources::new();
                resources.items.insert( PoolID::Points(Player::P1), 0.0 );
                resources
            }
}
    }
}
#[derive(Clone,Copy, Ord, PartialOrd, PartialEq, Eq)]
enum Location {
    Left,
    Right,
    TopL,
    TopR,
    Bottom
}

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
enum Player {
    P1
}


#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
enum B {
    B1,
    B2,
    B3
	
}

struct World {
    paddles: (Vec2, Vec2),
    ball: (Vec2, bool),
    ball2: (Vec2, bool),
    ball3: (Vec2, bool),
    ball_err: Vec2,
    ball_vel: Vec2,
    ball_vel2: Vec2,
    ball_vel3: Vec2,
    score: (u8, u8),
}


fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("clowder")
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
        if input.update(&event) {
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
            paddles: (Vec2::new((WIDTH/2-PADDLE_WIDTH/2) as f32, PADDLE_OFF_Y as f32),
		      Vec2::new((WIDTH/2-PADDLE_WIDTH/2) as f32,
		    ( HEIGHT-PADDLE_OFF_Y-PADDLE_HEIGHT) as f32)),
            ball: (Vec2::new((WIDTH/2-BALL_SIZE) as f32, (BALL_SIZE * 2) as f32), true),
	    ball2: (Vec2::new((WIDTH/2-BALL_SIZE) as f32, (WIDTH/2) as f32), true),
	    ball3: (Vec2::new((WIDTH-PADDLE_HEIGHT) as f32, (HEIGHT-PADDLE_WIDTH) as f32), true),
            ball_err: Vec2::new(0.0,0.0),
            ball_vel: Vec2::new(0.0,0.0),
	    ball_vel2: Vec2::new(0.0,0.0),
	    ball_vel3: Vec2::new(0.0,0.0),
            score: (0, 0),
        }
    }

       fn update(&mut self, logics: &mut Logics, input: &WinitInputHelper) {
        self.project_control(&mut logics.control);
        logics.control.update(input);
        self.unproject_control(&logics.control);

        self.project_physics(&mut logics.physics);
        logics.physics.update();
        self.unproject_physics(&logics.physics);

        self.project_collision(&mut logics.collision, &logics.control);
        logics.collision.update();
        self.unproject_collision(&logics.collision);

        for (idx,contact) in logics.collision.contacts.iter().enumerate() {
            match (logics.collision.metadata[contact.i].id,
                logics.collision.metadata[contact.j].id) {
                (CollisionID::Goal(_player), CollisionID::Ball(b)) => {
		    match b {
			B::B1 => {self.ball_vel = Vec2::new(0.0, 0.0);
		    self.ball.0 = Vec2::new(WIDTH as f32,HEIGHT as f32);
				  self.ball.1 = false},
			B::B2 =>
			    {self.ball_vel2 = Vec2::new(0.0, 0.0);
		    self.ball2.0 = Vec2::new(WIDTH as f32,HEIGHT as f32);
			     self.ball2.1 = false},
			B::B3 => {
			    self.ball_vel3 = Vec2::new(0.0, 0.0);
		    self.ball3.0 = Vec2::new(WIDTH as f32,HEIGHT as f32);
			    self.ball3.1 = false},
		    }
		    println!("goal");
                    logics.resources.transactions.push(vec![(PoolID::Points(Player::P1),
							     Transaction::Change(1))]);
			    
		    
		}
			
                (CollisionID::InertWall(location), CollisionID::Ball(b)) => {
		    match location {
			Location::Right | Location::Left =>
			    match b {
				B::B1 => {self.ball_vel.x *= -1.0;
				 println!("ball 1 hit side inertwall");
				},
				B::B2 => self.ball_vel2.x *= -1.0,
				B::B3 => self.ball_vel3.x *= -1.0,
			    },
			Location::TopL | Location::TopR  =>
			     match b {
				 B::B1 => {self.ball_vel.y *= -1.0;
				 println!("ball 1 hits top  wall");},
				B::B2 => self.ball_vel2.y *= -1.0,
				B::B3 => self.ball_vel3.y *= -1.0,
			     },
			Location::Bottom =>
			    match b {
				 B::B1 => {self.ball_vel.y *= -1.0;
				 println!("ball 1 hits bottom wall");},
				B::B2 => self.ball_vel2.y *= -1.0,
				B::B3 => self.ball_vel3.y *= -1.0,
			     },
		    }
                   
                }
		 

		(CollisionID::Ball(b), CollisionID::Paddle(player)) => {
		    let Vec2 {x:touch_x, y:touch_y} = logics.collision.sides_touched(idx);
		    if (self.ball_vel.x, self.ball_vel.y) == (0.0,0.0) ||
			(self.ball_vel2.x, self.ball_vel2.y) == (0.0,0.0) ||
			(self.ball_vel3.x, self.ball_vel3.y) == (0.0,0.0)
		    {
			if touch_y == 1.0
			{
			    match b {
				B::B1 => self.ball_vel = Vec2::new(1.0,1.0),
				B::B2 => self.ball_vel2 = Vec2::new(1.0,1.0),
				B::B3 => self.ball_vel3 = Vec2::new(1.0,1.0),
				
			    }
			    

			}

			else if touch_y == -1.0
			{
			     match b {
				B::B1 => self.ball_vel = Vec2::new(-1.0,-1.0),
				B::B2 => self.ball_vel2 = Vec2::new(-1.0,-1.0),
				B::B3 => self.ball_vel3 = Vec2::new(-1.0,-1.0),
				
			    }

			}
			
		    }
		    else {
			if touch_y != 0.0
			{
			    match b {
				B::B1 => self.ball_vel.y *= -1.0,
				B::B2 => self.ball_vel2.y *= -1.0,
				B::B3 => self.ball_vel3.y *= -1.0,
			    }
			}
			
			else if touch_x != 0.0
			{
			     match b {
				B::B1 => self.ball_vel.x *= -1.0,
				B::B2 => self.ball_vel2.x *= -1.0,
				B::B3 => self.ball_vel3.x *= -1.0,
			    }
			}
		    }
                    self.change_angle(player);
		},
		    _ => {}
		
	    
	    }
	}

        self.project_resources(&mut logics.resources);
        logics.resources.update();
        self.unproject_resources(&logics.resources);

        for (completed, item_types) in logics.resources.completed.iter() {
            if *completed {
                for item_type in item_types {
                    match item_type {
                        PoolID::Points(player) => {
                            match player {
                                Player::P1 => print!("p1"),
                           
                            }
                            println!(" scores! p1: {}, p2: {}", self.score.0, self.score.1);
                        }
                    }
                }
            }
        }
    }

    fn project_control(&self, control: &mut WinitKeyboardControl<ActionID>) {
        control.mapping[0][0].is_valid = true;
        control.mapping[0][1].is_valid = true;
	control.mapping[0][2].is_valid = true;
	control.mapping[0][3].is_valid = true;
    }

    fn unproject_control(&mut self, control: &WinitKeyboardControl<ActionID>) {
        self.paddles.0.x = ((self.paddles.0.x -
                control.values[0][0].value as f32 +
                control.values[0][1].value as f32)//confusing, incorporate ActionIds
			  .max(0.0) as f32).min((255 - PADDLE_WIDTH) as f32);//drive with data not code
	self.paddles.0.y = ((self.paddles.0.y as f32 -
                control.values[0][2].value as f32 +
                control.values[0][3].value as f32)
            .max(0.0) as f32).min((255 - PADDLE_HEIGHT) as f32);
    }
        
    


    fn project_physics(&self, physics: &mut PointPhysics<Vec2>) {
        physics.positions.resize_with(3, Vec2::default);
        physics.velocities.resize_with(3, Vec2::default);
        physics.accelerations.resize_with(3, Vec2::default);
        physics.add_physics_entity(0,
            Vec2::new(
                self.ball.0.x  + self.ball_err.x,
                self.ball.0.y  + self.ball_err.y),
            self.ball_vel,
				   Vec2::new(0.0, 0.0));
	 physics.add_physics_entity(1,
            Vec2::new(
                self.ball2.0.x  + self.ball_err.x,
                self.ball2.0.y  + self.ball_err.y),
            self.ball_vel2,
				    Vec2::new(0.0, 0.0));
	 physics.add_physics_entity(2,
            Vec2::new(
                self.ball3.0.x  + self.ball_err.x,
                self.ball3.0.y  + self.ball_err.y),
            self.ball_vel3,
            Vec2::new(0.0, 0.0));
    }


    fn unproject_physics(&mut self, physics: &PointPhysics<Vec2>) {
        self.ball.0.x = physics.positions[0].x.trunc().max(0.0).min((WIDTH - BALL_SIZE) as f32);
        self.ball.0.y = physics.positions[0].y.trunc().max(0.0).min((HEIGHT - BALL_SIZE) as f32);
	self.ball2.0.x = physics.positions[1].x.trunc().max(0.0).min((WIDTH - BALL_SIZE) as f32);
        self.ball2.0.y = physics.positions[1].y.trunc().max(0.0).min((HEIGHT - BALL_SIZE) as f32);
	self.ball3.0.x = physics.positions[2].x.trunc().max(0.0).min((WIDTH - BALL_SIZE) as f32);
        self.ball3.0.y = physics.positions[2].y.trunc().max(0.0).min((HEIGHT - BALL_SIZE) as f32);
        self.ball_err = physics.positions[0] - Vec2::new(self.ball.0.x as f32, self.ball.0.y as f32);
        self.ball_vel = physics.velocities[0];
	self.ball_vel2 = physics.velocities[1];
	self.ball_vel3 = physics.velocities[2];
    }

    fn project_collision(&self, collision: &mut AabbCollision<CollisionID,Vec2>, control: &WinitKeyboardControl<ActionID>) {
        collision.centers.resize_with(4, Default::default);
	collision.half_sizes.resize_with(4, Default::default);
        collision.velocities.resize_with(4, Default::default);
        collision.metadata.resize_with(4, Default::default);
        collision.add_entity_as_xywh(
            self.ball.0.x, self.ball.0.y,
            BALL_SIZE as f32, BALL_SIZE as f32,
            self.ball_vel,
            true, false, CollisionID::Ball(B::B1));
	collision.add_entity_as_xywh(
            self.ball2.0.x, self.ball2.0.y,
            BALL_SIZE as f32, BALL_SIZE as f32,
            self.ball_vel2,
            true, false, CollisionID::Ball(B::B2));
	collision.add_entity_as_xywh(
            self.ball3.0.x, self.ball3.0.y,
            BALL_SIZE as f32, BALL_SIZE as f32,
            self.ball_vel3,
            true, false, CollisionID::Ball(B::B3));
        collision.add_entity_as_xywh(
            self.paddles.0.x as f32, self.paddles.0.y as f32,
            PADDLE_WIDTH as f32, PADDLE_HEIGHT as f32,
            Vec2::new(-control.values[0][0].value + control.values[0][1].value,
		      -control.values[0][2].value+control.values[0][3].value),
                true, true, CollisionID::Paddle(Player::P1));
    }


    fn unproject_collision(&mut self, collision: &AabbCollision<CollisionID,Vec2>) {
	self.ball.0.x = (collision.centers[4].x - collision.half_sizes[4].x).trunc(); 
	self.ball.0.y = (collision.centers[4].y - collision.half_sizes[4].y).trunc();
    }

    fn change_angle(&mut self, player: Player) {
        let Vec2{x, y} = &mut self.ball_vel;
        let paddle_center = match player {
            Player::P1 => self.paddles.0.x + (PADDLE_WIDTH / 2) as f32
        } as f32;
        let angle: f32 = ((self.ball.0.x + (BALL_SIZE / 2) as f32 - paddle_center)
            .max(- (PADDLE_WIDTH as f32) / 2.0)
            .min(PADDLE_WIDTH as f32 / 2.0) / PADDLE_WIDTH as f32).abs() * 80.0;
        let magnitude = f32::sqrt(*x * *x + *y * *y);
        *x = angle.to_radians().sin() * magnitude
            * if *x < 0.0 { -1.0 } else { 1.0 };
        *y = angle.to_radians().cos() * magnitude
            * if *y < 0.0 { -1.0 } else { 1.0 };
    }

    fn project_resources(&self, resources: &mut QueuedResources<PoolID>) {
        if !resources.items.contains_key(&PoolID::Points(Player::P1)) {
            resources.items.insert(PoolID::Points(Player::P1), 0.0);
        }
    }

    fn unproject_resources(&mut self, resources: &QueuedResources<PoolID>) {
        for (completed, item_types) in resources.completed.iter() {
            if *completed {
                for item_type in item_types {
                    let value = resources.get_value_by_itemtype(item_type).min(255.0) as u8;
                    match item_type {
                        PoolID::Points(player) =>  {
                            match player {
                                Player::P1 => self.score.0 = value,
                               
                            }
                        }
                    }
                }
            }
        }
	
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0,0,128,255]);
        }
        draw_rect(self.paddles.0.x as u8, self.paddles.0.y as u8,
            PADDLE_WIDTH, PADDLE_HEIGHT,
            [255,255,255,255],
		  frame);
	if self.ball.1
	    {
		draw_rect(self.ball.0.x  as u8, self.ball.0.y as u8,
			  BALL_SIZE, BALL_SIZE,
			  [255,200,0,255],
			  frame);
	    }
	if self.ball2.1
	    {
		draw_rect(self.ball2.0.x as u8, self.ball2.0.y as u8,
			  BALL_SIZE, BALL_SIZE,
			  [255,200,0,255],
			  frame);
	    }
	if self.ball3.1
	    {
		draw_rect(self.ball3.0.x as u8, self.ball3.0.y as u8,
			  BALL_SIZE, BALL_SIZE,
			  [255,200,0,255],
			  frame);
	    }
	draw_rect(0, 0,
                  (WIDTH/2)-BALL_SIZE, BALL_SIZE,
		  [255,255,255,255],
		  frame);
	draw_rect((WIDTH/2)+(BALL_SIZE *2), 0,
		  (WIDTH/2)-BALL_SIZE,BALL_SIZE,
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
        for pixel in slice.chunks_exact_mut(4) {           pixel.copy_from_slice(&color);
        }
    }
}

