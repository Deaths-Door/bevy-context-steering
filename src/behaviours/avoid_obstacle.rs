use bevy_many_relationships::OutgoingRelationships;

use crate::obstacles::Obstacle;

use super::*;


/// Steering behavior that pushes the agent away from nearby obstacles.
///
///  Obstacles that also have a `GlobalTransform` + `LinearVelocity` are
/// avoided at their *predicted* future position rather than where they are
/// right now; obstacles without a velocity component are
/// avoided at their current hit position, since there's nothing to predict.
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Debug)]
#[component(
    on_add = on_add_into_steering_context::<Self>, 
    on_remove = on_remove_from_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct AvoidObstacles { 
    pub falloff : Falloff,
    pub prediction : EntityPrediction 
}

impl AvoidObstacles {
    /// Creates a new `Self`  no distance falloff ([`Falloff::None`]).
    pub const fn new() -> Self {
        Self {
            falloff: Falloff::None,
            prediction : EntityPrediction::DEFAULT
        }
    }

    /// Sets the falloff behavior
    pub const fn with_falloff(mut self, falloff: Falloff) -> Self {
        self.falloff = falloff;
        self
    }

     /// Sets the preidiction
    pub const fn with_prediction(mut self, prediction : EntityPrediction) -> Self {
        self.prediction = prediction;
        self
    }
}

impl AvoidObstacles {
    pub(crate) fn steering_behaviour_update(
        mut agent_query: ActiveAgentsQuery<BehaviourQueryData>, 
        obstacle_query: Query<ObstacleQueryData>
    ) {
        agent_query
            .par_iter_mut()
            .for_each(|mut agent|{
                for (entity, obstacle ) in agent.obstacles.iter() {
                    let dir = match obstacle_query.get(entity).ok() {
                        Some(obstacle_item) => {
                            let target_translation = obstacle_item.transform.translation();
                            let target_velocity = **obstacle_item.velocity;

                            let agent_translation = agent.transform.translation();
                            let agent_velocity = **agent.velocity;

                            let predicted_position = agent.behaviour.prediction.predict_position(
                                agent_translation, target_translation, agent_velocity, target_velocity
                            );

                            predicted_position - agent_translation
                        }
                        None => {                            
                            let agent_translation = agent.transform.translation();
                            obstacle.shape_hit.point2 - agent_translation
                        },
                    };

                    agent.context.danger::<Self>(dir);
                }
            })
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData {
    context: &'static mut SteeringContext,

    behaviour : &'static AvoidObstacles,
    obstacles : &'static OutgoingRelationships<Obstacle>,
    transform : &'static GlobalTransform,
    velocity : &'static LinearVelocity
}

#[derive(QueryData)]
pub(crate) struct ObstacleQueryData {
    transform : &'static GlobalTransform,
    velocity : &'static LinearVelocity
}