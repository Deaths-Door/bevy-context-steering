use super::*;
use bevy::platform::collections::HashMap;

#[derive(Resource, Clone, PartialEq, Eq, Debug, Reflect, Deref, Default)]
pub struct ClusterMap(HashMap<ClusterId, Entity>);

pub(crate) fn on_insert_cluster(
    trigger: On<Insert, Cluster>,
    mut clusters: ResMut<ClusterMap>,
    query: Query<&Cluster>,
) {
    if let Ok(Cluster(id)) = query.get(trigger.entity) {
        if let Some(previous_entity) = clusters.0.insert(*id, trigger.entity) {
            warn!(
                "Cluster ID collision detected! ID {id:?} was reassigned from entity {previous_entity:?} to entity {:?}.",
                trigger.entity
            );
        }
    }
}
pub(crate) fn on_discard_cluster(
    trigger: On<Discard, Cluster>,
    mut clusters: ResMut<ClusterMap>,
    query: Query<&Cluster>,
) {
    if let Ok(Cluster(id)) = query.get(trigger.entity) {
        clusters.0.remove(id);
    }
}
