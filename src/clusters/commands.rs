use bevy_many_relationships::ManyRelatedEntityCommands;

use super::*;

pub trait ClusterEntityCommandsExt {
    /// Links this entity directly to the cluster identified by `ClusterId`.
    /// Handles dummy cluster creation automatically in O(1).
    fn enter_cluster(self, cluster_id: ClusterId);

    /// Unlinks this entity from the cluster identified by `ClusterId`.
    fn exit_cluster(self, cluster_id: ClusterId);
}

impl<'a> ClusterEntityCommandsExt for EntityCommands<'a> {
    fn enter_cluster(mut self, cluster_id: ClusterId) {
        (&mut self).enter_cluster(cluster_id);
    }

    fn exit_cluster(mut self, cluster_id: ClusterId) {
        (&mut self).exit_cluster(cluster_id);
    }
}

impl<'a> ClusterEntityCommandsExt for &'_ mut EntityCommands<'a> {
    fn enter_cluster(self, cluster_id: ClusterId) {
        let source_entity = self.id();
        self.commands().queue(move |world: &mut World| {
            let clusters = world.resource::<ClusterMap>();

            let cluster_entity = match clusters.get(&cluster_id) {
                Some(entity) => *entity,
                None => {
                    let commands = world.spawn(Cluster(cluster_id));
                    commands.id()
                }
            };
            let mut commands = world.commands();
            commands
                .entity(cluster_entity)
                .add_incoming_from(source_entity, ClusterMember);
        });
    }

    fn exit_cluster(self, cluster_id: ClusterId) {
        let source_entity = self.id();
        self.commands().queue(move |world: &mut World| {
            let clusters = world.resource::<ClusterMap>();

            let Some(&cluster_entity) = clusters.get(&cluster_id) else {
                warn!("Cannot leave cluster: Cluster ID {cluster_id:?} does not exist.");
                return;
            };

            let mut commands = world.commands();
            commands
                .entity(source_entity)
                .remove_outgoing_to::<ClusterMember>(cluster_entity);
        });
    }
}
