use crate::{collision, physics, resources, Logic, Reaction};

pub struct Data<PoolID, V2, CollisionID>
where
    PoolID: resources::PoolInfo,
    V2: collision::Vec2,
    CollisionID: Copy + Eq,
{
    pub resource_interactions: Vec<(
        <resources::QueuedResources<PoolID> as Logic>::Event,
        Box<dyn Reaction>,
    )>,

    pub collision_interactions: Vec<(
        <collision::AabbCollision<CollisionID, V2> as Logic>::Event,
        Box<dyn Reaction>,
    )>,

    pub physics_interactions: Vec<(
        <physics::PointPhysics<V2> as Logic>::Event,
        Box<dyn Reaction>,
    )>,
}

impl<PoolID, V2, CollisionID> Data<PoolID, V2, CollisionID>
where
    PoolID: resources::PoolInfo,
    V2: collision::Vec2,
    CollisionID: Copy + Eq,
{
    pub fn resource_react(
        &self,
        event: <resources::QueuedResources<PoolID> as Logic>::Event,
        logic: &mut resources::QueuedResources<PoolID>,
    ) {
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
    }

    pub fn physics_react(
        &self,
        event: <physics::PointPhysics<V2> as Logic>::Event,
        logic: &mut physics::PointPhysics<V2>,
    ) {
    }

    pub fn collision_react(
        &self,
        event: <collision::AabbCollision<CollisionID, V2> as Logic>::Event,
        logic: &mut collision::AabbCollision<CollisionID, V2>,
    ) {
        if let Some((_, reaction)) = self
            .collision_interactions
            .iter()
            .position(|(cllsn_event, _)| *cllsn_event == event)
            .and_then(|i| Some(&self.resource_interactions[i]))
        {}
    }
}
