use super::*;

/// Evade acts to steer the character away from another moving character
#[derive(Component, Debug, Reflect)]
#[component(
    on_add = on_add_into_steering_context::<Self>, 
    on_remove = on_remove_from_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct Evade { 
    pub target: Entity,
    pub prediction: EntityPrediction,
}

impl Evade {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            prediction: Default::default(),
        }
    }
}


impl pursuit::BehaviourData for Evade {
    fn target(&self) -> Entity {
        self.target
    }

    fn entity_prediction(&self) -> &EntityPrediction {
        &self.prediction
    }
}

impl Evade {
    pub(crate) fn steering_behaviour_update(
        query: ActiveAgentsQuery<pursuit::BehaviourQueryData<Self>>,
        target_query: Query<pursuit::BehaviourTargetQueryData>,
    ) {
        Pursuit::steering_impl(query, target_query, |desired_direction, context| {
             context.set_danger::<Self>(desired_direction);

            // This prevents snapping to default direction (perpendicular to target) when using danger-only.
            // By telling it to go directly opposite
            context.set_interest::<Self>(-desired_direction);
        });
    }
}