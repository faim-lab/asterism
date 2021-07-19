//! adding/removing entities
use asterism::Logic;
use asterism::{collision::CollisionReaction, physics::PhysicsReaction};

use crate::types::*;
use crate::Game;

impl Game {
    pub fn add_paddle(&mut self, paddle: Paddle) -> PaddleID {
        let id = PaddleID::new(self.state.paddle_id_max);
        self.logics.consume_paddle(
            id,
            self.state
                .get_col_idx(self.state.paddles.len(), CollisionEnt::Paddle),
            paddle,
        );

        self.state.paddle_id_max += 1;
        self.state.paddles.push(id);
        id
    }

    pub fn add_ball(&mut self, ball: Ball) -> BallID {
        let id = BallID::new(self.state.ball_id_max);
        let col_idx = self
            .state
            .get_col_idx(self.state.balls.len(), CollisionEnt::Ball);
        self.logics.consume_ball(col_idx, ball);
        self.state.ball_id_max += 1;
        self.state.balls.push(id);

        id
    }

    pub fn add_wall(&mut self, wall: Wall) -> WallID {
        let id = WallID::new(self.state.wall_id_max);
        let col_idx = self
            .state
            .get_col_idx(self.state.walls.len(), CollisionEnt::Wall);
        self.logics.consume_wall(col_idx, wall);
        self.state.wall_id_max += 1;
        self.state.walls.push(id);

        id
    }

    pub fn add_score(&mut self, score: Score) -> ScoreID {
        let id = ScoreID::new(self.state.score_id_max);
        self.logics.consume_score(id, score);
        self.state.score_id_max += 1;
        self.state.scores.push(id);
        id
    }

    pub(crate) fn remove_paddle(&mut self, paddle: PaddleID) {
        let ent_i = self
            .state
            .paddles
            .iter()
            .position(|pid| *pid == paddle)
            .unwrap();
        self.logics.control.mapping.remove(ent_i);
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(
                self.state.get_col_idx(ent_i, CollisionEnt::Paddle),
            ));

        self.state.paddles.remove(ent_i);
    }

    pub(crate) fn remove_wall(&mut self, wall: WallID) {
        let ent_i = self
            .state
            .walls
            .iter()
            .position(|wid| *wid == wall)
            .unwrap();
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(
                self.state.get_col_idx(ent_i, CollisionEnt::Wall),
            ));

        self.state.walls.remove(ent_i);
    }

    pub(crate) fn remove_ball(&mut self, ball: BallID) {
        let ent_i = self
            .state
            .balls
            .iter()
            .position(|bid| *bid == ball)
            .unwrap();
        self.logics
            .physics
            .handle_predicate(&PhysicsReaction::RemoveBody(ent_i));
        self.logics
            .collision
            .handle_predicate(&CollisionReaction::RemoveBody(
                self.state.get_col_idx(ent_i, CollisionEnt::Ball),
            ));

        self.state.balls.remove(ent_i);
    }

    pub(crate) fn remove_score(&mut self, score: ScoreID) {
        let ent_i = self
            .state
            .scores
            .iter()
            .position(|sid| *sid == score)
            .unwrap();
        let rsrc = RsrcPool::Score(score);
        self.logics.resources.items.remove(&rsrc);

        self.state.scores.remove(ent_i);
    }
}
