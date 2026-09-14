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

/// A steering component that pushes the agent toward matching a fixed
/// target velocity, independent of any other entity.
///
/// Unlike [`Throttle`], which tracks another entity's current velocity via
/// a query, `ThrottleTo` reads its target directly off the component with
/// no lookup required — useful for a constant cruise speed, patrol pace,
/// or any case where "match this entity" doesn't apply.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Debug)]
#[component(
    on_add = on_add_into_steering_context::<Self>,
    on_remove = on_remove_from_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct ThrottleTo {
    /// The fixed velocity this agent will attempt to match.
    pub velocity: Vec3,
}

impl Throttle {
    /// Creates a new `Throttle` steering behavior targeting the specified entity.
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

impl ThrottleTo {
    /// Creates a new `ThrottleTo` steering behavior 
    pub fn new( velocity: Vec3) -> Self {
        Self { velocity }
    }
}

impl ThrottleTo {
    pub(crate) fn steering_behaviour_update(mut agent_query: ActiveAgentsQuery<BehaviourQueryData<Self>>) {
        agent_query
            .par_iter_mut()
            .for_each(|mut agent|{
                let target_velocity = agent.behaviour.velocity;

                agent.context.set_interest::<Self>(target_velocity);
                agent.context.overwrite_velocity::<Self>( target_velocity);

            })
    }
}


impl Throttle{
    pub(crate) fn steering_behaviour_update(mut agent_query: ActiveAgentsQuery<BehaviourQueryData<Self>>, entity_query : Query<&LinearVelocity> ) {
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

                agent.context.set_interest::<Self>(target_velocity);
                agent.context.overwrite_velocity::<Self>(target_velocity);

            })
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: Component> {
    entity : Entity, 
    behaviour: &'static T,
    context: &'static mut SteeringContext,
}