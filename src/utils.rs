use std::ops::{Add, Div, Mul};

use super::*;

use bevy::ecs::{
    lifecycle::{Add as AddEvent, HookContext},
    world::DeferredWorld,
};

pub(crate) type ActiveAgentsQuery<'w, 's, D, F = ()> = Query<'w, 's, D, (With<SteeringAgent>, F)>;
pub(crate) type ClusterQuery<'w, 's, D, F = ()> = Query<'w, 's, D, (With<Cluster>, F)>;

pub(crate) fn on_add_cluster_add_default_behaviour_weight<T: Component + Default>(
    trigger: On<AddEvent, Cluster>,
    mut commands: Commands,
    query: Query<&T>,
) {
    if query.get(trigger.entity).ok().is_none() {
        commands.entity(trigger.entity).insert(T::default());
    }
}

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

pub(crate) fn weighted_average<'a, T>(
    clusters: impl Iterator<Item = &'a ClusterId>,
    mut property: impl FnMut(&ClusterId) -> Option<(T, ClusterWeight)>,
) -> Option<T>
where
    T: Mul<f32, Output = T> + Add<T, Output = T> + Div<f32, Output = T>,
{
    let on_each_cluster = clusters.filter_map(|cluster_id| {
        let property = (property)(cluster_id);
        property.map(|(value, weight)| (value * weight.0, weight))
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
