// very shaky on the difference between predicate and structural synthesis but honestly the theoretical difference is also kind of vague so it's fine

use asterism::{Logic, QueryTable};

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
            use asterism::collision::AabbColData;

            let control = self.logics.control.check_predicate(|_: &(usize, _)| true);
            let collison = self
                .logics
                .collision
                .check_predicate(|ent: &(usize, AabbColData)| ent.0 < self.state.paddles.len());

            for ((_, ctrl), (col_idx, mut col)) in control.into_iter().zip(collison.into_iter()) {
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
            let control = self.logics.control.check_predicate(|_: &(usize, _)| true);
            let collison = self.logics.collision.check_predicate(
                |ent: &(usize, asterism::collision::AabbColData)| ent.0 < self.state.paddles.len(),
            );

            for ((set, mut ctrl), (_, col)) in control.into_iter().zip(collison.into_iter()) {
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
                self.logics.control.update_synthesis(set, ctrl);
            }
        }
    }

    pub(crate) fn wall_synthesis(&mut self) {
        if let Some(synthesis) = self.events.wall_synth.col.as_ref() {
            let collison = self.logics.collision.check_predicate(
                |ent: &(usize, asterism::collision::AabbColData)| {
                    ent.0 < self.state.walls.len() + self.state.paddles.len()
                        && ent.0 >= self.state.paddles.len()
                },
            );
            for (col_idx, mut col) in collison.into_iter() {
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
            let physics = self.logics.physics.check_predicate(|_: &(usize, _)| true);
            let collison = self.logics.collision.check_predicate(
                |ent: &(usize, asterism::collision::AabbColData)| {
                    ent.0 >= self.state.paddles.len() + self.state.walls.len()
                },
            );

            for ((_, phys), (col_idx, mut col)) in physics.into_iter().zip(collison.into_iter()) {
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
            let physics = self.logics.physics.check_predicate(|_: &(usize, _)| true);
            let collison = self.logics.collision.check_predicate(
                |ent: &(usize, asterism::collision::AabbColData)| {
                    ent.0 >= self.state.paddles.len() + self.state.walls.len()
                },
            );

            for ((idx, mut phys), (_, col)) in physics.into_iter().zip(collison.into_iter()) {
                let mut ball = Ball::new();
                // get position from collision
                ball.pos = col.center - col.half_size;
                ball.size = col.half_size * 2.0;
                ball.vel = phys.vel;

                let ball = synthesis(ball);
                phys.pos = ball.pos;
                self.logics.physics.update_synthesis(idx, phys);
            }
        }
    }

    pub fn score_synthesis(&mut self) {
        if let Some(synthesis) = self.events.score_synth.rsrc.as_ref() {
            let resources = self
                .logics
                .resources
                .check_predicate(|_: &(RsrcPool, _)| true);
            for (id, vals) in resources.into_iter() {
                let mut score = Score::new();
                score.value = vals.0;

                let score = synthesis(score);
                self.logics
                    .resources
                    .update_synthesis(id, (score.value, u16::MIN, u16::MAX));
            }
        }
    }
}
