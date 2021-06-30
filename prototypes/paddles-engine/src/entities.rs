//! adding/removing entities
use asterism::Logic;
use asterism::{collision::CollisionReaction, physics::PhysicsReaction};

use crate::types::*;
use crate::Game;
use crate::Predicate;

impl Game {
    pub fn add_paddle(&mut self, paddle: Paddle) -> PaddleID {
        let id = PaddleID::new(self.state.paddle_id_max);
        self.logics.consume_paddle(
            id,
            self.state
                .get_col_idx(self.state.paddles.len(), CollisionEnt::Paddle),
            paddle,
        );

        for Predicate { predicate, .. } in self.events.collision.iter_mut() {
            if let ColEvent::ByIdx(i, j) = predicate {
                if *i <= id.idx() {
                    *i += 1;
                }
                if *j <= id.idx() {
                    *j += 1;
                }
            }
        }

        self.state.paddle_id_max += 1;
        self.state.paddles.push(id);
        id
    }

    pub fn add_ball(&mut self, ball: Ball) -> BallID {
        let id = BallID::new(self.state.ball_id_max);
        self.logics.consume_ball(
            self.state
                .get_col_idx(self.state.balls.len(), CollisionEnt::Ball),
            ball,
        );
        self.state.ball_id_max += 1;
        self.state.balls.push(id);

        for Predicate { predicate, .. } in self.events.collision.iter_mut() {
            if let ColEvent::ByIdx(i, j) = predicate {
                if *i <= id.idx() {
                    *i += 1;
                }
                if *j <= id.idx() {
                    *j += 1;
                }
            }
        }

        id
    }

    pub fn add_wall(&mut self, wall: Wall) -> WallID {
        let id = WallID::new(self.state.wall_id_max);
        self.logics.consume_wall(
            self.state
                .get_col_idx(self.state.walls.len(), CollisionEnt::Wall),
            wall,
        );
        self.state.wall_id_max += 1;
        self.state.walls.push(id);

        for Predicate { predicate, .. } in self.events.collision.iter_mut() {
            if let ColEvent::ByIdx(i, j) = predicate {
                if *i <= id.idx() {
                    *i += 1;
                }
                if *j <= id.idx() {
                    *j += 1;
                }
            }
        }

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

        let mut remove = Vec::new();

        // collision events
        let col_idx = self.state.get_col_idx(ent_i, CollisionEnt::Paddle);

        for (idx, Predicate { predicate, .. }) in self.events.collision.iter_mut().enumerate() {
            if let ColEvent::ByIdx(i, j) = predicate {
                if *i == col_idx || *j == col_idx {
                    remove.push(idx);
                }
                if *i > ent_i {
                    *i -= 1;
                }
                if *j > ent_i {
                    *j -= 1;
                }
            }
        }
        for i in remove.iter().rev() {
            self.events.collision.remove(*i);
        }

        // control events
        remove.clear();
        for (idx, Predicate { predicate, .. }) in self.events.control.iter_mut().enumerate() {
            if predicate.set == ent_i {
                remove.push(idx);
            }
            if predicate.set > ent_i {
                predicate.set -= 1;
            }
        }
        for i in remove.into_iter().rev() {
            self.events.control.remove(i);
        }

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

        let mut remove = Vec::new();

        // collision events
        let col_idx = self.state.get_col_idx(ent_i, CollisionEnt::Wall);

        for (idx, Predicate { predicate, .. }) in self.events.collision.iter_mut().enumerate() {
            if let ColEvent::ByIdx(i, j) = predicate {
                if *i == col_idx || *j == col_idx {
                    remove.push(idx);
                }
                if *i > ent_i {
                    *i -= 1;
                }
                if *j > ent_i {
                    *j -= 1;
                }
            }
        }
        for i in remove.into_iter().rev() {
            self.events.collision.remove(i);
        }

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

        let mut remove = Vec::new();

        // collision events
        let col_idx = self.state.get_col_idx(ent_i, CollisionEnt::Wall);

        for (idx, Predicate { predicate, .. }) in self.events.collision.iter_mut().enumerate() {
            if let ColEvent::ByIdx(i, j) = predicate {
                if *i == col_idx || *j == col_idx {
                    remove.push(idx);
                }
                if *i > ent_i {
                    *i -= 1;
                }
                if *j > ent_i {
                    *j -= 1;
                }
            }
        }
        for i in remove.into_iter().rev() {
            self.events.collision.remove(i);
        }

        // remove physics events, n/a

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
