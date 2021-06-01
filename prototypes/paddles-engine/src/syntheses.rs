//! very shaky on the difference between predicate and structural synthesis but honestly the theoretical difference is also kind of vague so it's fine

use asterism::collision::{CollisionEvent, CollisionReaction};
use asterism::control::{ControlEvent, ControlEventType, ControlReaction};
use asterism::physics::{PhysicsEvent, PhysicsReaction};
use asterism::resources::{ResourceEvent, ResourceEventType, ResourceReaction, Transaction};
use asterism::Logic;

use crate::types::*;
use crate::{Game, Logics, State};

pub type Synthesis<Ident> = Box<dyn Fn(&mut Ident)>;

impl Game {
    pub fn set_paddle_synthesis(&mut self, synthesis: Synthesis<Paddle>) {
        self.events.paddle_synth = synthesis;
    }
    pub fn set_ball_synthesis(&mut self, synthesis: Synthesis<Ball>) {
        self.events.ball_synth = synthesis;
    }
    pub fn set_wall_synthesis(&mut self, synthesis: Synthesis<Wall>) {
        self.events.wall_synth = synthesis;
    }
    pub fn set_score_synthesis(&mut self, synthesis: Synthesis<Score>) {
        self.events.score_synth = synthesis;
    }

    // this is just projection but for a single entity instead of an entire game state
    pub(crate) fn paddle_synthesis(&mut self) {
        for paddle_id in self.state.paddles.iter() {
            let col_idx = self.state.get_col_idx(CollisionEnt::Paddle(*paddle_id));
            let mut col = self.logics.collision.get_synthesis(col_idx);
            let mut ctrl = self.logics.control.get_synthesis(paddle_id.idx());

            let mut paddle = Paddle::new();
            paddle.pos = col.center - col.half_size;
            paddle.size = col.half_size * 2.0;
            for (actions, values) in ctrl.0.iter().zip(ctrl.1.iter()) {
                let ctrl = (
                    actions.id,
                    *actions.get_keycode(),
                    actions.is_valid,
                    *values,
                );
                paddle.controls.push(ctrl);
            }

            (self.events.paddle_synth)(&mut paddle);

            col.half_size = paddle.size / 2.0;
            col.center = paddle.pos + col.half_size;
            for (((_, _, valid, vals), actions), values) in paddle
                .controls
                .iter()
                .zip(ctrl.0.iter_mut())
                .zip(ctrl.1.iter_mut())
            {
                actions.is_valid = *valid;
                *values = *vals;
            }

            self.logics.collision.update_synthesis(col_idx, col);
            self.logics.control.update_synthesis(paddle_id.idx(), ctrl);
        }
    }

    pub(crate) fn wall_synthesis(&mut self) {
        for wall_id in self.state.walls.iter() {
            let col_idx = self.state.get_col_idx(CollisionEnt::Wall(*wall_id));
            let mut col = self.logics.collision.get_synthesis(col_idx);

            let mut wall = Wall::new();
            wall.pos = col.center - col.half_size;
            wall.size = col.half_size * 2.0;

            (self.events.wall_synth)(&mut wall);

            col.half_size = wall.size / 2.0;
            col.center = wall.pos + col.half_size;

            self.logics.collision.update_synthesis(col_idx, col);
        }
    }

    pub(crate) fn ball_synthesis(&mut self) {
        for ball_id in self.state.balls.iter() {
            let col_idx = self.state.get_col_idx(CollisionEnt::Ball(*ball_id));
            let mut col = self.logics.collision.get_synthesis(col_idx);
            let mut phys = self.logics.physics.get_synthesis(ball_id.idx());

            let mut ball = Ball::new();
            // pos can come from both collision and physics...
            // maybe paddle should hold half_size, center, and pos separately?
            ball.pos = col.center - col.half_size;
            ball.size = col.half_size * 2.0;
            ball.vel = phys.vel;

            (self.events.ball_synth)(&mut ball);

            col.half_size = ball.size / 2.0;
            col.center = ball.pos + col.half_size;
            phys.pos = ball.pos;
            phys.vel = ball.vel;

            self.logics.collision.update_synthesis(col_idx, col);
            self.logics.physics.update_synthesis(ball_id.idx(), phys);
        }
    }

    pub fn score_synthesis(&mut self) {
        for score_id in self.state.scores.iter() {
            let rsrc_id = RsrcPool::Score(*score_id);
            let mut rsrc = self.logics.resources.get_synthesis(rsrc_id);

            let mut score = Score::new();
            // pos can come from both collision and physics...
            // maybe should hold half_size, center, and pos separately?
            score.value = rsrc.0;

            (self.events.score_synth)(&mut score);

            rsrc.0 = score.value;

            self.logics.resources.update_synthesis(rsrc_id, rsrc);
        }
    }
}
