// very shaky on the difference between predicate and structural synthesis but honestly the theoretical difference is also kind of vague so it's fine

use asterism::collision::{AabbColData, AabbCollision, CollisionReaction, Contact};
use asterism::control::{Action, ControlEvent, ControlEventType, ControlReaction, Values};
use asterism::physics::{PhysicsEvent, PhysicsReaction, PointPhysData};
use asterism::resources::{ResourceEvent, ResourceEventType, ResourceReaction, Transaction};
use asterism::Logic;

use crate::types::*;
use crate::{Game, Logics, State};

pub type Synthesis<Ident> = Box<dyn Fn(Ident) -> Ident>;

impl Game {
    pub fn set_paddle_col_synthesis(&mut self, synthesis: Synthesis<Paddle>) {
        self.events.paddle_synth.col = Some(synthesis);
    }
    pub fn set_paddle_ctrl_synthesis(&mut self, synthesis: Synthesis<Paddle>) {
        self.events.paddle_synth.ctrl = Some(synthesis);
    }

    pub fn set_ball_col_synthesis(&mut self, synthesis: Synthesis<Ball>) {
        self.events.ball_synth.col = Some(synthesis);
    }
    pub fn set_ball_phys_synthesis(&mut self, synthesis: Synthesis<Ball>) {
        self.events.ball_synth.phys = Some(synthesis);
    }

    pub fn set_wall_synthesis(&mut self, synthesis: Synthesis<Wall>) {
        self.events.wall_synth.col = Some(synthesis);
    }

    pub fn set_score_synthesis(&mut self, synthesis: Synthesis<Score>) {
        self.events.score_synth.rsrc = Some(synthesis);
    }

    pub(crate) fn paddle_col_synthesis(&mut self) {
        if let Some(synthesis) = self.events.paddle_synth.col.as_ref() {
            for paddle_id in (0..self.state.num_paddles).map(PaddleID::new) {
                let col_idx = self.state.get_col_idx(CollisionEnt::Paddle(paddle_id));
                let mut col = self.logics.collision.get_synthesis(col_idx);
                let ctrl = self.logics.control.get_synthesis(paddle_id.idx());

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

                let paddle = synthesis(paddle);
                col.half_size = paddle.size / 2.0;
                col.center = paddle.pos + col.half_size;
                self.logics.collision.update_synthesis(col_idx, col);
            }
        }
    }

    pub(crate) fn paddle_ctrl_synthesis(&mut self) {
        if let Some(synthesis) = self.events.paddle_synth.ctrl.as_ref() {
            for paddle_id in (0..self.state.num_paddles).map(PaddleID::new) {
                let col_idx = self.state.get_col_idx(CollisionEnt::Paddle(paddle_id));
                let col = self.logics.collision.get_synthesis(col_idx);
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
                let paddle = synthesis(paddle);

                for (((_, _, valid, vals), actions), values) in paddle
                    .controls
                    .iter()
                    .zip(ctrl.0.iter_mut())
                    .zip(ctrl.1.iter_mut())
                {
                    actions.is_valid = *valid;
                    *values = *vals;
                }
                self.logics.control.update_synthesis(paddle_id.idx(), ctrl);
            }
        }
    }

    pub(crate) fn wall_synthesis(&mut self) {
        if let Some(synthesis) = self.events.wall_synth.col.as_ref() {
            for wall_id in (0..self.state.num_walls).map(WallID::new) {
                let col_idx = self.state.get_col_idx(CollisionEnt::Wall(wall_id));
                let mut col = self.logics.collision.get_synthesis(col_idx);

                let mut wall = Wall::new();
                wall.pos = col.center - col.half_size;
                wall.size = col.half_size * 2.0;

                let wall = synthesis(wall);
                col.half_size = wall.size / 2.0;
                col.center = wall.pos + col.half_size;
                self.logics.collision.update_synthesis(col_idx, col);
            }
        }
    }

    pub(crate) fn ball_col_synthesis(&mut self) {
        if let Some(synthesis) = self.events.ball_synth.col.as_ref() {
            for ball_id in (0..self.state.num_balls).map(BallID::new) {
                let col_idx = self.state.get_col_idx(CollisionEnt::Ball(ball_id));
                let mut col = self.logics.collision.get_synthesis(col_idx);
                let phys = self.logics.physics.get_synthesis(ball_id.idx());

                let mut ball = Ball::new();
                // get position from physics
                ball.pos = phys.pos;
                ball.size = col.half_size * 2.0;
                ball.vel = phys.vel;

                let ball = synthesis(ball);

                col.half_size = ball.size / 2.0;
                col.center = ball.pos + col.half_size;

                self.logics.collision.update_synthesis(col_idx, col);
            }
        }
    }

    pub(crate) fn ball_phys_synthesis(&mut self) {
        if let Some(synthesis) = self.events.ball_synth.phys.as_ref() {
            for ball_id in (0..self.state.num_balls).map(BallID::new) {
                let col_idx = self.state.get_col_idx(CollisionEnt::Ball(ball_id));
                let col = self.logics.collision.get_synthesis(col_idx);
                let mut phys = self.logics.physics.get_synthesis(ball_id.idx());

                let mut ball = Ball::new();
                // get position from collision
                ball.pos = col.center - col.half_size;
                ball.size = col.half_size * 2.0;
                ball.vel = phys.vel;

                let ball = synthesis(ball);
                phys.pos = ball.pos;
                self.logics.physics.update_synthesis(ball_id.idx(), phys);
            }
        }
    }

    pub fn score_synthesis(&mut self) {
        if let Some(synthesis) = self.events.score_synth.rsrc.as_ref() {
            for score_id in (0..self.state.num_scores).map(ScoreID::new) {
                let rsrc_id = RsrcPool::Score(score_id);
                let rsrc = self.logics.resources.get_synthesis(rsrc_id);

                let mut score = Score::new();
                // pos can come from both collision and physics...
                // maybe should hold half_size, center, and pos separately?
                score.value = rsrc.0;

                let score = synthesis(score);
                self.logics
                    .resources
                    .update_synthesis(rsrc_id, (score.value, u16::MIN, u16::MAX));
            }
        }
    }
}
