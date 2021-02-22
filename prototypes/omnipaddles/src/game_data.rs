use crate::{Player, PoolID, World};
use asterism::{
    physics::PointPhysics,
    resources::{QueuedResources, Transaction},
};
use macroquad::math::Vec2;

struct AddPoint {
    player: Player,
    // i don't know how this information gets passed from the event to the reaction...
    // eugh i probably should rethink how events are represented
    ball_idx: usize,
}

impl Reaction<World, QueuedResources<PoolID>> for AddPoint {
    fn for_logic(&self) -> LogicType {
        LogicType::Resource
    }

    fn react_unchecked(&self, state: &mut World, logic: &mut QueuedResources<PoolID>) {
        state.balls[self.ball_idx].pos = self.balls[self.ball_idx].starting_pos;
        state.balls[self.ball_idx].vel = Vec2::zero();
        match self.player {
            Player::P1 => {
                logic
                    .transactions
                    .push(vec![(PoolID::Points(Player::P2), Transaction::Change(1.0))]);
                state.serving = Some(Player::P2);
            }
            Player::P2 => {
                logic
                    .transactions
                    .push(vec![(PoolID::Points(Player::P1), Transaction::Change(1.0))]);
                state.serving = Some(Player::P2);
            }
        }
    }
}

struct BounceWall(usize);

impl Reaction<World, PointPhysics<Vec2>> for BounceWall {
    fn for_logic(&self) -> LogicType {
        LogicType::Physics
    }

    fn react_unchecked(&self, state: &mut World, logic: &mut PointPhysics<Vec2>) {
        // ???
    }
}
