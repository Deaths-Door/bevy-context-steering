use super::*;

/// A marker component designating an entity as an active, movable steering agent.
#[derive(Component, Reflect, Default)]
#[require(SteeringContext)]

pub struct SteeringAgent;
