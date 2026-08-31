use super::*;

/// The spatial center of the cluster, computed across all members.
#[derive(Component, Deref)]
pub struct ClusterCentre(pub(super) Vec3);

/// The mean velocity vector of all members belonging to the cluster.
#[derive(Component, Deref)]
pub struct ClusterAverageVelocity(pub(super) Vec3);
