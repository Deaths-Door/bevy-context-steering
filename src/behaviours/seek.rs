use super::*;

/// Seek (or pursuit of a static target) acts to steer the character towards a specified position in
/// global space.
#[derive(Component, Debug, Reflect)]
#[component(
    on_add = on_add_insert_into_steering_context::<Self>, 
    on_remove = on_remove_insert_into_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct Seek {
    pub target: Vec3,
}

impl Seek {
    pub const fn new(target: Vec3) -> Self {
        Self { target }
    }
}


impl Seek {
    pub(crate) fn steering_behaviour_update(mut query: ActiveAgentsQuery<BehaviourQueryData<Self>>) {
        query
            .par_iter_mut()
            .for_each(|mut agent| {
                let desired_direction = Self:: desired_direction(&agent);
                agent.context.set_interest::<Self>(desired_direction);
            });
    }

    pub(super) fn desired_direction<T:BehaviourData>(agent: &BehaviourQueryDataItem<'_, '_, T>) -> Vec3 {
        let position = agent.transform.translation();
        let target = agent.behaviour.target();
        let desired_direction = target - position;
        desired_direction
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: Component> {
    behaviour: &'static T,
    transform: &'static GlobalTransform,
    pub(crate) context: &'static mut SteeringContext,
}

pub(crate) trait BehaviourData : Component {
    fn target(&self) -> Vec3;
}

impl BehaviourData for Seek {
    fn target(&self) -> Vec3 {
        self.target
    }
}

