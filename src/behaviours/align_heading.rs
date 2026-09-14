use super::*;

/// Aligns an agent's heading with the mean heading of its neighbours.
#[derive(Component, Debug, Clone, Reflect, Default)]
#[reflect(Component, Debug)]
#[component(
    on_add = on_add_into_steering_context::<Self>,
    on_remove = on_remove_from_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct AlignHeading;

impl AlignHeading {
    /// Creates a neighbour heading-alignment behaviour.
    pub const fn new() -> Self {
        Self
    }

    pub(crate) fn steering_behaviour_update(
        mut agent_query: ActiveAgentsQuery<BehaviourQueryData<Self>>,
    ) {
        agent_query.par_iter_mut().for_each(|mut agent| {
            agent
                .context
                .set_interest::<Self>(**agent.mean_heading);
        });
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: Component> {
    _behaviour: &'static T,
    mean_heading: &'static SteeringGroupMeanHeading,
    context: &'static mut SteeringContext,
}