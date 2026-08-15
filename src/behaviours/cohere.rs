use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    platform::collections::HashMap,
};

use super::*;

/// Steering behavior that pulls the agent toward the center of mass of its assigned clusters.
///
/// Each [`ClusterId`] represents a group the agent belongs to. The agent's final
/// interest vector is calculated as a weighted average of all cluster centers,
/// allowing for hierarchical grouping (e.g., Squad within a Company).
#[derive(Component, Debug, Reflect, Default, Deref, DerefMut)]
#[component(on_add = on_add_into_steering_context::<Self>, on_remove = on_remove_from_steering_context::<Self>)]
#[require(SteeringContext)]
pub struct Cohere {
    clusters: HashMap<ClusterId, ClusterWeight>,
}

impl Cohere {
    pub(crate) fn steering_behaviour_update(
        clusters: Res<ClusterMap>,
        agent_query: ActiveAgentsQuery<BehaviourQueryData<Self>>,
        cluster_query: Query<&ClusterCentre, With<Cluster>>,
    ) {
        update_internal::<Self>(clusters, agent_query, cluster_query)
    }
}

//--

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: Component> {
    behaviour: &'static T,
    transform: &'static GlobalTransform,
    context: &'static mut SteeringContext,
}

pub(super) fn update_internal<T>(
    clusters: Res<ClusterMap>,
    mut agent_query: ActiveAgentsQuery<BehaviourQueryData<T>>,
    cluster_query: Query<&ClusterCentre, With<Cluster>>,
) where
    T: Component + BehaviourData,
{
    agent_query.par_iter_mut().for_each(|mut agent| {
        let cluster_data = agent.behaviour.clusters().map(|(a, b)| (*a, *b));
        let Some(target_direction) = weighted_average(cluster_data, |id| {
            let cluster_entity = clusters.get(&id);
            let centre = cluster_entity.and_then(|entity| cluster_query.get(*entity).ok());
            centre.map(|c| &**c).cloned()
        }) else {
            return;
        };

        let agent_translation = agent.transform.translation();
        let cohesion_dir = target_direction - agent_translation;
        agent.behaviour.apply(&mut agent.context, cohesion_dir);
    });
}

pub(crate) trait BehaviourData {
    fn clusters(&self) -> impl Iterator<Item = (&ClusterId, &ClusterWeight)>;
    // direction is always towards the position
    fn apply(&self, context: &mut SteeringContext, cohesion_dir: Vec3);
}

impl BehaviourData for Cohere {
    fn clusters(&self) -> impl Iterator<Item = (&ClusterId, &ClusterWeight)> {
        self.iter()
    }

    fn apply(&self, context: &mut SteeringContext, cohesion_dir: Vec3) {
        context.set_interest::<Self>(cohesion_dir);
    }
}
//--

impl FromIterator<ClusterId> for Cohere {
    fn from_iter<T: IntoIterator<Item = ClusterId>>(iter: T) -> Self {
        Self::from_iter(
            iter.into_iter()
                .map(|value| (value, ClusterWeight::default())),
        )
    }
}

impl FromIterator<(ClusterId, ClusterWeight)> for Cohere {
    fn from_iter<T: IntoIterator<Item = (ClusterId, ClusterWeight)>>(iter: T) -> Self {
        Self {
            clusters: HashMap::from_iter(iter),
        }
    }
}

impl From<ClusterId> for Cohere {
    fn from(id: ClusterId) -> Self {
        Self::from_iter([id])
    }
}

impl From<(ClusterId, ClusterWeight)> for Cohere {
    fn from(id: (ClusterId, ClusterWeight)) -> Self {
        Self::from_iter([id])
    }
}
