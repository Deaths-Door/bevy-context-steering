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
    target: Vec3,
}

impl Seek {
    pub const fn new(target: Vec3) -> Self {
        Self { target }
    }
}


#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: Component> {
    behaviour: &'static T,
    transform: &'static GlobalTransform,
    context: &'static mut SteeringContext,
}

pub(crate) trait BehaviourData {
    fn target(&self) -> Vec3;
    fn apply(&self, context: &mut SteeringContext, desired_direction: Vec3);
}

pub(super) fn update_internal<T>(mut query: Query<BehaviourQueryData<T>, With<SteeringAgent>>)
where
    T: Component + BehaviourData,
{
    query.par_iter_mut().for_each(|mut item| {
        let position = item.transform.translation();
        let target = item.behaviour.target();
        let desired_direction = target - position;
        item.behaviour.apply(&mut item.context, desired_direction);
    });
}


impl BehaviourData for Seek {
    fn target(&self) -> Vec3 {
        self.target
    }

    fn apply(&self, context: &mut SteeringContext, desired_direction: Vec3) {
        context.set_interest::<Self>(desired_direction);
    }
}

pub(crate) fn update(query: Query<BehaviourQueryData<Seek>, With<SteeringAgent>>) {
    update_internal(query);
}
