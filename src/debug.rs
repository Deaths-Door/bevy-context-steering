use bevy::color::palettes::css::*;

use super::*;

pub struct SteeringDebugPlugin;

impl Plugin for SteeringDebugPlugin {
    fn build(&self, app: &mut App) {}
}

fn debug_steering_context(
    mut gizmos: Gizmos,
    query: ActiveAgentsQuery<(&SteeringContext, &GlobalTransform)>,
) {/*
    // TODO: improve this;
    let palette = [RED, BLUE, GREEN];
    for (context, transform) in query.iter() {
        let start = transform.translation();
        for ((id, behaviour), base_color) in context.iter().zip(palette.iter().cycle()) {
            let interest_color = *base_color;
            let danger_color = base_color.with_luminance(0.5);
            for (value, direction) in behaviour.field().iter().zip(DIRECTIONS.iter()) {
                let end_interest = transform.transform_point(direction * value.interest());
                let end_danger = transform.transform_point(direction * value.danger());
                gizmos.arrow(start, end_interest, interest_color);
                gizmos.arrow(start, end_danger, danger_color);
            }
        }
    } */
}
