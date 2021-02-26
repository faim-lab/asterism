use crate::{data::Reaction, Player, PoolID, World};
use asterism::{
    physics::PointPhysics,
    resources::{QueuedResources, Transaction},
};
use macroquad::math::Vec2;

pub struct AddPoint {
    pub player: Player,
    pub ball_idx: usize,
}

impl Reaction<World, QueuedResources<PoolID>> for AddPoint {
    fn react(&self, state: &mut World, logic: &mut QueuedResources<PoolID>) {
        state.balls[self.ball_idx].pos = state.balls[self.ball_idx].starting_pos;
        state.balls[self.ball_idx].vel = Vec2::zero();
        logic.transactions.push(vec![(
            PoolID::Points(self.player),
            Transaction::Change(1.0),
        )]);
        state.serving = Some(self.player);
    }
}

pub struct BounceWall(usize);

impl BounceWall {
    pub fn new(ball_idx: usize) -> Self {
        Self(ball_idx)
    }
}

impl Reaction<World, PointPhysics<Vec2>> for BounceWall {
    fn react(&self, state: &mut World, logic: &mut PointPhysics<Vec2>) {
        state.balls[self.0].vel.y *= -1.0;
    }
}
