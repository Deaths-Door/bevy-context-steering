use super::*;

/// Flee ( acts to steer the character away from a specified position in
/// global space.
#[derive(Component, Debug, Reflect)]
#[component(
    on_add = on_add_insert_into_steering_context::<Self>, 
    on_remove = on_remove_insert_into_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct Flee {
   pub  target: Vec3,
}

impl Flee {
    pub const fn new(target: Vec3) -> Self {
        Self { target }
    }
}


impl Flee {
    pub(crate) fn steering_behaviour_update(mut query: ActiveAgentsQuery<seek::BehaviourQueryData<Self>>) {
        query
            .par_iter_mut()
            .for_each(|mut agent| {
                let desired_direction = Seek::desired_direction(&agent);
                agent.context.set_danger::<Self>(desired_direction);
            });
    }
}

impl seek::BehaviourData for Flee {
    fn target(&self) -> Vec3 {
        self.target
    }
}
