use super::*;

/// A marker component designating an entity as an active, movable steering agent.
#[derive(Component, Reflect, Default)]
#[require(SteeringContext)]

pub struct SteeringAgent;

/// The center of the roup (cluster or neighbourhood)
#[derive(Component, Deref, Default)]
pub struct SteeringGroupCentre(pub(super) Vec3);

/// The mean velocity vector of all members belonging to the group (cluster or neighbourhood).
#[derive(Component, Deref, Default)]
pub struct SteeringGroupMeanVelocity(pub(super) Vec3);

/// The mean heading vector of all members belonging to the group (cluster or neighbourhood).
#[derive(Component, Deref, Default)]
pub struct SteeringGroupMeanHeading(pub(super) Vec3);