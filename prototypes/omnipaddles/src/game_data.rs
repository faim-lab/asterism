use crate::{data::Reaction, Player, PoolID, World};
use asterism::resources::{QueuedResources, Transaction};
// physics::PointPhysics,
// Logic,
use macroquad::math::Vec2;

pub struct AddPoint {
    pub player: Player,
    pub ball_idx: usize,
}

impl Reaction<World> for AddPoint {
    type ReactingLogic = QueuedResources<PoolID>;

    fn react(&self, state: &mut World, resources: &mut QueuedResources<PoolID>) {
        state.balls[self.ball_idx].pos = state.balls[self.ball_idx].starting_pos;
        state.balls[self.ball_idx].vel = Vec2::zero();
        resources.transactions.push(vec![(
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

/* impl Reaction<World> for BounceWall {
    fn react(&self, state: &mut World, logic: &mut PointPhysics<Vec2>) {
        state.balls[self.0].vel.y *= -1.0;
    }
}*/
