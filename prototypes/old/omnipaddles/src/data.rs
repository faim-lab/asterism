use asterism::control::ControlReaction;
use asterism::data::{Data as AsterData, *};
use asterism::resources::ResourceEvent;
use macroquad::prelude::{KeyCode, Vec2};

use crate::ids::*;

pub type Data = AsterData<CollisionID, ActionID, PoolID, Vec2, KeyCode>;

pub fn define_data() -> Data {
    let mut data = Data::new();
    data.add_interaction(
        EventWrapper::Collision((CollisionID::Ball(0), CollisionID::ScoreWall(Player::P1))),
        ReactionWrapper::Resource((PoolID::Points(Player::P2), 1.0)),
    );
    data.add_interaction(
        EventWrapper::Collision((CollisionID::Ball(0), CollisionID::ScoreWall(Player::P2))),
        ReactionWrapper::Resource((PoolID::Points(Player::P1), 1.0)),
    );
    data.add_interaction(
        EventWrapper::Collision((CollisionID::Ball(0), CollisionID::ScoreWall(Player::P1))),
        ReactionWrapper::Control(ControlReaction::SetKeyValid(1, ActionID::Serve)),
    );
    data.add_interaction(
        EventWrapper::Collision((CollisionID::Ball(0), CollisionID::ScoreWall(Player::P2))),
        ReactionWrapper::Control(ControlReaction::SetKeyValid(0, ActionID::Serve)),
    );
    data.add_interaction(
        EventWrapper::Collision((CollisionID::Ball(0), CollisionID::ScoreWall(Player::P1))),
        ReactionWrapper::GameState,
    );
    data.add_interaction(
        EventWrapper::Collision((CollisionID::Ball(0), CollisionID::ScoreWall(Player::P2))),
        ReactionWrapper::GameState,
    );
    data.add_interaction(
        EventWrapper::Collision((CollisionID::Ball(0), CollisionID::BounceWall)),
        ReactionWrapper::GameState,
    );
    data.add_interaction(
        EventWrapper::Collision((CollisionID::Ball(0), CollisionID::Paddle(Player::P1))),
        ReactionWrapper::GameState,
    );
    data.add_interaction(
        EventWrapper::Collision((CollisionID::Ball(0), CollisionID::Paddle(Player::P2))),
        ReactionWrapper::GameState,
    );
    data.add_interaction(
        EventWrapper::Resource(ResourceEvent::PoolUpdated(PoolID::Points(Player::P1))),
        ReactionWrapper::GameState,
    );
    data.add_interaction(
        EventWrapper::Resource(ResourceEvent::PoolUpdated(PoolID::Points(Player::P2))),
        ReactionWrapper::GameState,
    );
    data
}
