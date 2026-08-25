use std::ops::{Add, Div, Mul};

use super::*;

use bevy::ecs::{lifecycle::HookContext, system::SystemParam, world::DeferredWorld};

pub(crate) type ActiveAgentsQuery<'w, 's, D, F = ()> = Query<'w, 's, D, (With<SteeringAgent>, F)>;

pub(crate) fn on_add_into_steering_context<T: 'static>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    if let Some(mut context) = world.get_mut::<SteeringContext>(entity) {
        context.insert::<T>();
    }
}

pub(crate) fn on_remove_from_steering_context<T: 'static>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    if let Some(mut context) = world.get_mut::<SteeringContext>(entity) {
        context.remove::<T>();
    }
}

pub(crate) fn weighted_average<T>(
    clusters: impl Iterator<Item = (ClusterId, ClusterWeight)>,
    mut property_getter: impl FnMut(ClusterId) -> Option<T>,
) -> Option<T>
where
    T: Mul<f32, Output = T> + Add<T, Output = T> + Div<f32, Output = T>,
{
    let on_each_cluster = clusters.filter_map(|(cluster_id, weight)| {
        let value = (property_getter)(cluster_id);
        value.map(|value| (value * weight.0, weight))
    });

    let sum = on_each_cluster.reduce(|(a_value, a_weight), (b_value, b_weight)| {
        (a_value + b_value, a_weight + b_weight)
    });

    let average_value = sum
        // Handle case where weight is zero
        .filter(|(_, total_weights)| total_weights.0 > f32::EPSILON)
        // Find weighted direction
        .map(|(total_value, total_weights)| total_value / total_weights.0);

    average_value
}
