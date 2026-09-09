use bevy::{ecs::system::SystemParam, platform::collections::HashSet};

use super::*;

/// Steering behavior that pulls the agent toward the center of mass of the cluster
#[derive(Component, Debug, Default, Deref, DerefMut)]
#[component(on_add = on_add_into_steering_context::<Self>, on_remove = on_remove_from_steering_context::<Self>)]
#[require(SteeringContext)]
pub struct CohereCluster(HashSet<ClusterId>);

/// Inserted on the cluster entity itself
#[derive(Component, Debug, Reflect, Default)]
pub struct CohereClusterWeight {
    pub weight: ClusterWeight,
}

impl CohereCluster {
    pub(crate) fn steering_behaviour_update(system_parm: BehaviourSystemParm<Self>) {
        update_internal::<Self>(system_parm)
    }
}

impl Behaviour for CohereCluster {
    fn clusters(&self) -> impl Iterator<Item = &ClusterId> {
        self.0.iter()
    }

    fn update_context(&self, context: &mut SteeringContext, target_direction: Vec3) {
        context.set_interest::<Self>(target_direction);
    }
}
pub(super) fn update_internal<T>(mut system_parm: BehaviourSystemParm<T>)
where
    T: Component + Behaviour,
{
    system_parm
        .agent_query
        .par_iter_mut()
        .for_each(|mut agent| {
            let clusters = agent.behaviour.clusters();
            let Some(target_direction) = weighted_average(clusters, |cluster_id| {
                system_parm
                    .cluster_map
                    .get(cluster_id)
                    .and_then(|entity| system_parm.cluster_query.get(*entity).ok())
                    .map(|(centre, weight)| (**centre, weight.weight))
            }) else {
                return;
            };

            agent
                .behaviour
                .update_context(&mut *agent.context, target_direction);
        });
}

impl From<ClusterId> for CohereCluster {
    fn from(id: ClusterId) -> Self {
        Self::from_iter([id])
    }
}

impl FromIterator<ClusterId> for CohereCluster {
    fn from_iter<T: IntoIterator<Item = ClusterId>>(iter: T) -> Self {
        Self(HashSet::from_iter(iter))
    }
}

#[derive(SystemParam)]
pub(crate) struct BehaviourSystemParm<'w, 's, T: Component> {
    cluster_map: Res<'w, ClusterMap>,
    agent_query: ActiveAgentsQuery<'w, 's, BehaviourQueryData<T>>,
    cluster_query: ClusterQuery<'w, 's, (&'static ClusterCentre, &'static CohereClusterWeight)>,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: Component> {
    behaviour: &'static T,
    transform: &'static Transform,
    context: &'static mut SteeringContext,
}

pub(crate) trait Behaviour {
    fn clusters(&self) -> impl Iterator<Item = &ClusterId>;
    fn update_context(&self, context: &mut SteeringContext, target_direction: Vec3);
}
