use super::*;
use bevy::platform::collections::HashMap;

/// Steering behavior that pushes the agent away from the center of mass of its assigned clusters.
///
/// While [`Scatter`] pulls agents together to form a group, `Scatter` acts as a
/// "cluster-level repulsion." It calculates a vector pointing radially outward from the
/// weighted average of cluster centers.
#[derive(Component, Debug, Reflect, Default, Deref, DerefMut)]
#[component(on_add = on_add_into_steering_context::<Self>, on_remove = on_remove_from_steering_context::<Self>)]
#[require(SteeringContext)]
pub struct Scatter {
    clusters: HashMap<ClusterId, ClusterWeight>,
}

impl Scatter {
    pub(crate) fn steering_behaviour_update(
        clusters: Res<ClusterMap>,
        agent_query: ActiveAgentsQuery<cohere::BehaviourQueryData<Self>>,
        cluster_query: Query<&ClusterCentre, With<Cluster>>,
    ) {
        cohere::update_internal::<Self>(clusters, agent_query, cluster_query)
    }
}

impl cohere::BehaviourData for Scatter {
    fn clusters(&self) -> impl Iterator<Item = (&ClusterId, &ClusterWeight)> {
        self.clusters.iter()
    }
    fn apply(&self, context: &mut SteeringContext, cohesion_dir: Vec3) {
        context.set_danger::<Self>(cohesion_dir);
    }
}

impl FromIterator<ClusterId> for Scatter {
    fn from_iter<T: IntoIterator<Item = ClusterId>>(iter: T) -> Self {
        Self::from_iter(
            iter.into_iter()
                .map(|value| (value, ClusterWeight::default())),
        )
    }
}

impl FromIterator<(ClusterId, ClusterWeight)> for Scatter {
    fn from_iter<T: IntoIterator<Item = (ClusterId, ClusterWeight)>>(iter: T) -> Self {
        Self {
            clusters: HashMap::from_iter(iter),
        }
    }
}

impl From<ClusterId> for Scatter {
    fn from(id: ClusterId) -> Self {
        Self::from_iter([id])
    }
}

impl From<(ClusterId, ClusterWeight)> for Scatter {
    fn from(id: (ClusterId, ClusterWeight)) -> Self {
        Self::from_iter([id])
    }
}
