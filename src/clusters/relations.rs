use bevy_many_relationships::{
    IncomingRelationships, OnManyRelationshipAdded, OnManyRelationshipRemoved, OutgoingRelationships,
};

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Component, Reflect, Deref)]
#[component(immutable)]
pub struct ClusterMember(ClusterId);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Component)]
#[component(immutable)]
pub struct Cluster;

pub(crate) fn on_add_cluster_member(
    trigger: On<Add, OutgoingRelationships<ClusterMember>>,
    mut commands: Commands,
    query_cluster: Query<&ClusterMember, With<Cluster>>,
    query_cluster_member: Query<&ClusterMember, Without<Cluster>>,
) {
    let Ok(cluster_member) = query_cluster_member.get(trigger.entity) else {
        return;
    };
    
    let exists = query_cluster
        .iter()
        .any(|existing_member| existing_member == cluster_member);

    if !exists {
        commands.spawn((Cluster, *cluster_member));
    }
}

pub(crate) fn on_remove_cluster_member(
    trigger: On<Remove, (Cluster, IncomingRelationships<ClusterMember>)>,
    mut commands: Commands,
) {
    commands.entity(trigger.entity).despawn();
}
