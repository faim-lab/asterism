// very shaky on the difference between predicate and structural synthesis but honestly the theoretical difference is also kind of vague so it's fine

use asterism::Logic;

use crate::types::*;
use crate::Game;

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
            for (i, paddle_id) in self.state.paddles.iter().enumerate() {
                let col_idx = self.state.get_col_idx(i, CollisionEnt::Paddle);
                let mut col = self.logics.collision.get_synthesis(col_idx);
                let ctrl = self.logics.control.get_synthesis(paddle_id.idx());

                let mut paddle = Paddle::new();
                paddle.pos = col.center - col.half_size;
                paddle.size = col.half_size * 2.0;
                for actions in ctrl.iter() {
                    let ctrl = (actions.id, *actions.get_keycode(), actions.is_valid);
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
            for (i, paddle_id) in self.state.paddles.iter().enumerate() {
                let col_idx = self.state.get_col_idx(i, CollisionEnt::Paddle);
                let col = self.logics.collision.get_synthesis(col_idx);
                let mut ctrl = self.logics.control.get_synthesis(paddle_id.idx());

                let mut paddle = Paddle::new();
                paddle.pos = col.center - col.half_size;
                paddle.size = col.half_size * 2.0;
                for actions in ctrl.iter() {
                    let ctrl = (actions.id, *actions.get_keycode(), actions.is_valid);
                    paddle.controls.push(ctrl);
                }

                let paddle = synthesis(paddle);

                for ((_, _, valid), actions) in paddle.controls.iter().zip(ctrl.iter_mut()) {
                    actions.is_valid = *valid;
                }
                self.logics.control.update_synthesis(paddle_id.idx(), ctrl);
            }
        }
    }

    pub(crate) fn wall_synthesis(&mut self) {
        if let Some(synthesis) = self.events.wall_synth.col.as_ref() {
            for (i, _) in self.state.walls.iter().enumerate() {
                let col_idx = self.state.get_col_idx(i, CollisionEnt::Wall);
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
            for (i, ball_id) in self.state.balls.iter().enumerate() {
                let col_idx = self.state.get_col_idx(i, CollisionEnt::Ball);
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
            for (i, ball_id) in self.state.balls.iter().enumerate() {
                let col_idx = self.state.get_col_idx(i, CollisionEnt::Ball);
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
            for score_id in self.state.scores.iter() {
                let rsrc_id = RsrcPool::Score(*score_id);
                let rsrc = self.logics.resources.get_synthesis(rsrc_id);

                let mut score = Score::new();
                score.value = rsrc.0;

                let score = synthesis(score);
                self.logics
                    .resources
                    .update_synthesis(rsrc_id, (score.value, u16::MIN, u16::MAX));
            }
        }
    }
}
