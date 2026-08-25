use super::*;

/// Pre-Update Stage: Spatial partitioning & cluster generation
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SteeringSpatialSet;

/// Update Stage: Behavior calculation & steering force evaluation
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SteeringBehaviorSet;

/// Post-Update Stage: Force resolution, accumulation & integration
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SteeringPhysicsSet;
