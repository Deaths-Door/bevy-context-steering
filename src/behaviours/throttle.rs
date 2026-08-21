use super::*;

/// A steering component that matches the current linear speed of a target entity.
///
/// `Throttle` inspects a target entity's velocity (which can be the agent itself) 
/// and populates the steering context interest map along the target's movement vector.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Debug)]
#[component(
    on_add = on_add_into_steering_context::<Self>, 
    on_remove = on_remove_from_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct Throttle { 
    /// The target entity whose velocity this agent will attempt to match.
    pub entity : Entity 
}

impl Throttle {
    /// Creates a new `Throttle` steering behavior targeting the specified entity.
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}


impl Throttle{
    pub(crate) fn steering_behaviour_update(mut agent_query: ActiveAgentsQuery<BehaviourQueryData>, entity_query : Query<&LinearVelocity> ) {
        agent_query
            .par_iter_mut()
            .for_each(|mut agent|{
                let target_entity = agent.behaviour.entity;
                let Ok(target_velocity) = entity_query.get(target_entity) else {
                    warn!(
                        "[{:?}] Throttle failed: Target entity {:?} missing LinearVelocity",
                        agent.entity, target_entity
                    );
                    return;
                };

                let target_velocity = **target_velocity;
                let current_velocity = **agent.velocity;

                // Calculate required delta to reach target velocity
                let desired_steering = target_velocity - current_velocity;

                agent.context.set_interest::<Self>(desired_steering);
            })
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData {
    entity : Entity, 
    behaviour: &'static Throttle,
    velocity: &'static LinearVelocity,
    context: &'static mut SteeringContext,
}