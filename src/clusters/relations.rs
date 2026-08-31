use super::*;

/// Connects individual member entities to their parent `Cluster` root entity.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ClusterMember;

/// Marker component for the cluster's root entity.
///
/// The root entity acts as an organizational hub and holds aggregated cluster data
/// useful for collective behaviors (e.g., `ClusterCentre`, `ClusterAverageVelocity`).
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Reflect, Deref)]
#[component(immutable)]
pub struct Cluster(pub ClusterId);
