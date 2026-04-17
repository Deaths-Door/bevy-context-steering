use super::*;

/// Pursuit acts to steer the character to another moving character
#[derive(Component, Debug, Reflect)]
#[component(
    on_add = on_add_insert_into_steering_context::<Self>,
    on_remove = on_remove_insert_into_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct Pursuit {
    pub target: Entity,
    pub prediction: EntityPrediction,
}

impl Pursuit {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            prediction: Default::default(),
        }
    }
}

impl Pursuit {
    pub(crate) fn steering_behaviour_update(
        mut query: ActiveAgentsQuery<BehaviourQueryData<Self>>,
        target_query: Query<BehaviourTargetQueryData>,
    ) {
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

            agent.context.set_interest::<Self>(desired_direction);
        });
    }

    pub(super) fn desired_direction<T: BehaviourData>(
        agent: &BehaviourQueryDataItem<'_, '_, T>,
        target: &BehaviourTargetQueryDataItem,
    ) -> Option<Vec3> {
        let target_data_changed = target.transform.is_changed() || target.velocity.is_changed();
        let agent_data_changed = agent.velocity.is_changed();
        let settings_changed = agent.behaviour.is_changed();

        // Did anything actually change for us to have to recalcuate it
        if !(target_data_changed || agent_data_changed || settings_changed) {
            return None;
        }

        let target_translation = target.transform.translation();
        let target_velocity = **target.velocity;

        let agent_translation = agent.transform.translation();
        let agent_velocity = **agent.velocity;

        let predicted_position = agent.behaviour.entity_prediction().predict_position(
            agent_translation,
            target_translation,
            agent_velocity,
            target_velocity,
        );

        let desired_direction = predicted_position - agent_translation;

        Some(desired_direction)
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: BehaviourData> {
    pub behaviour: Ref<'static, T>,
    transform: &'static GlobalTransform,
    velocity: Ref<'static, LinearVelocity>,
    pub context: &'static mut SteeringContext,
}

#[derive(QueryData)]
pub(crate) struct BehaviourTargetQueryData {
    transform: Ref<'static, GlobalTransform>,
    velocity: Ref<'static, LinearVelocity>,
}

pub(crate) trait BehaviourData: Component {
    fn target(&self) -> Entity;
    fn entity_prediction(&self) -> &EntityPrediction;
}

impl BehaviourData for Pursuit {
    fn target(&self) -> Entity {
        self.target
    }

    fn entity_prediction(&self) -> &EntityPrediction {
        &self.prediction
    }
}
