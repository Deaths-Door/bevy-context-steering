use bevy::ecs::component::Mutable;
use bevy_many_relationships::ManyRelatedEntityCommands;

use super::*;

pub trait ClusterEntityCommandsExt {
    /// Links this entity directly to the cluster identified by `ClusterId`.
    fn enter_cluster(self, cluster_id: ClusterId);

    /// Unlinks this entity from the cluster identified by `ClusterId`.
    fn exit_cluster(self, cluster_id: ClusterId);

    /// Same as [Self::enter_cluster] but for batching
    fn enter_clusters(self, cluster_ids: impl IntoIterator<Item = ClusterId> + Send + 'static);

    /// Same as [Self::enter_cluster] but for batching
    fn exit_clusters(self, cluster_ids: impl IntoIterator<Item = ClusterId> + Send + 'static);

    /// Sets the weight of a specific cluster property behavior for this entity.
    fn set_cluster_property_weight<B: Component<Mutability = Mutable> + AsMut<ClusterWeight>>(
        self,
        cluster_id: ClusterId,
        weight: ClusterWeight,
    );

    /// Modifies the weight of a cluster property behavior using a closure.
    fn update_cluster_property_weight<B: Component<Mutability = Mutable> + AsMut<ClusterWeight>>(
        self,
        cluster_id: ClusterId,
        modify: impl FnOnce(&mut ClusterWeight) + Send + 'static,
    );
}

impl<'a> ClusterEntityCommandsExt for EntityCommands<'a> {
    fn enter_cluster(mut self, cluster_id: ClusterId) {
        (&mut self).enter_cluster(cluster_id);
    }

    fn exit_cluster(mut self, cluster_id: ClusterId) {
        (&mut self).exit_cluster(cluster_id);
    }

    fn enter_clusters(mut self, cluster_ids: impl IntoIterator<Item = ClusterId> + Send + 'static) {
        (&mut self).enter_clusters(cluster_ids);
    }

    fn exit_clusters(mut self, cluster_ids: impl IntoIterator<Item = ClusterId> + Send + 'static) {
        (&mut self).exit_clusters(cluster_ids);
    }

    fn set_cluster_property_weight<B: Component<Mutability = Mutable> + AsMut<ClusterWeight>>(
        mut self,
        cluster_id: ClusterId,
        weight: ClusterWeight,
    ) {
        (&mut self).update_cluster_property_weight::<B>(cluster_id, move |source| *source = weight)
    }

    fn update_cluster_property_weight<B: Component<Mutability = Mutable> + AsMut<ClusterWeight>>(
        mut self,
        cluster_id: ClusterId,
        modify: impl FnOnce(&mut ClusterWeight) + Send + 'static,
    ) {
        (&mut self).update_cluster_property_weight::<B>(cluster_id, modify)
    }
}

impl<'a> ClusterEntityCommandsExt for &'_ mut EntityCommands<'a> {
    fn enter_cluster(self, cluster_id: ClusterId) {
        self.enter_clusters(std::iter::once(cluster_id));
    }

    fn exit_cluster(self, cluster_id: ClusterId) {
        self.exit_clusters(std::iter::once(cluster_id));
    }

    fn enter_clusters(self, cluster_ids: impl IntoIterator<Item = ClusterId> + Send + 'static) {
        let source_entity = self.id();
        self.commands().queue(move |world: &mut World| {
            for cluster_id in cluster_ids {
                let cluster_entity = match world.resource::<ClusterMap>().get(&cluster_id) {
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
            }
        });
    }

    fn exit_clusters(self, cluster_ids: impl IntoIterator<Item = ClusterId> + Send + 'static) {
        let source_entity = self.id();
        self.commands().queue(move |world: &mut World| {
            for cluster_id in cluster_ids {
                let clusters = world.resource::<ClusterMap>();

                let Some(&cluster_entity) = clusters.get(&cluster_id) else {
                    warn!("Cannot leave cluster: Cluster ID {cluster_id:?} does not exist.");
                    return;
                };

                let mut commands = world.commands();
                commands
                    .entity(source_entity)
                    .remove_outgoing_to::<ClusterMember>(cluster_entity);
            }
        });
    }

    fn set_cluster_property_weight<B: Component<Mutability = Mutable> + AsMut<ClusterWeight>>(
        self,
        cluster_id: ClusterId,
        weight: ClusterWeight,
    ) {
        self.update_cluster_property_weight::<B>(cluster_id, move |s| *s = weight);
    }

    fn update_cluster_property_weight<B: Component<Mutability = Mutable> + AsMut<ClusterWeight>>(
        self,
        cluster_id: ClusterId,
        modify: impl FnOnce(&mut ClusterWeight) + Send + 'static,
    ) {
        self.commands().queue(move |world: &mut World| {
            let Some(&cluster_entity) = world
                .get_resource::<ClusterMap>()
                .and_then(|clusters| clusters.get(&cluster_id))
            else {
                warn!("Cannot get property from cluster {cluster_id:?}");
                return;
            };

            let Some(mut component) = world.get_mut::<B>(cluster_entity) else {
                warn!("Component missing on cluster entity {cluster_entity:?}");
                return;
            };

            let weight = component.as_mut().as_mut();
            modify(weight)
        });
    }
}
