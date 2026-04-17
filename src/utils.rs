use super::*;

use bevy::ecs::{lifecycle::HookContext, system::SystemParam, world::DeferredWorld};

pub(crate) fn on_add_insert_into_steering_context<T: 'static>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    if let Some(mut context) = world.get_mut::<SteeringContext>(entity) {
        context.insert::<T>();
    }
}

pub(crate) fn on_remove_insert_into_steering_context<T: 'static>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    if let Some(mut context) = world.get_mut::<SteeringContext>(entity) {
        context.remove::<T>();
    }
}
