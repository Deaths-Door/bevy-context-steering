use super::*;

/// Evade acts to steer the character away from another moving character
#[derive(Component, Debug, Reflect)]
#[component(
    on_add = on_add_insert_into_steering_context::<Self>, 
    on_remove = on_remove_insert_into_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct Evade { pub target: Entity,
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
        mut query: ActiveAgentsQuery<pursuit::BehaviourQueryData<Self>>,
        target_query: Query<pursuit::BehaviourTargetQueryData>,
    ) {
        use pursuit::BehaviourData;
        
        query.par_iter_mut().for_each(|mut agent| {
            // Skip if the target entity no longer exists
            let Ok(target) = target_query.get(agent.behaviour.target()) else {
                warn!(
                    "Agent target {:?} for {} does not exist; skipping.",
                    agent.behaviour.target(),
                    std::any::type_name::<Self>(),
                );
                return;
            };

            // Means we dont want to update the map;
            let Some(desired_direction) = Pursuit::desired_direction(&agent, &target) else {
                return;
            };

            agent.context.set_danger::<Self>(desired_direction);

            // This prevents snapping to default direction (perpendicular to target) when using danger-only.
            // By telling it to go directly opposite
            agent.context.set_interest::<Self>(-desired_direction);
           
        });
    }

}