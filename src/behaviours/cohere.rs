use bevy_many_relationships::OutgoingRelationships;

use super::*;

/// Steering behavior that pulls the agent toward the center of mass of its neighbourhood
#[derive(Component, Debug, Reflect, Default, Deref, DerefMut)]
#[component(on_add = on_add_into_steering_context::<Self>, on_remove = on_remove_from_steering_context::<Self>)]
#[require(SteeringContext)]
pub struct Cohere {
    /// Distance-based attenuation profile controlling force magnitude drop-off.
    pub falloff: Falloff,
}

impl Cohere {
    /// Creates a new `Self`  no distance falloff ([`Falloff::None`]).
    pub const fn new() -> Self {
        Self {
            falloff: Falloff::None,
        }
    }

    /// Sets the falloff behavior
    pub const fn with_falloff(mut self, falloff: Falloff) -> Self {
        self.falloff = falloff;
        self
    }
}

impl Cohere {
    pub(crate) fn steering_behaviour_update(
        agent_query: ActiveAgentsQuery<BehaviourQueryData<Self>>,
    ) {
        update_internal::<Self>(agent_query)
    }
}

impl Behaviour for Cohere {
    fn update_context(&self, context: &mut SteeringContext, target_direction: Vec3) {
        let distance = target_direction.length();
        let factor = self.falloff.inwards_factor(distance);
        let target_direction = target_direction * factor;
        context.set_interest::<Self>(target_direction);
    }
}

pub(super) fn update_internal<T>(mut agent_query: ActiveAgentsQuery<BehaviourQueryData<T>>)
where
    T: Component + Behaviour,
{
    agent_query.par_iter_mut().for_each(|mut agent| {
        let neighbourhood = agent.neighbourhood;
        let sum_positions: Vec3 = neighbourhood
            .iter()
            .map(|(_, neighbour)| neighbour.hit_point)
            .sum();

        let length: f32 = neighbourhood.len() as f32;
        let center = sum_positions / length;
        let target_direction = center - agent.transform.translation;

        agent
            .behaviour
            .update_context(&mut *agent.context, target_direction);
    });
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: Component> {
    behaviour: &'static T,
    neighbourhood: &'static OutgoingRelationships<Neighbour>,
    transform: &'static Transform,
    context: &'static mut SteeringContext,
}

pub(crate) trait Behaviour {
    fn update_context(&self, context: &mut SteeringContext, target_direction: Vec3);
}

/*use bevy::platform::collections::HashMap;

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
    // TODO: INSTEAD OF THIS USE MULTIRELATIONSHIPS FOR THIS INSTEAD
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
 */
