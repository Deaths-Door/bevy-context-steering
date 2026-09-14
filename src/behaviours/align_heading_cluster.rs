use bevy::{ecs::system::SystemParam, platform::collections::HashSet};

use super::*;

/// Aligns an agent's heading with the mean heading of its clusters.
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component, Debug)]
#[component(
    on_add = on_add_into_steering_context::<Self>,
    on_remove = on_remove_from_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct AlignHeadingCluster(HashSet<ClusterId>);

/// Controls how much a cluster contributes to [`AlignHeadingCluster`].
#[derive(Component, Debug, Reflect, Default)]
pub struct AlignHeadingClusterWeight {
    pub weight: ClusterWeight,
}

impl AlignHeadingCluster {
    /// Creates a cluster heading-alignment behaviour for no clusters.
    pub const fn new() -> Self {
        Self(HashSet::new())
    }

    pub(crate) fn steering_behaviour_update(
        mut system_param: BehaviourSystemParam<Self>,
    ) {
        system_param.agent_query.par_iter_mut().for_each(|mut agent| {
            let clusters = &(*agent.behaviour).0;
            let Some(target_direction) = weighted_average(clusters.iter(), |cluster_id| {
                system_param
                    .cluster_map
                    .get(cluster_id)
                    .and_then(|entity| system_param.cluster_query.get(*entity).ok())
                    .map(|(mean_heading, weight)| (**mean_heading, weight.weight))
            }) else {
                return;
            };

            agent
                .context
                .set_interest::<Self>(target_direction);
        });
    }
}

impl From<ClusterId> for AlignHeadingCluster {
    fn from(id: ClusterId) -> Self {
        Self::from_iter([id])
    }
}

impl FromIterator<ClusterId> for AlignHeadingCluster {
    fn from_iter<T: IntoIterator<Item = ClusterId>>(iter: T) -> Self {
        Self(HashSet::from_iter(iter))
    }
}

#[derive(SystemParam)]
pub(crate) struct BehaviourSystemParam<'w, 's, T: Component> {
    cluster_map: Res<'w, ClusterMap>,
    agent_query: ActiveAgentsQuery<'w, 's, BehaviourQueryData<T>>,
    cluster_query: ClusterQuery<
        'w,
        's,
        (&'static SteeringGroupMeanHeading, &'static AlignHeadingClusterWeight),
    >,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: Component> {
    behaviour: &'static T,
    context: &'static mut SteeringContext,
}