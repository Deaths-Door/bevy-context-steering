use bevy_many_relationships::{IncomingRelationships, OutgoingRelationships};

use super::*;

#[derive(Component, Deref)]
pub struct ClusterCentre(Vec3);

#[derive(Component, Deref)]
pub struct ClusterAverageVelocity(Vec3);

#[derive(QueryData)]
pub(crate) struct MemberClusterQueryData {
    transform: &'static GlobalTransform,
    velocity: &'static LinearVelocity,
}
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct ClusterQueryData {
    relationships: &'static IncomingRelationships<ClusterMember>,
    cluster_centre: &'static mut ClusterCentre,
    cluster_avg_vel: &'static mut ClusterAverageVelocity,
}

pub(crate) fn update_cluster_data(
    mut query_clusters: Query<ClusterQueryData, With<Cluster>>,
    query_members: Query<MemberClusterQueryData, With<OutgoingRelationships<ClusterMember>>>,
) {
    query_clusters.par_iter_mut().for_each(|mut item| {
        let mut total_centre = Vec3::ZERO;
        let mut total_velocity = Vec3::ZERO;

        let mut count = 0u32;

        query_members
            .iter_many(item.relationships.sources())
            .for_each(|member| {
                let translation = member.transform.translation();
                let velocity = **member.velocity;

                total_centre += translation;
                total_velocity += velocity;
                count += 1
            });

        let centre = total_centre / count as f32;
        let average_velocity = total_velocity / count as f32;

        item.cluster_centre.0 = centre;
        item.cluster_avg_vel.0 = average_velocity;
    });
}
