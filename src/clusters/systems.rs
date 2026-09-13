use bevy_many_relationships::{IncomingRelationships, OutgoingRelationships};

use super::*;

#[derive(QueryData)]
pub(crate) struct MemberClusterQueryData {
    transform: &'static GlobalTransform,
    velocity: &'static LinearVelocity,
}
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct ClusterGroupQueryData {
    relationships: &'static IncomingRelationships<ClusterMember>,
    centre: &'static mut SteeringGroupCentre,
    average_velocity: &'static mut SteeringGroupMeanVelocity,
    average_heading: &'static mut SteeringGroupMeanHeading,
}

pub(crate) fn update_cluster_properties(
    mut query_group: Query<ClusterGroupQueryData, With<Cluster>>,
    query_members: Query<MemberClusterQueryData, With<OutgoingRelationships<ClusterMember>>>,
) {
    query_group.par_iter_mut().for_each(|mut item| {
        let mut total_centre = Vec3::ZERO;
        let mut total_velocity = Vec3::ZERO;
        let mut total_heading = Vec3::ZERO;
        let mut count = 0u32;

        query_members
            .iter_many(item.relationships.sources())
            .for_each(|member| {
                total_centre += member.transform.translation();
                total_velocity += **member.velocity;
                total_heading += member.transform.forward().as_vec3();
                count += 1
            });

        let centre = total_centre / count as f32;
        let average_velocity = total_velocity / count as f32;
        let average_heading = total_heading / count as f32;

        item.centre.0 = centre;
        item.average_velocity.0 = average_velocity;
        item.average_heading.0 = average_heading;
    });
}
