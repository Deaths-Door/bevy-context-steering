pub(super) mod seek;

use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
pub use seek::Seek;

use super::*;






pub fn on_add_insert_into_steering_context<T: 'static>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    if let Some(mut context) = world.get_mut::<SteeringContext>(entity) {
        context.insert::<T>();
    }
}

pub fn on_remove_insert_into_steering_context<T: 'static>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    if let Some(mut context) = world.get_mut::<SteeringContext>(entity) {
        context.remove::<T>();
    }
}
