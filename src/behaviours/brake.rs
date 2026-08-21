use super::*;

/// A steering component that decelerates an agent by applying interest in the direction opposite to its current velocity.
///
/// `Brake` reduces the agent's speed down to zero opposing the entities current velocity
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Debug)]
#[component(
    on_add = on_add_into_steering_context::<Self>, 
    on_remove = on_remove_from_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct Brake;

impl Brake{
    pub(crate) fn steering_behaviour_update(mut query: ActiveAgentsQuery<BehaviourQueryData>) {
        query
            .par_iter_mut()
            .for_each(|mut agent|{
                let velocity = **agent.velocity;
                agent.context.set_interest::<Self>(-velocity);
            })
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData {
    pub(crate) velocity: &'static LinearVelocity,
    pub(crate) context: &'static mut SteeringContext,
}